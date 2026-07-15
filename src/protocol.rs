//! MCP protocol types and message handling.
//!
//! Implements the Model Context Protocol (MCP) as specified at:
//! https://spec.modelcontextprotocol.io/

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (always "2.0").
    pub jsonrpc: String,
    /// Request ID.
    pub id: Option<JsonRpcId>,
    /// Method name.
    pub method: String,
    /// Parameters (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version (always "2.0").
    pub jsonrpc: String,
    /// Request ID (matches request).
    pub id: Option<JsonRpcId>,
    /// Result (success case).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (failure case).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: Option<JsonRpcId>, result: impl Serialize) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<JsonRpcId>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    /// Create an error response with data.
    pub fn error_with_data(
        id: Option<JsonRpcId>,
        code: i32,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
        }
    }
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC request ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// String ID.
    String(String),
    /// Numeric ID.
    Number(i64),
}

/// MCP message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpMessage {
    /// Request message.
    Request(JsonRpcRequest),
    /// Response message.
    Response(JsonRpcResponse),
    /// Notification (no ID, no response expected).
    Notification(JsonRpcRequest),
}

impl McpMessage {
    /// Parse a JSON string into an MCP message.
    pub fn parse(json: &str) -> crate::Result<Self> {
        let value: serde_json::Value = serde_json::from_str(json)?;

        // Check if it's a request or response
        if value.get("method").is_some() {
            let request: JsonRpcRequest = serde_json::from_value(value)?;
            if request.id.is_some() {
                Ok(McpMessage::Request(request))
            } else {
                Ok(McpMessage::Notification(request))
            }
        } else if value.get("result").is_some() || value.get("error").is_some() {
            let response: JsonRpcResponse = serde_json::from_value(value)?;
            Ok(McpMessage::Response(response))
        } else {
            Err(crate::Error::InvalidParams("invalid MCP message".into()))
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

// ============================================================================
// MCP-specific protocol types
// ============================================================================

/// MCP initialization request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client info.
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// MCP initialization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server info.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Client capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Roots capability.
    #[serde(default)]
    pub roots: Option<RootsCapability>,
    /// Sampling capability.
    #[serde(default)]
    pub sampling: Option<serde_json::Value>,
}

/// Roots capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
    /// Whether list changed notifications are supported.
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

/// Server capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Resources capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Prompts capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    /// Logging capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<serde_json::Value>,
}

/// Tools capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether tool list changed notifications are supported.
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

/// Resources capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether subscription is supported.
    #[serde(default)]
    pub subscribe: bool,
    /// Whether list changed notifications are supported.
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

/// Prompts capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether list changed notifications are supported.
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

/// Client information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name.
    pub name: String,
    /// Client version.
    pub version: String,
}

/// Server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    pub version: String,
}

/// Tool definition for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// Input schema (JSON Schema).
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Tool call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    /// Tool name.
    pub name: String,
    /// Tool arguments.
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// Tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// Result content.
    pub content: Vec<ContentItem>,
    /// Whether the tool encountered an error.
    #[serde(rename = "isError", default)]
    pub is_error: bool,
    /// Structured metadata for machine-readable tool outputs (e.g. session_id).
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl ToolCallResult {
    /// Create a successful text result.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem::text(content)],
            is_error: false,
            meta: None,
        }
    }
}

/// Content item in tool results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    /// Text content.
    #[serde(rename = "text")]
    Text {
        /// Text value.
        text: String,
    },
    /// Image content.
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data.
        data: String,
        /// MIME type.
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Resource content.
    #[serde(rename = "resource")]
    Resource {
        /// Resource URI.
        uri: String,
        /// MIME type.
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        /// Resource text.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
}

impl ContentItem {
    /// Create a text content item.
    pub fn text(text: impl Into<String>) -> Self {
        ContentItem::Text { text: text.into() }
    }

    /// Create an image content item.
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        ContentItem::Image {
            data: data.into(),
            mime_type: mime_type.into(),
        }
    }
}

/// List tools result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// Available tools.
    pub tools: Vec<ToolDefinition>,
}
