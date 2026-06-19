use std::path::{Path, PathBuf};

pub async fn fix_path_casing(base: &Path, relative: &Path) -> anyhow::Result<PathBuf> {
    let mut current = base.to_path_buf();
    'components: for components in relative.components() {
        let component_str = components.as_os_str().to_string_lossy();
        
        let mut entries = tokio::fs::read_dir(&current).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_name().to_string_lossy().to_lowercase() == component_str.to_lowercase() {
                current.push(entry.file_name());
                continue 'components;
            }
        }

        current.push(components.as_os_str());
    }

    Ok(current.strip_prefix(base)?.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn corrects_casing_to_match_disk() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("Tools")).unwrap();
        fs::write(dir.path().join("Tools").join("File.txt"), b"").unwrap();

        let got = fix_path_casing(dir.path(), Path::new("tools/file.txt")).await.unwrap();
        assert_eq!(got, PathBuf::from("Tools").join("File.txt"));
    }

    #[tokio::test]
    async fn keeps_already_correct_path() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("data")).unwrap();

        let got = fix_path_casing(dir.path(), Path::new("data")).await.unwrap();
        assert_eq!(got, PathBuf::from("data"));
    }

    #[tokio::test]
    async fn passes_through_missing_components() {
        let dir = TempDir::new().unwrap();
        let got = fix_path_casing(dir.path(), Path::new("nope")).await.unwrap();
        assert_eq!(got, PathBuf::from("nope"));
    }
}