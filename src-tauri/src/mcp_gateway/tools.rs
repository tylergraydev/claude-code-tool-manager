//! MCP Gateway Server Handler
//!
//! Implements the rmcp ServerHandler trait to proxy tools from multiple backend MCPs.

use crate::mcp_gateway::backend::GatewayBackendManager;
use rmcp::{
    ErrorData,
    model::{
        CallToolResult, ListToolsResult, PaginatedRequestParam, ServerCapabilities, ServerInfo,
        Tool, CallToolRequestParam, Content,
    },
    service::RequestContext, RoleServer, ServerHandler,
};
use serde_json::Value;
use std::sync::Arc;

/// The Gateway MCP Server handler
///
/// This handler aggregates tools from multiple backend MCP servers and proxies
/// tool calls to the appropriate backend.
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
                "MCP Gateway - Aggregates tools from multiple MCP servers into a single endpoint. \
                Tool names are prefixed with their source MCP name (e.g., 'filesystem__read_file')."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move {
            let backend_manager = self.backend_manager.lock().await;
            let tools = backend_manager.get_tools();

            // Convert our McpTool type to rmcp's Tool type
            let rmcp_tools: Vec<Tool> = tools.into_iter().map(|t| {
                // Build input schema - rmcp expects Arc<Map<String, Value>>
                let input_schema = t.input_schema
                    .and_then(|v| v.as_object().cloned())
                    .map(|m| Arc::new(m))
                    .unwrap_or_else(|| Arc::new(serde_json::Map::new()));

                Tool {
                    name: t.name.into(),
                    title: None,
                    description: t.description.map(|d| d.into()),
                    input_schema,
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: None,
                }
            }).collect();

            log::info!("[Gateway] Listing {} tools", rmcp_tools.len());

            Ok(ListToolsResult {
                tools: rmcp_tools,
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        async move {
            let tool_name = request.name.as_ref();
            let arguments = request.arguments.unwrap_or_default();

            log::info!("[Gateway] Calling tool: {}", tool_name);

            // Lock the backend manager and call the tool
            let mut backend_manager = self.backend_manager.lock().await;

            // Convert arguments from Map to Value
            let args_value = Value::Object(arguments);

            match backend_manager.call_tool(tool_name, args_value) {
                Ok(result) => {
                    // Convert our ToolCallResult to rmcp's CallToolResult
                    let content: Vec<Content> = result.content.into_iter().map(|c| {
                        match c {
                            crate::services::mcp_client::ToolContent::Text { text } => {
                                Content::text(text)
                            }
                            crate::services::mcp_client::ToolContent::Image { data, mime_type } => {
                                Content::image(data, mime_type)
                            }
                            crate::services::mcp_client::ToolContent::Resource { uri, text, .. } => {
                                // Create resource content (mime_type not supported in this method)
                                use rmcp::model::ResourceContents;
                                Content::resource(ResourceContents::text(text.unwrap_or_default(), uri))
                            }
                        }
                    }).collect();

                    if result.is_error {
                        Ok(CallToolResult::error(content))
                    } else {
                        Ok(CallToolResult::success(content))
                    }
                }
                Err(e) => {
                    log::error!("[Gateway] Tool call failed: {}", e);
                    Ok(CallToolResult::error(vec![Content::text(format!("Error: {}", e))]))
                }
            }
        }
    }
}
