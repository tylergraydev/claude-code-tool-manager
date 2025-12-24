//! MCP Tools for Claude Code Tool Manager
//!
//! This module defines all the tools exposed by the MCP server.

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::db::models::{
    CreateHookRequest, CreateMcpRequest, CreateSkillRequest, CreateSubAgentRequest,
};
use crate::db::Database;

/// The Tool Manager MCP Server handler
#[derive(Clone)]
pub struct ToolManagerServer {
    pub db: Arc<Mutex<Database>>,
    tool_router: ToolRouter<ToolManagerServer>,
}

impl std::fmt::Debug for ToolManagerServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolManagerServer").finish()
    }
}

impl ToolManagerServer {
    fn get_db(&self) -> Result<std::sync::MutexGuard<'_, Database>, String> {
        self.db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))
    }
}

// ============================================================================
// MCP Tool Parameter Types
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListMcpsParams {
    /// Filter by MCP type (stdio, sse, http)
    #[schemars(description = "Filter by MCP type: stdio, sse, or http")]
    pub filter_type: Option<String>,
    /// Search term for name or description
    #[schemars(description = "Search term to filter MCPs by name or description")]
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetByIdParams {
    /// The ID of the item to get
    #[schemars(description = "The unique ID of the item")]
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateMcpParams {
    /// Name of the MCP
    #[schemars(description = "Unique name for the MCP")]
    pub name: String,
    /// Type of MCP (stdio, sse, http)
    #[schemars(description = "Type of MCP: stdio, sse, or http")]
    #[serde(rename = "type")]
    pub mcp_type: String,
    /// Description of the MCP
    #[schemars(description = "Description of what the MCP does")]
    pub description: Option<String>,
    /// Command to run (for stdio type)
    #[schemars(description = "Command to execute (for stdio MCPs)")]
    pub command: Option<String>,
    /// Arguments for the command
    #[schemars(description = "Arguments for the command (for stdio MCPs)")]
    pub args: Option<Vec<String>>,
    /// URL endpoint (for sse/http types)
    #[schemars(description = "URL endpoint (for sse/http MCPs)")]
    pub url: Option<String>,
    /// HTTP headers
    #[schemars(description = "HTTP headers for authentication")]
    pub headers: Option<HashMap<String, String>>,
    /// Environment variables
    #[schemars(description = "Environment variables to set")]
    pub env: Option<HashMap<String, String>>,
    /// Icon emoji
    #[schemars(description = "Icon emoji for display")]
    pub icon: Option<String>,
    /// Tags for categorization
    #[schemars(description = "Tags for categorization")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateMcpParams {
    /// The ID of the MCP to update
    #[schemars(description = "The unique ID of the MCP to update")]
    pub id: i64,
    /// New name (optional)
    pub name: Option<String>,
    /// New type (optional)
    #[serde(rename = "type")]
    pub mcp_type: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
    /// New command (optional)
    pub command: Option<String>,
    /// New args (optional)
    pub args: Option<Vec<String>>,
    /// New URL (optional)
    pub url: Option<String>,
    /// New headers (optional)
    pub headers: Option<HashMap<String, String>>,
    /// New env vars (optional)
    pub env: Option<HashMap<String, String>>,
    /// New icon (optional)
    pub icon: Option<String>,
    /// New tags (optional)
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectMcpParams {
    /// Project ID
    #[schemars(description = "The project ID")]
    pub project_id: i64,
    /// MCP ID
    #[schemars(description = "The MCP ID")]
    pub mcp_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GlobalMcpParams {
    /// MCP ID to enable/disable globally
    #[schemars(description = "The MCP ID to enable or disable globally")]
    pub mcp_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSkillParams {
    /// Name of the skill
    #[schemars(description = "Unique name for the skill (used as slash command)")]
    pub name: String,
    /// Skill content (markdown)
    #[schemars(description = "The skill content/prompt in markdown")]
    pub content: String,
    /// Skill type (command or skill)
    #[schemars(description = "Type: 'command' for slash commands, 'skill' for agent skills")]
    pub skill_type: String,
    /// Description
    #[schemars(description = "Description of what the skill does")]
    pub description: Option<String>,
    /// Allowed tools
    #[schemars(description = "List of allowed tools for this skill")]
    pub allowed_tools: Option<Vec<String>>,
    /// Argument hint
    #[schemars(description = "Hint for expected arguments")]
    pub argument_hint: Option<String>,
    /// Model preference
    #[schemars(description = "Preferred model (sonnet, opus, haiku)")]
    pub model: Option<String>,
    /// Tags
    #[schemars(description = "Tags for categorization")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSubAgentParams {
    /// Name of the sub-agent
    #[schemars(description = "Unique name for the sub-agent")]
    pub name: String,
    /// Description
    #[schemars(description = "Short description of the sub-agent")]
    pub description: String,
    /// Content (system prompt)
    #[schemars(description = "The sub-agent's system prompt")]
    pub content: String,
    /// Allowed tools
    #[schemars(description = "List of tools the sub-agent can use")]
    pub tools: Option<Vec<String>>,
    /// Model preference
    #[schemars(description = "Preferred model (sonnet, opus, haiku)")]
    pub model: Option<String>,
    /// Permission mode
    #[schemars(description = "Permission mode: askUser or bypassPermissions")]
    pub permission_mode: Option<String>,
    /// Tags
    #[schemars(description = "Tags for categorization")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateHookParams {
    /// Name of the hook
    #[schemars(description = "Unique name for the hook")]
    pub name: String,
    /// Event type
    #[schemars(
        description = "Event type: PreToolUse, PostToolUse, Notification, Stop, SubAgentStop"
    )]
    pub event_type: String,
    /// Hook type
    #[schemars(description = "Hook type: 'command' or 'prompt'")]
    pub hook_type: String,
    /// Description
    #[schemars(description = "Description of what the hook does")]
    pub description: Option<String>,
    /// Matcher pattern
    #[schemars(description = "Tool name matcher pattern (e.g., 'Write|Edit')")]
    pub matcher: Option<String>,
    /// Command to run (for command hooks)
    #[schemars(description = "Shell command to execute")]
    pub command: Option<String>,
    /// Prompt (for prompt hooks)
    #[schemars(description = "Prompt to display to the model")]
    pub prompt: Option<String>,
    /// Timeout in ms
    #[schemars(description = "Timeout in milliseconds")]
    pub timeout: Option<i32>,
    /// Tags
    #[schemars(description = "Tags for categorization")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GlobalItemParams {
    /// Item ID to enable/disable globally
    #[schemars(description = "The item ID to enable or disable globally")]
    pub item_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncProjectParams {
    /// Project ID to sync
    #[schemars(description = "The project ID to sync configuration for")]
    pub project_id: i64,
}

// ============================================================================
// Tool Router Implementation
// ============================================================================

#[tool_router]
impl ToolManagerServer {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            tool_router: Self::tool_router(),
        }
    }

    // ========================================================================
    // MCP Management Tools
    // ========================================================================

    /// List all MCPs in the library
    #[tool(
        description = "List all MCPs in the library with optional filtering by type or search term"
    )]
    fn list_mcps(&self, Parameters(params): Parameters<ListMcpsParams>) -> Result<String, String> {
        let db = self.get_db()?;
        let mcps = db.get_all_mcps().map_err(|e| e.to_string())?;

        // Apply filters
        let filtered: Vec<_> = mcps
            .into_iter()
            .filter(|mcp| {
                if let Some(ref filter_type) = params.filter_type {
                    if mcp.mcp_type != *filter_type {
                        return false;
                    }
                }
                if let Some(ref search) = params.search {
                    let search_lower: String = search.to_lowercase();
                    let name_match = mcp.name.to_lowercase().contains(&search_lower);
                    let desc_match = mcp
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search_lower))
                        .unwrap_or(false);
                    if !name_match && !desc_match {
                        return false;
                    }
                }
                true
            })
            .collect();

        serde_json::to_string_pretty(&filtered).map_err(|e| e.to_string())
    }

    /// Get details of a specific MCP
    #[tool(description = "Get details of a specific MCP by its ID")]
    fn get_mcp(&self, Parameters(params): Parameters<GetByIdParams>) -> Result<String, String> {
        let db = self.get_db()?;
        let mcp = db
            .get_mcp_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("MCP with ID {} not found", params.id))?;

        serde_json::to_string_pretty(&mcp).map_err(|e| e.to_string())
    }

    /// Create a new MCP
    #[tool(description = "Create a new MCP in the library")]
    fn create_mcp(
        &self,
        Parameters(params): Parameters<CreateMcpParams>,
    ) -> Result<String, String> {
        let request = CreateMcpRequest {
            name: params.name,
            description: params.description,
            mcp_type: params.mcp_type,
            command: params.command,
            args: params.args,
            url: params.url,
            headers: params.headers,
            env: params.env,
            icon: params.icon,
            tags: params.tags,
        };

        let db = self.get_db()?;
        let mcp = db.create_mcp(&request).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(&mcp).map_err(|e| e.to_string())?;
        Ok(format!(
            "MCP '{}' created successfully:\n{}",
            mcp.name, json
        ))
    }

    /// Update an existing MCP
    #[tool(description = "Update an existing MCP by its ID")]
    fn update_mcp(
        &self,
        Parameters(params): Parameters<UpdateMcpParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;

        // Get existing MCP
        let mut mcp = db
            .get_mcp_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("MCP with ID {} not found", params.id))?;

        // Apply updates
        if let Some(name) = params.name {
            mcp.name = name;
        }
        if let Some(mcp_type) = params.mcp_type {
            mcp.mcp_type = mcp_type;
        }
        if params.description.is_some() {
            mcp.description = params.description;
        }
        if params.command.is_some() {
            mcp.command = params.command;
        }
        if params.args.is_some() {
            mcp.args = params.args;
        }
        if params.url.is_some() {
            mcp.url = params.url;
        }
        if params.headers.is_some() {
            mcp.headers = params.headers;
        }
        if params.env.is_some() {
            mcp.env = params.env;
        }
        if params.icon.is_some() {
            mcp.icon = params.icon;
        }
        if params.tags.is_some() {
            mcp.tags = params.tags;
        }

        let updated = db.update_mcp(&mcp).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(&updated).map_err(|e| e.to_string())?;
        Ok(format!(
            "MCP '{}' updated successfully:\n{}",
            updated.name, json
        ))
    }

    /// Delete an MCP
    #[tool(description = "Delete an MCP from the library by its ID")]
    fn delete_mcp(&self, Parameters(params): Parameters<GetByIdParams>) -> Result<String, String> {
        let db = self.get_db()?;

        // Get MCP name for confirmation message
        let mcp = db
            .get_mcp_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("MCP with ID {} not found", params.id))?;

        db.delete_mcp(params.id).map_err(|e| e.to_string())?;

        Ok(format!(
            "MCP '{}' (ID: {}) deleted successfully",
            mcp.name, params.id
        ))
    }

    // ========================================================================
    // Project Management Tools
    // ========================================================================

    /// List all projects
    #[tool(description = "List all registered projects")]
    fn list_projects(&self) -> Result<String, String> {
        let db = self.get_db()?;
        let projects = db.get_all_projects().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&projects).map_err(|e| e.to_string())
    }

    /// Get project details
    #[tool(description = "Get project details including assigned MCPs by project ID")]
    fn get_project(&self, Parameters(params): Parameters<GetByIdParams>) -> Result<String, String> {
        let db = self.get_db()?;
        let project = db
            .get_project_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project with ID {} not found", params.id))?;

        serde_json::to_string_pretty(&project).map_err(|e| e.to_string())
    }

    /// Assign an MCP to a project
    #[tool(description = "Assign an MCP to a project")]
    fn assign_mcp_to_project(
        &self,
        Parameters(params): Parameters<ProjectMcpParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.assign_mcp_to_project(params.project_id, params.mcp_id)
            .map_err(|e| e.to_string())?;
        Ok(format!(
            "MCP {} assigned to project {}",
            params.mcp_id, params.project_id
        ))
    }

    /// Remove an MCP from a project
    #[tool(description = "Remove an MCP assignment from a project")]
    fn remove_mcp_from_project(
        &self,
        Parameters(params): Parameters<ProjectMcpParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.remove_mcp_from_project(params.project_id, params.mcp_id)
            .map_err(|e| e.to_string())?;
        Ok(format!(
            "MCP {} removed from project {}",
            params.mcp_id, params.project_id
        ))
    }

    // ========================================================================
    // Global MCP Settings Tools
    // ========================================================================

    /// List globally enabled MCPs
    #[tool(description = "List all globally enabled MCPs")]
    fn list_global_mcps(&self) -> Result<String, String> {
        let db = self.get_db()?;
        let global_mcps = db.get_global_mcps().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&global_mcps).map_err(|e| e.to_string())
    }

    /// Enable an MCP globally
    #[tool(description = "Enable an MCP globally (affects all projects)")]
    fn enable_global_mcp(
        &self,
        Parameters(params): Parameters<GlobalMcpParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.add_global_mcp(params.mcp_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("MCP {} enabled globally", params.mcp_id))
    }

    /// Disable an MCP globally
    #[tool(description = "Disable an MCP globally")]
    fn disable_global_mcp(
        &self,
        Parameters(params): Parameters<GlobalMcpParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.remove_global_mcp(params.mcp_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("MCP {} disabled globally", params.mcp_id))
    }

    // ========================================================================
    // Skills Tools
    // ========================================================================

    /// List all skills
    #[tool(description = "List all skills in the library")]
    fn list_skills(&self) -> Result<String, String> {
        let db = self.get_db()?;
        let skills = db.get_all_skills().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&skills).map_err(|e| e.to_string())
    }

    /// Get a specific skill
    #[tool(description = "Get details of a specific skill by its ID")]
    fn get_skill(&self, Parameters(params): Parameters<GetByIdParams>) -> Result<String, String> {
        let db = self.get_db()?;
        let skill = db
            .get_skill_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Skill with ID {} not found", params.id))?;

        serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())
    }

    /// Create a new skill
    #[tool(description = "Create a new skill (slash command or agent skill)")]
    fn create_skill(
        &self,
        Parameters(params): Parameters<CreateSkillParams>,
    ) -> Result<String, String> {
        let request = CreateSkillRequest {
            name: params.name,
            description: params.description,
            content: params.content,
            skill_type: params.skill_type,
            allowed_tools: params.allowed_tools,
            argument_hint: params.argument_hint,
            model: params.model,
            disable_model_invocation: None,
            tags: params.tags,
        };

        let db = self.get_db()?;
        let skill = db.create_skill(&request).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())?;
        Ok(format!(
            "Skill '{}' created successfully:\n{}",
            skill.name, json
        ))
    }

    /// Delete a skill
    #[tool(description = "Delete a skill from the library by its ID")]
    fn delete_skill(
        &self,
        Parameters(params): Parameters<GetByIdParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;

        let skill = db
            .get_skill_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Skill with ID {} not found", params.id))?;

        db.delete_skill(params.id).map_err(|e| e.to_string())?;

        Ok(format!("Skill '{}' deleted successfully", skill.name))
    }

    /// Enable a skill globally
    #[tool(description = "Enable a skill globally")]
    fn enable_global_skill(
        &self,
        Parameters(params): Parameters<GlobalItemParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.add_global_skill(params.item_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("Skill {} enabled globally", params.item_id))
    }

    /// Disable a skill globally
    #[tool(description = "Disable a skill globally")]
    fn disable_global_skill(
        &self,
        Parameters(params): Parameters<GlobalItemParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.remove_global_skill(params.item_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("Skill {} disabled globally", params.item_id))
    }

    // ========================================================================
    // SubAgents Tools
    // ========================================================================

    /// List all sub-agents
    #[tool(description = "List all sub-agents in the library")]
    fn list_subagents(&self) -> Result<String, String> {
        let db = self.get_db()?;
        let subagents = db.get_all_subagents().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&subagents).map_err(|e| e.to_string())
    }

    /// Get a specific sub-agent
    #[tool(description = "Get details of a specific sub-agent by its ID")]
    fn get_subagent(
        &self,
        Parameters(params): Parameters<GetByIdParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        let subagent = db
            .get_subagent_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("SubAgent with ID {} not found", params.id))?;

        serde_json::to_string_pretty(&subagent).map_err(|e| e.to_string())
    }

    /// Create a new sub-agent
    #[tool(description = "Create a new sub-agent")]
    fn create_subagent(
        &self,
        Parameters(params): Parameters<CreateSubAgentParams>,
    ) -> Result<String, String> {
        let request = CreateSubAgentRequest {
            name: params.name,
            description: params.description,
            content: params.content,
            tools: params.tools,
            model: params.model,
            permission_mode: params.permission_mode,
            skills: None,
            tags: params.tags,
        };

        let db = self.get_db()?;
        let subagent = db.create_subagent(&request).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(&subagent).map_err(|e| e.to_string())?;
        Ok(format!(
            "SubAgent '{}' created successfully:\n{}",
            subagent.name, json
        ))
    }

    /// Delete a sub-agent
    #[tool(description = "Delete a sub-agent from the library by its ID")]
    fn delete_subagent(
        &self,
        Parameters(params): Parameters<GetByIdParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;

        let subagent = db
            .get_subagent_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("SubAgent with ID {} not found", params.id))?;

        db.delete_subagent(params.id).map_err(|e| e.to_string())?;

        Ok(format!("SubAgent '{}' deleted successfully", subagent.name))
    }

    /// Enable a sub-agent globally
    #[tool(description = "Enable a sub-agent globally")]
    fn enable_global_subagent(
        &self,
        Parameters(params): Parameters<GlobalItemParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.add_global_subagent(params.item_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("SubAgent {} enabled globally", params.item_id))
    }

    /// Disable a sub-agent globally
    #[tool(description = "Disable a sub-agent globally")]
    fn disable_global_subagent(
        &self,
        Parameters(params): Parameters<GlobalItemParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.remove_global_subagent(params.item_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("SubAgent {} disabled globally", params.item_id))
    }

    // ========================================================================
    // Hooks Tools
    // ========================================================================

    /// List all hooks
    #[tool(description = "List all hooks in the library")]
    fn list_hooks(&self) -> Result<String, String> {
        let db = self.get_db()?;
        let hooks = db.get_all_hooks().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&hooks).map_err(|e| e.to_string())
    }

    /// Get a specific hook
    #[tool(description = "Get details of a specific hook by its ID")]
    fn get_hook(&self, Parameters(params): Parameters<GetByIdParams>) -> Result<String, String> {
        let db = self.get_db()?;
        let hook = db
            .get_hook_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Hook with ID {} not found", params.id))?;

        serde_json::to_string_pretty(&hook).map_err(|e| e.to_string())
    }

    /// Create a new hook
    #[tool(description = "Create a new hook")]
    fn create_hook(
        &self,
        Parameters(params): Parameters<CreateHookParams>,
    ) -> Result<String, String> {
        let request = CreateHookRequest {
            name: params.name,
            description: params.description,
            event_type: params.event_type,
            matcher: params.matcher,
            hook_type: params.hook_type,
            command: params.command,
            prompt: params.prompt,
            timeout: params.timeout,
            tags: params.tags,
        };

        let db = self.get_db()?;
        let hook = db.create_hook(&request).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(&hook).map_err(|e| e.to_string())?;
        Ok(format!(
            "Hook '{}' created successfully:\n{}",
            hook.name, json
        ))
    }

    /// Delete a hook
    #[tool(description = "Delete a hook from the library by its ID")]
    fn delete_hook(&self, Parameters(params): Parameters<GetByIdParams>) -> Result<String, String> {
        let db = self.get_db()?;

        let hook = db
            .get_hook_by_id(params.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Hook with ID {} not found", params.id))?;

        db.delete_hook(params.id).map_err(|e| e.to_string())?;

        Ok(format!("Hook '{}' deleted successfully", hook.name))
    }

    /// Enable a hook globally
    #[tool(description = "Enable a hook globally")]
    fn enable_global_hook(
        &self,
        Parameters(params): Parameters<GlobalItemParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.add_global_hook(params.item_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("Hook {} enabled globally", params.item_id))
    }

    /// Disable a hook globally
    #[tool(description = "Disable a hook globally")]
    fn disable_global_hook(
        &self,
        Parameters(params): Parameters<GlobalItemParams>,
    ) -> Result<String, String> {
        let db = self.get_db()?;
        db.remove_global_hook(params.item_id)
            .map_err(|e| e.to_string())?;
        Ok(format!("Hook {} disabled globally", params.item_id))
    }
}

// ============================================================================
// ServerHandler Implementation
// ============================================================================

#[tool_handler]
impl ServerHandler for ToolManagerServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Claude Code Tool Manager MCP Server. Manage MCPs, Skills, Sub-Agents, Hooks, and Projects programmatically."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
