//! MCP server implementation.

use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;

use tokio::sync::RwLock;

use webpuppet::PermissionGuard;

use crate::error::{codes, Result};
use crate::protocol::{
    ClientCapabilities, InitializeParams, InitializeResult, JsonRpcId, JsonRpcRequest,
    JsonRpcResponse, ListToolsResult, McpMessage, ServerCapabilities, ServerInfo, ToolCallParams,
    ToolsCapability,
};
use crate::tools::ToolRegistry;

/// MCP protocol version.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// Server name.
pub const SERVER_NAME: &str = "webpuppet-mcp";

/// Server version.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// MCP server state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Waiting for initialization.
    Uninitialized,
    /// Server is initialized and ready.
    Ready,
    /// Server is shutting down.
    ShuttingDown,
}

/// MCP server for webpuppet.
pub struct McpServer {
    state: Arc<RwLock<ServerState>>,
    tools: Arc<ToolRegistry>,
    #[allow(dead_code)]
    client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
}

impl McpServer {
    /// Create a new MCP server with secure permissions.
    pub fn new() -> Self {
        Self::with_permissions(PermissionGuard::secure())
    }

    /// Create a new MCP server with custom permissions.
    pub fn with_permissions(permissions: PermissionGuard) -> Self {
        Self {
            state: Arc::new(RwLock::new(ServerState::Uninitialized)),
            tools: Arc::new(ToolRegistry::new(permissions)),
            client_capabilities: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new MCP server with visible browser (non-headless).
    pub fn with_visible_browser(permissions: PermissionGuard) -> Self {
        Self {
            state: Arc::new(RwLock::new(ServerState::Uninitialized)),
            tools: Arc::new(ToolRegistry::with_visible_browser(permissions)),
            client_capabilities: Arc::new(RwLock::new(None)),
        }
    }

    /// Run the server on stdio.
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        tracing::info!("MCP server starting on stdio");

        for line in reader.lines() {
            let line = line?;

            if line.is_empty() {
                continue;
            }

            tracing::debug!("Received: {}", line);

            let response = self.handle_message(&line).await;

            if let Some(response) = response {
                let json = serde_json::to_string(&response)?;
                tracing::debug!("Sending: {}", json);
                writeln!(stdout, "{}", json)?;
                stdout.flush()?;
            }

            // Check if we should exit
            if *self.state.read().await == ServerState::ShuttingDown {
                break;
            }
        }

        tracing::info!("MCP server shutting down");
        Ok(())
    }

    /// Handle an incoming message.
    pub async fn handle_message(&self, json: &str) -> Option<JsonRpcResponse> {
        match McpMessage::parse(json) {
            Ok(McpMessage::Request(request)) => Some(self.handle_request(request).await),
            Ok(McpMessage::Notification(notification)) => {
                self.handle_notification(notification).await;
                None
            }
            Ok(McpMessage::Response(_)) => {
                // We don't expect responses in this direction
                None
            }
            Err(e) => Some(JsonRpcResponse::error(
                None,
                codes::PARSE_ERROR,
                e.to_string(),
            )),
        }
    }

    /// Handle a JSON-RPC request.
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id, request.params).await,
            "tools/list" => self.handle_tools_list(id).await,
            "tools/call" => self.handle_tools_call(id, request.params).await,
            "ping" => JsonRpcResponse::success(id, serde_json::json!({})),
            "shutdown" => {
                *self.state.write().await = ServerState::ShuttingDown;
                JsonRpcResponse::success(id, serde_json::json!({}))
            }
            _ => JsonRpcResponse::error(
                id,
                codes::METHOD_NOT_FOUND,
                format!("method not found: {}", request.method),
            ),
        }
    }

    /// Handle a notification (no response expected).
    async fn handle_notification(&self, notification: JsonRpcRequest) {
        match notification.method.as_str() {
            "notifications/initialized" => {
                tracing::info!("Client initialized");
            }
            "notifications/cancelled" => {
                tracing::debug!("Request cancelled by client");
            }
            "exit" => {
                *self.state.write().await = ServerState::ShuttingDown;
            }
            _ => {
                tracing::debug!("Unknown notification: {}", notification.method);
            }
        }
    }

    /// Handle initialize request.
    async fn handle_initialize(
        &self,
        id: Option<JsonRpcId>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        // Parse params
        let _params: InitializeParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        codes::INVALID_PARAMS,
                        format!("invalid initialize params: {}", e),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    id,
                    codes::INVALID_PARAMS,
                    "initialize params required",
                );
            }
        };

        // Update state
        *self.state.write().await = ServerState::Ready;

        // Return capabilities
        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.into(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
                resources: None,
                prompts: None,
                logging: None,
            },
            server_info: ServerInfo {
                name: SERVER_NAME.into(),
                version: SERVER_VERSION.into(),
            },
        };

        JsonRpcResponse::success(id, result)
    }

    /// Handle tools/list request.
    async fn handle_tools_list(&self, id: Option<JsonRpcId>) -> JsonRpcResponse {
        let state = *self.state.read().await;
        if state != ServerState::Ready {
            return JsonRpcResponse::error(id, codes::INTERNAL_ERROR, "server not initialized");
        }

        let tools = self.tools.list_tools();
        let result = ListToolsResult { tools };

        JsonRpcResponse::success(id, result)
    }

    /// Handle tools/call request.
    async fn handle_tools_call(
        &self,
        id: Option<JsonRpcId>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let state = *self.state.read().await;
        if state != ServerState::Ready {
            return JsonRpcResponse::error(id, codes::INTERNAL_ERROR, "server not initialized");
        }

        // Parse params
        let params: ToolCallParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        codes::INVALID_PARAMS,
                        format!("invalid tool call params: {}", e),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    id,
                    codes::INVALID_PARAMS,
                    "tool call params required",
                );
            }
        };

        // Execute tool
        match self.tools.execute(&params.name, params.arguments).await {
            Ok(result) => JsonRpcResponse::success(id, result),
            Err(e) => {
                tracing::error!("Tool {} failed: {}", params.name, e);
                JsonRpcResponse::error(id, e.code(), e.to_string())
            }
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
