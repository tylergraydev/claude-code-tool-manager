use anyhow::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Info about a single agent's memory file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemoryFileInfo {
    pub agent_name: String,
    pub scope: String,
    pub exists: bool,
    pub file_path: String,
    pub content: String,
    pub last_modified: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Entry in the agent memory directory listing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMemoryEntry {
    pub agent_name: String,
    pub scope: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub last_modified: Option<String>,
}

/// Resolve the agent-memory directory for a given scope
fn resolve_agent_memory_dir(scope: &str, project_path: Option<&Path>) -> Result<PathBuf> {
    match scope {
        "user" => {
            let base_dirs =
                BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
            Ok(base_dirs.home_dir().join(".claude").join("agent-memory"))
        }
        "project" => {
            let project = project_path
                .ok_or_else(|| anyhow::anyhow!("Project path required for project scope"))?;
            Ok(project.join(".claude").join("agent-memory"))
        }
        _ => Err(anyhow::anyhow!("Invalid scope: {}", scope)),
    }
}

/// Resolve the MEMORY.md path for a specific agent
fn resolve_agent_memory_path(
    agent_name: &str,
    scope: &str,
    project_path: Option<&Path>,
) -> Result<PathBuf> {
    let dir = resolve_agent_memory_dir(scope, project_path)?;
    Ok(dir.join(agent_name).join("MEMORY.md"))
}

/// Read a single agent's memory file
pub fn read_agent_memory(
    agent_name: &str,
    scope: &str,
    project_path: Option<&Path>,
) -> Result<AgentMemoryFileInfo> {
    let path = resolve_agent_memory_path(agent_name, scope, project_path)?;
    let path_str = path.to_string_lossy().to_string();

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let content = content.replace("\r\n", "\n");
        let metadata = std::fs::metadata(&path)?;
        let last_modified = metadata.modified().ok().map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        });
        let size_bytes = Some(metadata.len());

        Ok(AgentMemoryFileInfo {
            agent_name: agent_name.to_string(),
            scope: scope.to_string(),
            exists: true,
            file_path: path_str,
            content,
            last_modified,
            size_bytes,
        })
    } else {
        Ok(AgentMemoryFileInfo {
            agent_name: agent_name.to_string(),
            scope: scope.to_string(),
            exists: false,
            file_path: path_str,
            content: String::new(),
            last_modified: None,
            size_bytes: None,
        })
    }
}

/// Write content to an agent's memory file
pub fn write_agent_memory(
    agent_name: &str,
    scope: &str,
    project_path: Option<&Path>,
    content: &str,
) -> Result<AgentMemoryFileInfo> {
    let path = resolve_agent_memory_path(agent_name, scope, project_path)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    crate::utils::backup::backup_file(&path)?;
    std::fs::write(&path, content)?;
    read_agent_memory(agent_name, scope, project_path)
}

/// Delete an agent's memory file
pub fn delete_agent_memory(
    agent_name: &str,
    scope: &str,
    project_path: Option<&Path>,
) -> Result<()> {
    let path = resolve_agent_memory_path(agent_name, scope, project_path)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    // Clean up empty agent directory
    if let Some(parent) = path.parent() {
        if parent.exists() && parent.read_dir()?.next().is_none() {
            let _ = std::fs::remove_dir(parent);
        }
    }
    Ok(())
}

/// List all agents that have memory files in a given scope
pub fn list_agent_memories(
    scope: &str,
    project_path: Option<&Path>,
) -> Result<Vec<AgentMemoryEntry>> {
    let dir = resolve_agent_memory_dir(scope, project_path)?;
    let mut entries = Vec::new();

    if !dir.exists() {
        return Ok(entries);
    }

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let agent_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if agent_name.is_empty() {
            continue;
        }

        let memory_path = entry_path.join("MEMORY.md");
        if memory_path.exists() {
            let metadata = std::fs::metadata(&memory_path)?;
            let last_modified = metadata.modified().ok().map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.to_rfc3339()
            });

            entries.push(AgentMemoryEntry {
                agent_name,
                scope: scope.to_string(),
                file_path: memory_path.to_string_lossy().to_string(),
                size_bytes: metadata.len(),
                last_modified,
            });
        }
    }

    entries.sort_by(|a, b| a.agent_name.cmp(&b.agent_name));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_nonexistent_agent_memory() {
        let dir = tempfile::tempdir().unwrap();
        let info = read_agent_memory("test-agent", "project", Some(dir.path())).unwrap();
        assert!(!info.exists);
        assert!(info.content.is_empty());
        assert_eq!(info.agent_name, "test-agent");
    }

    #[test]
    fn test_write_and_read_agent_memory() {
        let dir = tempfile::tempdir().unwrap();
        let content = "# Agent Memory\n\n- [task1](task1.md) — first task";

        let info = write_agent_memory("my-agent", "project", Some(dir.path()), content).unwrap();
        assert!(info.exists);
        assert_eq!(info.content, content);
        assert_eq!(info.agent_name, "my-agent");

        let read_info = read_agent_memory("my-agent", "project", Some(dir.path())).unwrap();
        assert!(read_info.exists);
        assert_eq!(read_info.content, content);
    }

    #[test]
    fn test_delete_agent_memory() {
        let dir = tempfile::tempdir().unwrap();
        write_agent_memory("del-agent", "project", Some(dir.path()), "content").unwrap();

        delete_agent_memory("del-agent", "project", Some(dir.path())).unwrap();

        let info = read_agent_memory("del-agent", "project", Some(dir.path())).unwrap();
        assert!(!info.exists);
    }

    #[test]
    fn test_list_agent_memories() {
        let dir = tempfile::tempdir().unwrap();
        write_agent_memory("agent-a", "project", Some(dir.path()), "memory a").unwrap();
        write_agent_memory("agent-b", "project", Some(dir.path()), "memory b").unwrap();

        let entries = list_agent_memories("project", Some(dir.path())).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].agent_name, "agent-a");
        assert_eq!(entries[1].agent_name, "agent-b");
    }
}
