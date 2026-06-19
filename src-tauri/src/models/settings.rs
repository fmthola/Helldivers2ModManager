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
                match check_game_path(game_path).await.first() {
                    Some(problem) => anyhow::bail!("invalid `game_path`: {}", problem),
                    None => Ok(()),
                }
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

/// Check a candidate Helldivers 2 install directory.
///
/// Returns a stable error key for each failed check (matching the frontend
/// i18n keys under `pages.settings.validation_error.game_path`). An empty
/// vector means the path is valid. Runs in Rust so it does not depend on the
/// webview's filesystem capability scope.
pub async fn check_game_path(path: &Path) -> Vec<&'static str> {
    let mut errors = Vec::new();

    if path.as_os_str().is_empty() {
        errors.push("empty");
        return errors;
    }
    if !tokio::fs::try_exists(path).await.unwrap_or(false) {
        errors.push("exists");
        return errors;
    }
    if !tokio::fs::try_exists(path.join("tools")).await.unwrap_or(false) {
        errors.push("tools_exists");
    }
    if !tokio::fs::try_exists(path.join("data")).await.unwrap_or(false) {
        errors.push("data_exists");
    }
    let bin = path.join("bin");
    if !tokio::fs::try_exists(&bin).await.unwrap_or(false) {
        errors.push("bin_exists");
    } else if !tokio::fs::try_exists(bin.join("helldivers2.exe")).await.unwrap_or(false) {
        errors.push("exe_exists");
    }

    errors
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

    // Lay out a valid install under `root`.
    fn make_install(root: &Path) {
        fs::create_dir_all(root.join("tools")).unwrap();
        fs::create_dir_all(root.join("data")).unwrap();
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("bin").join("helldivers2.exe"), b"").unwrap();
    }

    #[tokio::test]
    async fn check_valid_path_has_no_errors() {
        let dir = valid_install();
        assert!(check_game_path(dir.path()).await.is_empty());
    }

    // The path Steam uses contains a space ("Helldivers 2"). Confirm that is fine.
    #[tokio::test]
    async fn check_path_with_spaces_is_valid() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("steamapps/common/Helldivers 2");
        make_install(&root);
        assert!(check_game_path(&root).await.is_empty());
    }

    #[tokio::test]
    async fn check_empty_path() {
        assert_eq!(check_game_path(Path::new("")).await, vec!["empty"]);
    }

    #[tokio::test]
    async fn check_nonexistent_path() {
        assert_eq!(check_game_path(Path::new("/no/such/dir/xyz")).await, vec!["exists"]);
    }

    #[tokio::test]
    async fn check_reports_each_missing_piece() {
        let dir = valid_install();
        fs::remove_dir_all(dir.path().join("tools")).unwrap();
        assert_eq!(check_game_path(dir.path()).await, vec!["tools_exists"]);

        let dir = valid_install();
        fs::remove_dir_all(dir.path().join("data")).unwrap();
        assert_eq!(check_game_path(dir.path()).await, vec!["data_exists"]);

        let dir = valid_install();
        fs::remove_dir_all(dir.path().join("bin")).unwrap();
        assert_eq!(check_game_path(dir.path()).await, vec!["bin_exists"]);

        let dir = valid_install();
        fs::remove_file(dir.path().join("bin").join("helldivers2.exe")).unwrap();
        assert_eq!(check_game_path(dir.path()).await, vec!["exe_exists"]);
    }

    #[tokio::test]
    async fn check_reports_multiple_missing() {
        let dir = TempDir::new().unwrap();
        // Exists but empty: tools, data, and bin are all missing.
        let errs = check_game_path(dir.path()).await;
        assert!(errs.contains(&"tools_exists"));
        assert!(errs.contains(&"data_exists"));
        assert!(errs.contains(&"bin_exists"));
    }
}