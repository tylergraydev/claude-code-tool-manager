use anyhow::Result;
use directories::BaseDirs;
use serde_json::{json, Value};
use std::path::Path;

/// Read an existing settings.json file or return an empty object
fn read_settings_file(path: &Path) -> Result<Value> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or(json!({})))
    } else {
        Ok(json!({}))
    }
}

/// Write settings.json file, preserving other settings
fn write_settings_file(path: &Path, settings: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Write the spinnerVerbs key to ~/.claude/settings.json
pub fn write_spinner_verbs_to_settings(mode: &str, verbs: &[String]) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings = read_settings_file(&settings_path)?;

    let mut sv_config = serde_json::Map::new();
    sv_config.insert("mode".to_string(), json!(mode));
    sv_config.insert("verbs".to_string(), json!(verbs));

    settings["spinnerVerbs"] = Value::Object(sv_config);

    write_settings_file(&settings_path, &settings)
}

/// Remove the spinnerVerbs key from ~/.claude/settings.json
pub fn remove_spinner_verbs_from_settings() -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings = read_settings_file(&settings_path)?;

    if let Some(obj) = settings.as_object_mut() {
        obj.remove("spinnerVerbs");
    }

    write_settings_file(&settings_path, &settings)
}

/// Read the current spinnerVerbs config from ~/.claude/settings.json
pub fn read_current_spinner_verbs_config() -> Result<Option<Value>> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    let settings_path = home.join(".claude").join("settings.json");

    let settings = read_settings_file(&settings_path)?;
    Ok(settings.get("spinnerVerbs").cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_spinner_verbs_to_path(
        settings_path: &Path,
        mode: &str,
        verbs: &[String],
    ) -> Result<()> {
        let mut settings = read_settings_file(settings_path)?;
        let mut sv_config = serde_json::Map::new();
        sv_config.insert("mode".to_string(), json!(mode));
        sv_config.insert("verbs".to_string(), json!(verbs));
        settings["spinnerVerbs"] = Value::Object(sv_config);
        write_settings_file(settings_path, &settings)
    }

    #[test]
    fn test_write_spinner_verbs_creates_config() {
        let dir = TempDir::new().unwrap();
        let settings_path = dir.path().join("settings.json");

        let verbs = vec!["Pondering".to_string(), "Crafting".to_string()];
        write_spinner_verbs_to_path(&settings_path, "append", &verbs).unwrap();

        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(settings["spinnerVerbs"]["mode"], "append");
        assert_eq!(settings["spinnerVerbs"]["verbs"][0], "Pondering");
        assert_eq!(settings["spinnerVerbs"]["verbs"][1], "Crafting");
    }

    #[test]
    fn test_write_preserves_other_settings() {
        let dir = TempDir::new().unwrap();
        let settings_path = dir.path().join("settings.json");

        // Write initial settings
        let initial = json!({"statusLine": {"type": "command"}});
        write_settings_file(&settings_path, &initial).unwrap();

        // Write spinner verbs
        let verbs = vec!["Brewing".to_string()];
        write_spinner_verbs_to_path(&settings_path, "replace", &verbs).unwrap();

        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();

        // Both keys should exist
        assert!(settings.get("statusLine").is_some());
        assert!(settings.get("spinnerVerbs").is_some());
        assert_eq!(settings["spinnerVerbs"]["mode"], "replace");
    }
}
