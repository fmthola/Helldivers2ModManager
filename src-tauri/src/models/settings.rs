use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

mod ascii_string {
    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S: Serializer>(bytes: &[[u8; 16]], serializer: S) -> Result<S::Ok, S::Error> {
        let strings: Vec<&str> = bytes.iter()
            .map(|arr| str::from_utf8(arr).map_err(serde::ser::Error::custom))
            .collect::<Result<_, _>>()?;
        strings.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<[u8; 16]>, D::Error> {
        let strings = Vec::<String>::deserialize(deserializer)?;
        let mut result = Vec::with_capacity(strings.len());
        for s in strings {
            let bytes = s.as_bytes();
            if bytes.len() != 16 {
                return Err(serde::de::Error::custom("expected 16 characters"));
            }
            let mut arr = [0u8; 16];
            arr.copy_from_slice(bytes);
            result.push(arr);
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Version", rename_all = "PascalCase")]
pub enum Settings {
    #[serde(rename_all = "PascalCase")]
    V1 {
        game_path: PathBuf,
        #[serde(with = "ascii_string")]
        skip_list: Vec<[u8; 16]>
    }
}

impl Settings {
    pub async fn validate(&self) -> anyhow::Result<()> {
        match self {
            Settings::V1 { game_path, .. } => {
                if game_path.as_os_str().is_empty() {
                    anyhow::bail!("`game_path` is empty");
                }
                
                if !tokio::fs::try_exists(game_path).await.unwrap_or(false) {
                    anyhow::bail!("`game_path` doesn't exist");
                } else {
                    if !tokio::fs::try_exists(game_path.join("tools")).await.unwrap_or(false) {
                        anyhow::bail!("`game_path` doesn't contain dir \"tools\"");
                    }
                    if !tokio::fs::try_exists(game_path.join("data")).await.unwrap_or(false) {
                        anyhow::bail!("`game_path` doesn't contain dir \"data\"");
                    }
                    let bin_path = game_path.join("bin");
                    if !tokio::fs::try_exists(&bin_path).await.unwrap_or(false) {
                        anyhow::bail!("`game_path` doesn't contain dir \"bin\"");
                    } else {
                        if !tokio::fs::try_exists(bin_path.join("helldivers2.exe")).await.unwrap_or(false) {
                            anyhow::bail!("\"bin\" dir does not contain \"helldivers2.exe\"");
                        }
                    }
                }
                
                Ok(())
            },
        }
    }

    pub fn game_path(&self) -> &Path {
        match self {
            Settings::V1 { game_path, .. } => {
                game_path.as_path()
            },
        }
    }

    pub fn has_skip_entry(&self, s: &str) -> bool {
        match self {
            Settings::V1 { skip_list, .. } => {
                skip_list.iter()
                    .filter_map(|entry| {
                        str::from_utf8(entry).ok()
                    })
                    .any(|entry| entry == s)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Build a temp dir that looks like a valid Helldivers 2 install.
    fn valid_install() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("tools")).unwrap();
        fs::create_dir(root.join("data")).unwrap();
        fs::create_dir(root.join("bin")).unwrap();
        // Present under Proton too — only existence is checked.
        fs::write(root.join("bin").join("helldivers2.exe"), b"").unwrap();
        dir
    }

    fn settings_for(path: &Path) -> Settings {
        Settings::V1 { game_path: path.to_path_buf(), skip_list: vec![] }
    }

    #[tokio::test]
    async fn valid_install_passes() {
        let dir = valid_install();
        assert!(settings_for(dir.path()).validate().await.is_ok());
    }

    #[tokio::test]
    async fn empty_path_fails() {
        let s = Settings::V1 { game_path: PathBuf::new(), skip_list: vec![] };
        assert!(s.validate().await.is_err());
    }

    #[tokio::test]
    async fn missing_path_fails() {
        let s = settings_for(Path::new("/nonexistent/hd2/path/xyz"));
        assert!(s.validate().await.is_err());
    }

    #[tokio::test]
    async fn missing_tools_fails() {
        let dir = valid_install();
        fs::remove_dir_all(dir.path().join("tools")).unwrap();
        assert!(settings_for(dir.path()).validate().await.is_err());
    }

    #[tokio::test]
    async fn missing_data_fails() {
        let dir = valid_install();
        fs::remove_dir_all(dir.path().join("data")).unwrap();
        assert!(settings_for(dir.path()).validate().await.is_err());
    }

    #[tokio::test]
    async fn missing_exe_fails() {
        let dir = valid_install();
        fs::remove_file(dir.path().join("bin").join("helldivers2.exe")).unwrap();
        assert!(settings_for(dir.path()).validate().await.is_err());
    }

    #[test]
    fn skip_entry_lookup() {
        let s = Settings::V1 {
            game_path: PathBuf::new(),
            skip_list: vec![*b"0cf14e223de06a26"],
        };
        assert!(s.has_skip_entry("0cf14e223de06a26"));
        assert!(!s.has_skip_entry("ffffffffffffffff"));
    }
}