use crate::services::agent_memory_writer::{self, AgentMemoryEntry, AgentMemoryFileInfo};
use log::info;
use std::path::Path;

/// Get an agent's memory file
#[tauri::command]
pub fn get_agent_memory(
    agent_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<AgentMemoryFileInfo, String> {
    info!(
        "[AgentMemory] Reading memory for agent={} scope={} (project={:?})",
        agent_name, scope, project_path
    );
    let pp = project_path.as_deref().map(Path::new);
    agent_memory_writer::read_agent_memory(&agent_name, &scope, pp).map_err(|e| e.to_string())
}

/// Save content to an agent's memory file
#[tauri::command]
pub fn save_agent_memory(
    agent_name: String,
    scope: String,
    project_path: Option<String>,
    content: String,
) -> Result<AgentMemoryFileInfo, String> {
    info!(
        "[AgentMemory] Saving memory for agent={} scope={} ({} bytes)",
        agent_name,
        scope,
        content.len()
    );
    let pp = project_path.as_deref().map(Path::new);
    agent_memory_writer::write_agent_memory(&agent_name, &scope, pp, &content)
        .map_err(|e| e.to_string())
}

/// Delete an agent's memory file
#[tauri::command]
pub fn delete_agent_memory(
    agent_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<(), String> {
    info!(
        "[AgentMemory] Deleting memory for agent={} scope={}",
        agent_name, scope
    );
    let pp = project_path.as_deref().map(Path::new);
    agent_memory_writer::delete_agent_memory(&agent_name, &scope, pp).map_err(|e| e.to_string())
}

/// List all agents that have memory files in a given scope
#[tauri::command]
pub fn list_agent_memories(
    scope: String,
    project_path: Option<String>,
) -> Result<Vec<AgentMemoryEntry>, String> {
    info!(
        "[AgentMemory] Listing agent memories scope={} (project={:?})",
        scope, project_path
    );
    let pp = project_path.as_deref().map(Path::new);
    agent_memory_writer::list_agent_memories(&scope, pp).map_err(|e| e.to_string())
}
