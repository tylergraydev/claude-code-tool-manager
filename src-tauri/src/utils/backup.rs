use anyhow::Result;
use std::path::Path;

/// Create a `.bak` backup of a file before modifying it.
///
/// Returns `Ok(())` if the file does not exist (nothing to back up).
/// The backup is placed alongside the original with `.bak` appended to the
/// full filename (e.g. `settings.json` → `settings.json.bak`,
/// `SKILL.md` → `SKILL.md.bak`).
pub fn backup_file(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let file_name = path
        .file_name()
        .ok_or_else(|| {
            anyhow::anyhow!("Cannot determine file name for backup: {}", path.display())
        })?
        .to_string_lossy();

    let backup_name = format!("{}.bak", file_name);
    let backup_path = path.with_file_name(backup_name);

    std::fs::copy(path, &backup_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create backup of {} before writing: {}",
            path.display(),
            e
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_nonexistent_file_succeeds() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.json");
        assert!(backup_file(&path).is_ok());
    }

    #[test]
    fn test_backup_json_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"key": "value"}"#).unwrap();

        backup_file(&path).unwrap();

        let backup = dir.path().join("settings.json.bak");
        assert!(backup.exists());
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            r#"{"key": "value"}"#
        );
    }

    #[test]
    fn test_backup_md_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("SKILL.md");
        std::fs::write(&path, "# My Skill").unwrap();

        backup_file(&path).unwrap();

        let backup = dir.path().join("SKILL.md.bak");
        assert!(backup.exists());
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "# My Skill");
    }

    #[test]
    fn test_backup_local_settings_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.local.json");
        std::fs::write(&path, "{}").unwrap();

        backup_file(&path).unwrap();

        let backup = dir.path().join("settings.local.json.bak");
        assert!(backup.exists());
    }

    #[test]
    fn test_backup_overwrites_previous_backup() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let backup = dir.path().join("config.json.bak");

        std::fs::write(&path, "v1").unwrap();
        backup_file(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "v1");

        std::fs::write(&path, "v2").unwrap();
        backup_file(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "v2");
    }
}
