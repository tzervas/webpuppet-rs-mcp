//! MCP server implementation.

use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;

use webpuppet::PermissionGuard;

use crate::error::{codes, Result};
use crate::protocol::{
    ClientCapabilities, InitializeParams, InitializeResult, JsonRpcId, JsonRpcRequest,
    JsonRpcResponse, ListToolsResult, McpMessage, ServerCapabilities, ServerInfo, ToolCallParams,
    ToolsCapability,
};
use crate::tools::ToolRegistry;

/// Methods that must be handled in-order, inline with the read loop.
///
/// `initialize` / `shutdown` / `exit` mutate [`ServerState`], and every other request is
/// rejected until `initialize` has completed. Dispatching them concurrently would let a
/// pipelined client's first tool call race the handshake and be rejected with
/// "server not initialized". Notifications are ordered for the same reason and are cheap.
fn must_handle_inline(line: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
        // Unparseable: handle inline so the parse-error response keeps its position.
        return true;
    };
    match value.get("method").and_then(|m| m.as_str()) {
        Some(method) => {
            method == "initialize"
                || method == "shutdown"
                || method == "exit"
                || method.starts_with("notifications/")
        }
        // A response, or a request with no method: cheap, keep it ordered.
        None => true,
    }
}

/// Write one JSON-RPC response as a single line, serialised against other writers.
async fn write_response(stdout: &Mutex<tokio::io::Stdout>, response: &JsonRpcResponse) {
    let json = match serde_json::to_string(response) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Failed to serialize response: {}", e);
            return;
        }
    };
    tracing::debug!("Sending: {}", json);

    // Build the full line first so one response is exactly one write.
    let mut buf = json.into_bytes();
    buf.push(b'\n');

    let mut out = stdout.lock().await;
    if let Err(e) = out.write_all(&buf).await {
        tracing::error!("Failed to write response: {}", e);
    } else if let Err(e) = out.flush().await {
        tracing::error!("Failed to flush response: {}", e);
    }
}

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
///
/// Cheap to clone: every field is behind an `Arc`, so clones share one server.
/// `run_stdio` relies on this to hand a handle to each concurrently dispatched request.
#[derive(Clone)]
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
    ///
    /// Requests are dispatched **concurrently**. This is load-bearing, not an
    /// optimisation: the previous implementation awaited each request to completion
    /// before reading the next line, so any tool call that waited on a human (a HITL
    /// pause) deadlocked the process — the `webpuppet_resume` /
    /// `webpuppet_intervention_complete` call that would release the pause was sitting
    /// unread in stdin behind the very call that was blocking on it.
    ///
    /// Responses may therefore be written out of order. That is explicitly allowed by
    /// JSON-RPC 2.0: clients correlate by `id`. Writes are serialised by a mutex so
    /// individual response lines are never interleaved.
    pub async fn run_stdio(&self) -> Result<()> {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        let stdout = Arc::new(Mutex::new(tokio::io::stdout()));
        let mut inflight: JoinSet<()> = JoinSet::new();

        tracing::info!("MCP server starting on stdio");

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            tracing::debug!("Received: {}", line);

            if must_handle_inline(&line) {
                if let Some(response) = self.handle_message(&line).await {
                    write_response(&stdout, &response).await;
                }
            } else {
                let server = self.clone();
                let stdout = Arc::clone(&stdout);

                inflight.spawn(async move {
                    if let Some(response) = server.handle_message(&line).await {
                        write_response(&stdout, &response).await;
                    }
                });
            }

            // Reap completed handlers so the JoinSet does not grow without bound.
            while let Some(joined) = inflight.try_join_next() {
                if let Err(e) = joined {
                    tracing::error!("Request handler task failed: {}", e);
                }
            }

            // Check if we should exit
            if *self.state.read().await == ServerState::ShuttingDown {
                break;
            }
        }

        // Let in-flight handlers finish writing before we tear down the browsers.
        while let Some(joined) = inflight.join_next().await {
            if let Err(e) = joined {
                tracing::error!("Request handler task failed: {}", e);
            }
        }

        tracing::info!("MCP server shutting down");
        self.tools.shutdown().await;
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
