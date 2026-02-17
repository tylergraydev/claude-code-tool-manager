use anyhow::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingBlock {
    pub context: String,
    pub bindings: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingsFile {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub bindings: Vec<KeybindingBlock>,
}

/// Get the path to ~/.claude/keybindings.json
pub fn keybindings_path() -> Result<PathBuf> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(base_dirs
        .home_dir()
        .join(".claude")
        .join("keybindings.json"))
}

/// Read keybindings from the default path. Returns empty bindings if file is missing/malformed.
pub fn read_keybindings() -> Result<KeybindingsFile> {
    let path = keybindings_path()?;
    read_keybindings_from_path(&path)
}

/// Read keybindings from a specific path (testable variant).
pub fn read_keybindings_from_path(path: &Path) -> Result<KeybindingsFile> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        match serde_json::from_str::<KeybindingsFile>(&content) {
            Ok(kb) => Ok(kb),
            Err(_) => Ok(KeybindingsFile {
                schema: None,
                bindings: vec![],
            }),
        }
    } else {
        Ok(KeybindingsFile {
            schema: None,
            bindings: vec![],
        })
    }
}

/// Write keybindings to the default path. Creates parent dir, filters empty contexts.
pub fn write_keybindings(kb: &KeybindingsFile) -> Result<()> {
    let path = keybindings_path()?;
    write_keybindings_to_path(&path, kb)
}

/// Write keybindings to a specific path (testable variant).
pub fn write_keybindings_to_path(path: &Path, kb: &KeybindingsFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Filter out context blocks with empty bindings
    let filtered = KeybindingsFile {
        schema: kb.schema.clone(),
        bindings: kb
            .bindings
            .iter()
            .filter(|b| !b.bindings.is_empty())
            .cloned()
            .collect(),
    };

    let content = serde_json::to_string_pretty(&filtered)?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_read_nonexistent_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("keybindings.json");

        let kb = read_keybindings_from_path(&path).unwrap();
        assert!(kb.bindings.is_empty());
        assert!(kb.schema.is_none());
    }

    #[test]
    fn test_read_empty_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("keybindings.json");
        fs::write(&path, "").unwrap();

        let kb = read_keybindings_from_path(&path).unwrap();
        assert!(kb.bindings.is_empty());
    }

    #[test]
    fn test_read_malformed_json_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("keybindings.json");
        fs::write(&path, "not valid json {{{").unwrap();

        let kb = read_keybindings_from_path(&path).unwrap();
        assert!(kb.bindings.is_empty());
    }

    #[test]
    fn test_write_and_read_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("keybindings.json");

        let mut bindings_map = HashMap::new();
        bindings_map.insert(
            "ctrl+e".to_string(),
            Some("chat:externalEditor".to_string()),
        );

        let kb = KeybindingsFile {
            schema: Some("https://www.schemastore.org/claude-code-keybindings.json".to_string()),
            bindings: vec![KeybindingBlock {
                context: "Chat".to_string(),
                bindings: bindings_map,
            }],
        };

        write_keybindings_to_path(&path, &kb).unwrap();
        let read_back = read_keybindings_from_path(&path).unwrap();

        assert_eq!(read_back.bindings.len(), 1);
        assert_eq!(read_back.bindings[0].context, "Chat");
        assert_eq!(
            read_back.bindings[0].bindings.get("ctrl+e"),
            Some(&Some("chat:externalEditor".to_string()))
        );
    }

    #[test]
    fn test_write_preserves_schema() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("keybindings.json");

        let kb = KeybindingsFile {
            schema: Some("https://www.schemastore.org/claude-code-keybindings.json".to_string()),
            bindings: vec![KeybindingBlock {
                context: "Global".to_string(),
                bindings: HashMap::from([("ctrl+q".to_string(), Some("app:exit".to_string()))]),
            }],
        };

        write_keybindings_to_path(&path, &kb).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("$schema"));
        assert!(content.contains("schemastore.org"));

        let read_back = read_keybindings_from_path(&path).unwrap();
        assert_eq!(
            read_back.schema,
            Some("https://www.schemastore.org/claude-code-keybindings.json".to_string())
        );
    }

    #[test]
    fn test_write_with_null_values() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("keybindings.json");

        let kb = KeybindingsFile {
            schema: None,
            bindings: vec![KeybindingBlock {
                context: "Chat".to_string(),
                bindings: HashMap::from([("ctrl+u".to_string(), None)]),
            }],
        };

        write_keybindings_to_path(&path, &kb).unwrap();
        let read_back = read_keybindings_from_path(&path).unwrap();

        assert_eq!(read_back.bindings[0].bindings.get("ctrl+u"), Some(&None));
    }

    #[test]
    fn test_write_filters_empty_context_blocks() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("keybindings.json");

        let kb = KeybindingsFile {
            schema: None,
            bindings: vec![
                KeybindingBlock {
                    context: "Chat".to_string(),
                    bindings: HashMap::from([(
                        "ctrl+e".to_string(),
                        Some("chat:externalEditor".to_string()),
                    )]),
                },
                KeybindingBlock {
                    context: "Global".to_string(),
                    bindings: HashMap::new(), // empty — should be filtered
                },
            ],
        };

        write_keybindings_to_path(&path, &kb).unwrap();
        let read_back = read_keybindings_from_path(&path).unwrap();

        assert_eq!(read_back.bindings.len(), 1);
        assert_eq!(read_back.bindings[0].context, "Chat");
    }

    #[test]
    fn test_write_creates_parent_directory() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("subdir").join("keybindings.json");

        let kb = KeybindingsFile {
            schema: None,
            bindings: vec![KeybindingBlock {
                context: "Global".to_string(),
                bindings: HashMap::from([("ctrl+q".to_string(), Some("app:exit".to_string()))]),
            }],
        };

        write_keybindings_to_path(&path, &kb).unwrap();
        assert!(path.exists());
    }
}
