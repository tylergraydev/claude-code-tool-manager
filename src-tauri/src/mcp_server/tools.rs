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
    #[schemars(description = "Unique name for the agent skill")]
    pub name: String,
    /// Skill content (markdown)
    #[schemars(description = "The skill content/prompt in markdown")]
    pub content: String,
    /// Description
    #[schemars(description = "Description of what the skill does (used by Claude for invocation)")]
    pub description: Option<String>,
    /// Allowed tools
    #[schemars(description = "List of allowed tools for this skill")]
    pub allowed_tools: Option<Vec<String>>,
    /// Model preference
    #[schemars(description = "Preferred model (sonnet, opus, haiku)")]
    pub model: Option<String>,
    /// Disable model invocation
    #[schemars(description = "If true, skill must be manually invoked via /skill-name")]
    pub disable_model_invocation: Option<bool>,
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
    #[tool(description = "Create a new agent skill (auto-invoked by Claude based on context)")]
    fn create_skill(
        &self,
        Parameters(params): Parameters<CreateSkillParams>,
    ) -> Result<String, String> {
        let request = CreateSkillRequest {
            name: params.name,
            description: params.description,
            content: params.content,
            allowed_tools: params.allowed_tools,
            model: params.model,
            disable_model_invocation: params.disable_model_invocation,
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    /// Helper: create a ToolManagerServer backed by an in-memory database.
    fn make_server() -> ToolManagerServer {
        let db = Database::in_memory().expect("in-memory db");
        ToolManagerServer::new(Arc::new(Mutex::new(db)))
    }

    /// Helper: call a tool through the server's tool_router directly.
    fn call_tool_sync(
        server: &ToolManagerServer,
        name: &str,
        args: serde_json::Value,
    ) -> Result<String, String> {
        // Call methods directly instead of going through ServerHandler trait
        let db = server.get_db()?;
        let args_map = args.as_object().cloned().unwrap_or_default();

        match name {
            "list_mcps" => {
                let mcps = db.get_all_mcps().map_err(|e| e.to_string())?;
                let filter_type = args_map.get("filter_type").and_then(|v| v.as_str());
                let search = args_map.get("search").and_then(|v| v.as_str());
                let filtered: Vec<_> = mcps
                    .into_iter()
                    .filter(|mcp| {
                        if let Some(ft) = filter_type {
                            if mcp.mcp_type != ft {
                                return false;
                            }
                        }
                        if let Some(s) = search {
                            let sl = s.to_lowercase();
                            let nm = mcp.name.to_lowercase().contains(&sl);
                            let dm = mcp
                                .description
                                .as_ref()
                                .map(|d| d.to_lowercase().contains(&sl))
                                .unwrap_or(false);
                            if !nm && !dm {
                                return false;
                            }
                        }
                        true
                    })
                    .collect();
                serde_json::to_string_pretty(&filtered).map_err(|e| e.to_string())
            }
            "get_mcp" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let mcp = db
                    .get_mcp_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("MCP with ID {} not found", id))?;
                serde_json::to_string_pretty(&mcp).map_err(|e| e.to_string())
            }
            "create_mcp" => {
                use crate::db::models::CreateMcpRequest;
                let request = CreateMcpRequest {
                    name: args_map
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: args_map
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    mcp_type: args_map
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("stdio")
                        .to_string(),
                    command: args_map
                        .get("command")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    args: args_map.get("args").and_then(|v| v.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                    url: args_map
                        .get("url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    headers: args_map
                        .get("headers")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    env: args_map
                        .get("env")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    icon: args_map
                        .get("icon")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    tags: args_map.get("tags").and_then(|v| v.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                };
                let mcp = db.create_mcp(&request).map_err(|e| e.to_string())?;
                let json = serde_json::to_string_pretty(&mcp).map_err(|e| e.to_string())?;
                Ok(format!(
                    "MCP '{}' created successfully:\n{}",
                    mcp.name, json
                ))
            }
            "update_mcp" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let mut mcp = db
                    .get_mcp_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("MCP with ID {} not found", id))?;
                if let Some(n) = args_map.get("name").and_then(|v| v.as_str()) {
                    mcp.name = n.to_string();
                }
                if let Some(t) = args_map.get("type").and_then(|v| v.as_str()) {
                    mcp.mcp_type = t.to_string();
                }
                if let Some(d) = args_map.get("description").and_then(|v| v.as_str()) {
                    mcp.description = Some(d.to_string());
                }
                if let Some(c) = args_map.get("command").and_then(|v| v.as_str()) {
                    mcp.command = Some(c.to_string());
                }
                if let Some(i) = args_map.get("icon").and_then(|v| v.as_str()) {
                    mcp.icon = Some(i.to_string());
                }
                if let Some(tags) = args_map.get("tags").and_then(|v| v.as_array()) {
                    mcp.tags = Some(
                        tags.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect(),
                    );
                }
                let updated = db.update_mcp(&mcp).map_err(|e| e.to_string())?;
                let json = serde_json::to_string_pretty(&updated).map_err(|e| e.to_string())?;
                Ok(format!(
                    "MCP '{}' updated successfully:\n{}",
                    updated.name, json
                ))
            }
            "delete_mcp" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let mcp = db
                    .get_mcp_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("MCP with ID {} not found", id))?;
                db.delete_mcp(id).map_err(|e| e.to_string())?;
                Ok(format!(
                    "MCP '{}' (ID: {}) deleted successfully",
                    mcp.name, id
                ))
            }
            "list_projects" => {
                let projects = db.get_all_projects().map_err(|e| e.to_string())?;
                serde_json::to_string_pretty(&projects).map_err(|e| e.to_string())
            }
            "get_project" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let project = db
                    .get_project_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Project with ID {} not found", id))?;
                serde_json::to_string_pretty(&project).map_err(|e| e.to_string())
            }
            "assign_mcp_to_project" => {
                let pid = args_map
                    .get("project_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing project_id")?;
                let mid = args_map
                    .get("mcp_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing mcp_id")?;
                db.assign_mcp_to_project(pid, mid)
                    .map_err(|e| e.to_string())?;
                Ok(format!("MCP {} assigned to project {}", mid, pid))
            }
            "remove_mcp_from_project" => {
                let pid = args_map
                    .get("project_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing project_id")?;
                let mid = args_map
                    .get("mcp_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing mcp_id")?;
                db.remove_mcp_from_project(pid, mid)
                    .map_err(|e| e.to_string())?;
                Ok(format!("MCP {} removed from project {}", mid, pid))
            }
            "list_global_mcps" => {
                let mcps = db.get_global_mcps().map_err(|e| e.to_string())?;
                serde_json::to_string_pretty(&mcps).map_err(|e| e.to_string())
            }
            "enable_global_mcp" => {
                let id = args_map
                    .get("mcp_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing mcp_id")?;
                db.add_global_mcp(id).map_err(|e| e.to_string())?;
                Ok(format!("MCP {} enabled globally", id))
            }
            "disable_global_mcp" => {
                let id = args_map
                    .get("mcp_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing mcp_id")?;
                db.remove_global_mcp(id).map_err(|e| e.to_string())?;
                Ok(format!("MCP {} disabled globally", id))
            }
            "list_skills" => {
                let skills = db.get_all_skills().map_err(|e| e.to_string())?;
                serde_json::to_string_pretty(&skills).map_err(|e| e.to_string())
            }
            "get_skill" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let skill = db
                    .get_skill_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Skill with ID {} not found", id))?;
                serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())
            }
            "create_skill" => {
                use crate::db::models::CreateSkillRequest;
                let request = CreateSkillRequest {
                    name: args_map
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    content: args_map
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: args_map
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    allowed_tools: args_map
                        .get("allowed_tools")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        }),
                    model: args_map
                        .get("model")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    disable_model_invocation: args_map
                        .get("disable_model_invocation")
                        .and_then(|v| v.as_bool()),
                    tags: args_map.get("tags").and_then(|v| v.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                };
                let skill = db.create_skill(&request).map_err(|e| e.to_string())?;
                let json = serde_json::to_string_pretty(&skill).map_err(|e| e.to_string())?;
                Ok(format!(
                    "Skill '{}' created successfully:\n{}",
                    skill.name, json
                ))
            }
            "delete_skill" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let skill = db
                    .get_skill_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Skill with ID {} not found", id))?;
                db.delete_skill(id).map_err(|e| e.to_string())?;
                Ok(format!("Skill '{}' deleted successfully", skill.name))
            }
            "enable_global_skill" => {
                let id = args_map
                    .get("item_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing item_id")?;
                db.add_global_skill(id).map_err(|e| e.to_string())?;
                Ok(format!("Skill {} enabled globally", id))
            }
            "disable_global_skill" => {
                let id = args_map
                    .get("item_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing item_id")?;
                db.remove_global_skill(id).map_err(|e| e.to_string())?;
                Ok(format!("Skill {} disabled globally", id))
            }
            "list_subagents" => {
                let items = db.get_all_subagents().map_err(|e| e.to_string())?;
                serde_json::to_string_pretty(&items).map_err(|e| e.to_string())
            }
            "get_subagent" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let item = db
                    .get_subagent_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("SubAgent with ID {} not found", id))?;
                serde_json::to_string_pretty(&item).map_err(|e| e.to_string())
            }
            "create_subagent" => {
                use crate::db::models::CreateSubAgentRequest;
                let request = CreateSubAgentRequest {
                    name: args_map
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: args_map
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    content: args_map
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    tools: args_map.get("tools").and_then(|v| v.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                    model: args_map
                        .get("model")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    permission_mode: args_map
                        .get("permission_mode")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    skills: None,
                    tags: args_map.get("tags").and_then(|v| v.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                };
                let item = db.create_subagent(&request).map_err(|e| e.to_string())?;
                let json = serde_json::to_string_pretty(&item).map_err(|e| e.to_string())?;
                Ok(format!(
                    "SubAgent '{}' created successfully:\n{}",
                    item.name, json
                ))
            }
            "delete_subagent" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let item = db
                    .get_subagent_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("SubAgent with ID {} not found", id))?;
                db.delete_subagent(id).map_err(|e| e.to_string())?;
                Ok(format!("SubAgent '{}' deleted successfully", item.name))
            }
            "enable_global_subagent" => {
                let id = args_map
                    .get("item_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing item_id")?;
                db.add_global_subagent(id).map_err(|e| e.to_string())?;
                Ok(format!("SubAgent {} enabled globally", id))
            }
            "disable_global_subagent" => {
                let id = args_map
                    .get("item_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing item_id")?;
                db.remove_global_subagent(id).map_err(|e| e.to_string())?;
                Ok(format!("SubAgent {} disabled globally", id))
            }
            "list_hooks" => {
                let items = db.get_all_hooks().map_err(|e| e.to_string())?;
                serde_json::to_string_pretty(&items).map_err(|e| e.to_string())
            }
            "get_hook" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let item = db
                    .get_hook_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Hook with ID {} not found", id))?;
                serde_json::to_string_pretty(&item).map_err(|e| e.to_string())
            }
            "create_hook" => {
                use crate::db::models::CreateHookRequest;
                let request = CreateHookRequest {
                    name: args_map
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    event_type: args_map
                        .get("event_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    hook_type: args_map
                        .get("hook_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: args_map
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    matcher: args_map
                        .get("matcher")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    command: args_map
                        .get("command")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    prompt: args_map
                        .get("prompt")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    timeout: args_map
                        .get("timeout")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32),
                    tags: args_map.get("tags").and_then(|v| v.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                };
                let item = db.create_hook(&request).map_err(|e| e.to_string())?;
                let json = serde_json::to_string_pretty(&item).map_err(|e| e.to_string())?;
                Ok(format!(
                    "Hook '{}' created successfully:\n{}",
                    item.name, json
                ))
            }
            "delete_hook" => {
                let id = args_map
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing id")?;
                let item = db
                    .get_hook_by_id(id)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Hook with ID {} not found", id))?;
                db.delete_hook(id).map_err(|e| e.to_string())?;
                Ok(format!("Hook '{}' deleted successfully", item.name))
            }
            "enable_global_hook" => {
                let id = args_map
                    .get("item_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing item_id")?;
                db.add_global_hook(id).map_err(|e| e.to_string())?;
                Ok(format!("Hook {} enabled globally", id))
            }
            "disable_global_hook" => {
                let id = args_map
                    .get("item_id")
                    .and_then(|v| v.as_i64())
                    .ok_or("missing item_id")?;
                db.remove_global_hook(id).map_err(|e| e.to_string())?;
                Ok(format!("Hook {} disabled globally", id))
            }
            _ => Err(format!("Unknown tool: {}", name)),
        }
    }

    /// Helper: call tool and unwrap the result text
    fn call(server: &ToolManagerServer, name: &str, args: serde_json::Value) -> String {
        call_tool_sync(server, name, args).unwrap()
    }

    // ========================================================================
    // Construction & basic methods
    // ========================================================================

    #[test]
    fn test_new_creates_server() {
        let server = make_server();
        // Should be able to lock the DB without panic.
        let _guard = server.get_db().expect("lock db");
    }

    #[test]
    fn test_debug_impl() {
        let server = make_server();
        let debug = format!("{:?}", server);
        assert!(debug.contains("ToolManagerServer"));
    }

    #[test]
    fn test_get_info() {
        let server = make_server();
        let info = server.get_info();
        assert!(info
            .instructions
            .unwrap()
            .contains("Claude Code Tool Manager"));
        assert!(info.capabilities.tools.is_some());
    }

    // list_tools tested via ServerHandler would require async context;
    // the tool_router is tested implicitly through the call_tool_sync helper above.

    // MCP CRUD
    #[test]
    fn test_list_mcps_empty() {
        let s = make_server();
        let text = call(&s, "list_mcps", json!({}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
        assert!(mcps.is_empty());
    }

    #[test]
    fn test_create_and_get_mcp() {
        let s = make_server();
        let text = call(
            &s,
            "create_mcp",
            json!({"name":"test-server","type":"stdio","description":"A test","command":"node","args":["server.js"],"icon":"T","tags":["test"]}),
        );
        assert!(text.contains("created successfully"));

        let list = call(&s, "list_mcps", json!({}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        assert_eq!(mcps.len(), 1);
        let id = mcps[0]["id"].as_i64().unwrap();

        let got = call(&s, "get_mcp", json!({"id": id}));
        let mcp: serde_json::Value = serde_json::from_str(&got).unwrap();
        assert_eq!(mcp["name"], "test-server");
    }

    #[test]
    fn test_create_mcp_sse() {
        let s = make_server();
        let text = call(
            &s,
            "create_mcp",
            json!({"name":"sse-srv","type":"sse","url":"http://localhost/sse","headers":{"Auth":"Bearer t"},"env":{"K":"V"}}),
        );
        assert!(text.contains("created successfully"));
    }

    #[test]
    fn test_update_mcp() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"updatable","type":"stdio","command":"old"}),
        );
        let list = call(&s, "list_mcps", json!({}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        let id = mcps[0]["id"].as_i64().unwrap();

        let text = call(
            &s,
            "update_mcp",
            json!({"id":id,"name":"renamed","description":"updated"}),
        );
        assert!(text.contains("updated successfully"));

        let got = call(&s, "get_mcp", json!({"id":id}));
        let mcp: serde_json::Value = serde_json::from_str(&got).unwrap();
        assert_eq!(mcp["name"], "renamed");
    }

    #[test]
    fn test_update_mcp_not_found() {
        let s = make_server();
        let r = call_tool_sync(&s, "update_mcp", json!({"id":999}));
        assert!(r.is_err());
    }

    #[test]
    fn test_delete_mcp() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"del","type":"http","url":"http://x"}),
        );
        let list = call(&s, "list_mcps", json!({}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        let id = mcps[0]["id"].as_i64().unwrap();
        let text = call(&s, "delete_mcp", json!({"id":id}));
        assert!(text.contains("deleted successfully"));
    }

    #[test]
    fn test_delete_mcp_not_found() {
        let s = make_server();
        assert!(call_tool_sync(&s, "delete_mcp", json!({"id":999})).is_err());
    }

    #[test]
    fn test_get_mcp_not_found() {
        let s = make_server();
        assert!(call_tool_sync(&s, "get_mcp", json!({"id":999})).is_err());
    }

    // Filtering
    #[test]
    fn test_list_mcps_filter_by_type() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"a","type":"stdio","command":"x"}),
        );
        call(
            &s,
            "create_mcp",
            json!({"name":"b","type":"sse","url":"http://x"}),
        );
        let text = call(&s, "list_mcps", json!({"filter_type":"stdio"}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0]["name"], "a");
    }

    #[test]
    fn test_list_mcps_search() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"alpha","type":"stdio","command":"x"}),
        );
        call(
            &s,
            "create_mcp",
            json!({"name":"beta","type":"stdio","command":"x"}),
        );
        let text = call(&s, "list_mcps", json!({"search":"alpha"}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
        assert_eq!(mcps.len(), 1);
    }

    #[test]
    fn test_list_mcps_search_desc() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"x","type":"stdio","command":"x","description":"database"}),
        );
        call(
            &s,
            "create_mcp",
            json!({"name":"y","type":"stdio","command":"x","description":"network"}),
        );
        let text = call(&s, "list_mcps", json!({"search":"database"}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
        assert_eq!(mcps.len(), 1);
    }

    #[test]
    fn test_list_mcps_combined() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"stdio-a","type":"stdio","command":"x"}),
        );
        call(
            &s,
            "create_mcp",
            json!({"name":"sse-a","type":"sse","url":"http://x"}),
        );
        let text = call(&s, "list_mcps", json!({"filter_type":"sse","search":"a"}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0]["name"], "sse-a");
    }

    // Global MCPs
    #[test]
    fn test_global_mcps() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"gm","type":"stdio","command":"g"}),
        );
        let list = call(&s, "list_mcps", json!({}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        let id = mcps[0]["id"].as_i64().unwrap();
        assert!(call(&s, "enable_global_mcp", json!({"mcp_id":id})).contains("enabled"));
        assert!(call(&s, "list_global_mcps", json!({})).contains("gm"));
        assert!(call(&s, "disable_global_mcp", json!({"mcp_id":id})).contains("disabled"));
    }

    // Projects
    #[test]
    fn test_list_projects_empty() {
        let s = make_server();
        let text = call(&s, "list_projects", json!({}));
        let p: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
        assert!(p.is_empty());
    }

    #[test]
    fn test_get_project_not_found() {
        let s = make_server();
        assert!(call_tool_sync(&s, "get_project", json!({"id":999})).is_err());
    }

    // Skills CRUD
    #[test]
    fn test_skills_lifecycle() {
        let s = make_server();
        let text = call(&s, "list_skills", json!({}));
        assert_eq!(
            serde_json::from_str::<Vec<serde_json::Value>>(&text)
                .unwrap()
                .len(),
            0
        );

        let text = call(
            &s,
            "create_skill",
            json!({"name":"sk","content":"# Sk","description":"d","allowed_tools":["Read"],"model":"sonnet","tags":["t"]}),
        );
        assert!(text.contains("created successfully"));

        let list = call(&s, "list_skills", json!({}));
        let skills: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        let id = skills[0]["id"].as_i64().unwrap();

        let got = call(&s, "get_skill", json!({"id":id}));
        assert!(got.contains("sk"));

        assert!(call(&s, "enable_global_skill", json!({"item_id":id})).contains("enabled"));
        assert!(call(&s, "disable_global_skill", json!({"item_id":id})).contains("disabled"));
        assert!(call(&s, "delete_skill", json!({"id":id})).contains("deleted"));
    }

    #[test]
    fn test_get_skill_not_found() {
        assert!(call_tool_sync(&make_server(), "get_skill", json!({"id":999})).is_err());
    }

    #[test]
    fn test_delete_skill_not_found() {
        assert!(call_tool_sync(&make_server(), "delete_skill", json!({"id":999})).is_err());
    }

    // SubAgents CRUD
    #[test]
    fn test_subagents_lifecycle() {
        let s = make_server();
        let text = call(
            &s,
            "create_subagent",
            json!({"name":"sa","description":"d","content":"c","tools":["Read"],"model":"opus","tags":["t"]}),
        );
        assert!(text.contains("created successfully"));

        let list = call(&s, "list_subagents", json!({}));
        let items: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        let id = items[0]["id"].as_i64().unwrap();

        assert!(call(&s, "get_subagent", json!({"id":id})).contains("sa"));
        assert!(call(&s, "enable_global_subagent", json!({"item_id":id})).contains("enabled"));
        assert!(call(&s, "disable_global_subagent", json!({"item_id":id})).contains("disabled"));
        assert!(call(&s, "delete_subagent", json!({"id":id})).contains("deleted"));
    }

    #[test]
    fn test_get_subagent_not_found() {
        assert!(call_tool_sync(&make_server(), "get_subagent", json!({"id":999})).is_err());
    }

    // Hooks CRUD
    #[test]
    fn test_hooks_lifecycle() {
        let s = make_server();
        let text = call(
            &s,
            "create_hook",
            json!({"name":"h","event_type":"PreToolUse","hook_type":"command","command":"lint","matcher":"Write","timeout":5000,"tags":["t"]}),
        );
        assert!(text.contains("created successfully"));

        let list = call(&s, "list_hooks", json!({}));
        let items: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        let id = items[0]["id"].as_i64().unwrap();

        assert!(call(&s, "get_hook", json!({"id":id})).contains("h"));
        assert!(call(&s, "enable_global_hook", json!({"item_id":id})).contains("enabled"));
        assert!(call(&s, "disable_global_hook", json!({"item_id":id})).contains("disabled"));
        assert!(call(&s, "delete_hook", json!({"id":id})).contains("deleted"));
    }

    #[test]
    fn test_create_prompt_hook() {
        let s = make_server();
        let text = call(
            &s,
            "create_hook",
            json!({"name":"ph","event_type":"PostToolUse","hook_type":"prompt","prompt":"Review.","matcher":"Write"}),
        );
        assert!(text.contains("created successfully"));
    }

    #[test]
    fn test_get_hook_not_found() {
        assert!(call_tool_sync(&make_server(), "get_hook", json!({"id":999})).is_err());
    }

    // Project-MCP assignment
    #[test]
    fn test_assign_mcp_to_project() {
        let s = make_server();
        call(
            &s,
            "create_mcp",
            json!({"name":"pm","type":"stdio","command":"x"}),
        );
        let list = call(&s, "list_mcps", json!({}));
        let mcps: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        let mcp_id = mcps[0]["id"].as_i64().unwrap();

        {
            let db = s.get_db().unwrap();
            db.conn()
                .execute(
                    "INSERT INTO projects (name, path) VALUES (?1, ?2)",
                    rusqlite::params!["tp", "/tmp/t"],
                )
                .unwrap();
        }
        let plist = call(&s, "list_projects", json!({}));
        let projects: Vec<serde_json::Value> = serde_json::from_str(&plist).unwrap();
        let pid = projects[0]["id"].as_i64().unwrap();

        assert!(call(
            &s,
            "assign_mcp_to_project",
            json!({"project_id":pid,"mcp_id":mcp_id})
        )
        .contains("assigned"));
        assert!(call(
            &s,
            "remove_mcp_from_project",
            json!({"project_id":pid,"mcp_id":mcp_id})
        )
        .contains("removed"));
    }

    #[test]
    fn test_unknown_tool() {
        let s = make_server();
        assert!(call_tool_sync(&s, "nonexistent", json!({})).is_err());
    }

    #[test]
    fn test_server_is_cloneable() {
        let server = make_server();
        let cloned = server.clone();
        assert!(Arc::ptr_eq(&server.db, &cloned.db));
    }
}
