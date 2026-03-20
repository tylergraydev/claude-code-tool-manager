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
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "MCP Gateway with lazy-loading. Use these 3 meta-tools:\n\
                1. list_available_mcps - Discover available MCP servers\n\
                2. load_mcp_tools - Connect to an MCP and get its tools\n\
                3. call_mcp_tool - Execute a tool on a specific MCP\n\n\
                Flow: First call list_available_mcps to see what's available, \
                then call load_mcp_tools to connect and see tools, \
                then call call_mcp_tool to execute tools."
                .to_string(),
        )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move {
            // Return only the 3 meta-tools
            let meta_tools = vec![
                Tool::new(
                    "list_available_mcps",
                    "List all MCP servers available through this gateway. \
                    Call this first to discover what MCPs you can use.",
                    Arc::new(serde_json::Map::from_iter([
                        ("type".to_string(), json!("object")),
                        ("properties".to_string(), json!({})),
                        ("required".to_string(), json!([])),
                    ])),
                ),
                Tool::new(
                    "load_mcp_tools",
                    "Load and return all tools from a specific MCP server. \
                    The MCP will be connected if not already. \
                    Call this after list_available_mcps to see what tools an MCP offers.",
                    Arc::new(serde_json::Map::from_iter([
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
                ),
                Tool::new(
                    "call_mcp_tool",
                    "Execute a tool on a specific MCP server. \
                    The MCP must be connected first via load_mcp_tools.",
                    Arc::new(serde_json::Map::from_iter([
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
                ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::mcp_gateway::backend::GatewayBackendManager;
    use std::sync::Mutex;

    fn make_test_gateway_server() -> GatewayServer {
        let db = Database::in_memory().unwrap();
        let db_arc = Arc::new(Mutex::new(db));
        let manager = GatewayBackendManager::new(db_arc);
        let backend_manager = Arc::new(tokio::sync::Mutex::new(manager));
        GatewayServer::new(backend_manager)
    }

    #[test]
    fn test_gateway_server_new() {
        let server = make_test_gateway_server();
        // Just verify it constructs without panicking
        assert!(format!("{:?}", server).contains("GatewayServer"));
    }

    #[test]
    fn test_gateway_server_clone() {
        let server = make_test_gateway_server();
        let cloned = server.clone();
        // Both should have the same debug representation
        assert_eq!(format!("{:?}", server), format!("{:?}", cloned));
    }

    #[test]
    fn test_gateway_server_debug() {
        let server = make_test_gateway_server();
        let debug_str = format!("{:?}", server);
        assert_eq!(debug_str, "GatewayServer");
    }

    #[test]
    fn test_gateway_server_get_info() {
        let server = make_test_gateway_server();
        let info = server.get_info();

        assert!(info.instructions.is_some());
        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("list_available_mcps"));
        assert!(instructions.contains("load_mcp_tools"));
        assert!(instructions.contains("call_mcp_tool"));
        assert!(instructions.contains("lazy-loading"));
    }

    #[test]
    fn test_gateway_server_get_info_has_tool_capabilities() {
        let server = make_test_gateway_server();
        let info = server.get_info();
        // Capabilities should have tools enabled
        assert!(info.capabilities.tools.is_some());
    }

    // ===== Internal struct tests =====

    #[test]
    fn test_load_mcp_tools_args_deserialize() {
        let json = r#"{"mcp_name": "test-server"}"#;
        let args: LoadMcpToolsArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.mcp_name, "test-server");
    }

    #[test]
    fn test_call_mcp_tool_args_deserialize() {
        let json = r#"{"mcp_name": "server", "tool_name": "read", "arguments": {"path": "/tmp"}}"#;
        let args: CallMcpToolArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.mcp_name, "server");
        assert_eq!(args.tool_name, "read");
        assert_eq!(args.arguments, json!({"path": "/tmp"}));
    }

    #[test]
    fn test_call_mcp_tool_args_default_arguments() {
        let json = r#"{"mcp_name": "server", "tool_name": "ping"}"#;
        let args: CallMcpToolArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.arguments, Value::Null);
    }

    #[test]
    fn test_tool_info_serialize() {
        let info = ToolInfo {
            name: "read_file".to_string(),
            description: Some("Read a file".to_string()),
            input_schema: Some(json!({"type": "object"})),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("read_file"));
        assert!(json.contains("Read a file"));
        assert!(json.contains("input_schema"));
    }

    #[test]
    fn test_tool_info_serialize_skip_none_schema() {
        let info = ToolInfo {
            name: "ping".to_string(),
            description: None,
            input_schema: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(!json.contains("input_schema"));
    }
}
