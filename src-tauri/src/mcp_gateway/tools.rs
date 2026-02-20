//! MCP Gateway Server Handler
//!
//! Implements the rmcp ServerHandler trait with lazy-loading meta-tools.
//! Instead of exposing all backend tools upfront, exposes 3 meta-tools:
//! - list_available_mcps: Discover available MCP servers
//! - load_mcp_tools: Connect to an MCP and get its tools
//! - call_mcp_tool: Execute a tool on a specific MCP

use crate::mcp_gateway::backend::GatewayBackendManager;
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, Content, ListToolsResult, PaginatedRequestParams,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    ErrorData, RoleServer, ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

/// Arguments for load_mcp_tools meta-tool
#[derive(Debug, Deserialize)]
struct LoadMcpToolsArgs {
    mcp_name: String,
}

/// Arguments for call_mcp_tool meta-tool
#[derive(Debug, Deserialize)]
struct CallMcpToolArgs {
    mcp_name: String,
    tool_name: String,
    #[serde(default)]
    arguments: Value,
}

/// Tool information returned by load_mcp_tools
#[derive(Debug, Serialize)]
struct ToolInfo {
    name: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_schema: Option<Value>,
}

/// The Gateway MCP Server handler
///
/// Uses lazy-loading meta-tools instead of exposing all backend tools upfront.
pub struct GatewayServer {
    backend_manager: Arc<tokio::sync::Mutex<GatewayBackendManager>>,
}

impl std::fmt::Debug for GatewayServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayServer").finish()
    }
}

impl Clone for GatewayServer {
    fn clone(&self) -> Self {
        Self {
            backend_manager: self.backend_manager.clone(),
        }
    }
}

impl GatewayServer {
    pub fn new(backend_manager: Arc<tokio::sync::Mutex<GatewayBackendManager>>) -> Self {
        Self { backend_manager }
    }
}

impl ServerHandler for GatewayServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "MCP Gateway with lazy-loading. Use these 3 meta-tools:\n\
                1. list_available_mcps - Discover available MCP servers\n\
                2. load_mcp_tools - Connect to an MCP and get its tools\n\
                3. call_mcp_tool - Execute a tool on a specific MCP\n\n\
                Flow: First call list_available_mcps to see what's available, \
                then call load_mcp_tools to connect and see tools, \
                then call call_mcp_tool to execute tools."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move {
            // Return only the 3 meta-tools
            let meta_tools = vec![
                Tool {
                    name: "list_available_mcps".into(),
                    title: None,
                    description: Some(
                        "List all MCP servers available through this gateway. \
                        Call this first to discover what MCPs you can use."
                            .into(),
                    ),
                    input_schema: Arc::new(serde_json::Map::from_iter([
                        ("type".to_string(), json!("object")),
                        ("properties".to_string(), json!({})),
                        ("required".to_string(), json!([])),
                    ])),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: None,
                    execution: None,
                },
                Tool {
                    name: "load_mcp_tools".into(),
                    title: None,
                    description: Some(
                        "Load and return all tools from a specific MCP server. \
                        The MCP will be connected if not already. \
                        Call this after list_available_mcps to see what tools an MCP offers."
                            .into(),
                    ),
                    input_schema: Arc::new(serde_json::Map::from_iter([
                        ("type".to_string(), json!("object")),
                        (
                            "properties".to_string(),
                            json!({
                                "mcp_name": {
                                    "type": "string",
                                    "description": "Name of the MCP to load tools from"
                                }
                            }),
                        ),
                        ("required".to_string(), json!(["mcp_name"])),
                    ])),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: None,
                    execution: None,
                },
                Tool {
                    name: "call_mcp_tool".into(),
                    title: None,
                    description: Some(
                        "Execute a tool on a specific MCP server. \
                        The MCP must be connected first via load_mcp_tools."
                            .into(),
                    ),
                    input_schema: Arc::new(serde_json::Map::from_iter([
                        ("type".to_string(), json!("object")),
                        (
                            "properties".to_string(),
                            json!({
                                "mcp_name": {
                                    "type": "string",
                                    "description": "Name of the MCP containing the tool"
                                },
                                "tool_name": {
                                    "type": "string",
                                    "description": "Name of the tool to call"
                                },
                                "arguments": {
                                    "type": "object",
                                    "description": "Arguments to pass to the tool",
                                    "default": {}
                                }
                            }),
                        ),
                        ("required".to_string(), json!(["mcp_name", "tool_name"])),
                    ])),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: None,
                    execution: None,
                },
            ];

            log::info!("[Gateway] Listing 3 meta-tools (lazy mode)");

            Ok(ListToolsResult {
                tools: meta_tools,
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        async move {
            let tool_name = request.name.as_ref();
            let arguments = request.arguments.unwrap_or_default();

            log::info!("[Gateway] Meta-tool call: {}", tool_name);

            match tool_name {
                "list_available_mcps" => {
                    let backend_manager = self.backend_manager.lock().await;
                    let mcps = backend_manager.get_available_mcps();
                    let result = serde_json::to_string_pretty(&mcps)
                        .unwrap_or_else(|e| format!("Error serializing MCPs: {}", e));
                    Ok(CallToolResult::success(vec![Content::text(result)]))
                }

                "load_mcp_tools" => {
                    let args: LoadMcpToolsArgs = serde_json::from_value(Value::Object(arguments))
                        .map_err(|e| {
                        log::error!("[Gateway] Invalid load_mcp_tools arguments: {}", e);
                        ErrorData::invalid_params(format!("Invalid arguments: {}", e), None)
                    })?;

                    let mut backend_manager = self.backend_manager.lock().await;
                    match backend_manager.connect_backend_lazy(&args.mcp_name).await {
                        Ok(tools) => {
                            let tool_infos: Vec<ToolInfo> = tools
                                .into_iter()
                                .map(|t| ToolInfo {
                                    name: t.name,
                                    description: t.description,
                                    input_schema: t.input_schema,
                                })
                                .collect();
                            let result = serde_json::to_string_pretty(&tool_infos)
                                .unwrap_or_else(|e| format!("Error serializing tools: {}", e));
                            Ok(CallToolResult::success(vec![Content::text(result)]))
                        }
                        Err(e) => {
                            log::error!("[Gateway] Failed to load MCP tools: {}", e);
                            Ok(CallToolResult::error(vec![Content::text(format!(
                                "Error: {}",
                                e
                            ))]))
                        }
                    }
                }

                "call_mcp_tool" => {
                    let args: CallMcpToolArgs = serde_json::from_value(Value::Object(arguments))
                        .map_err(|e| {
                            log::error!("[Gateway] Invalid call_mcp_tool arguments: {}", e);
                            ErrorData::invalid_params(format!("Invalid arguments: {}", e), None)
                        })?;

                    let mut backend_manager = self.backend_manager.lock().await;
                    match backend_manager.call_tool_on_mcp(
                        &args.mcp_name,
                        &args.tool_name,
                        args.arguments,
                    ) {
                        Ok(result) => {
                            // Convert our ToolCallResult to rmcp's CallToolResult
                            let content: Vec<Content> = result
                                .content
                                .into_iter()
                                .map(|c| match c {
                                    crate::services::mcp_client::ToolContent::Text { text } => {
                                        Content::text(text)
                                    }
                                    crate::services::mcp_client::ToolContent::Image {
                                        data,
                                        mime_type,
                                    } => Content::image(data, mime_type),
                                    crate::services::mcp_client::ToolContent::Resource {
                                        uri,
                                        text,
                                        ..
                                    } => {
                                        use rmcp::model::ResourceContents;
                                        Content::resource(ResourceContents::text(
                                            text.unwrap_or_default(),
                                            uri,
                                        ))
                                    }
                                })
                                .collect();

                            if result.is_error {
                                Ok(CallToolResult::error(content))
                            } else {
                                Ok(CallToolResult::success(content))
                            }
                        }
                        Err(e) => {
                            log::error!("[Gateway] Tool call failed: {}", e);
                            Ok(CallToolResult::error(vec![Content::text(format!(
                                "Error: {}",
                                e
                            ))]))
                        }
                    }
                }

                _ => {
                    log::warn!("[Gateway] Unknown meta-tool: {}", tool_name);
                    Ok(CallToolResult::error(vec![Content::text(format!(
                        "Unknown tool: {}. Available meta-tools: list_available_mcps, load_mcp_tools, call_mcp_tool",
                        tool_name
                    ))]))
                }
            }
        }
    }
}
