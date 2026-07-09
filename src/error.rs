//! Error types for the MCP server.

use thiserror::Error;

/// Result type alias.
pub type Result<T> = std::result::Result<T, Error>;

/// MCP server errors.
#[derive(Error, Debug)]
pub enum Error {
    /// JSON-RPC protocol error.
    #[error("JSON-RPC error: {code} - {message}")]
    JsonRpc {
        /// Error code.
        code: i32,
        /// Error message.
        message: String,
        /// Additional data.
        data: Option<serde_json::Value>,
    },

    /// Tool not found.
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// Invalid parameters.
    #[error("invalid parameters: {0}")]
    InvalidParams(String),

    /// Permission denied by guardrails.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Webpuppet error.
    #[error("webpuppet error: {0}")]
    Webpuppet(#[from] webpuppet::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Get the JSON-RPC error code for this error.
    pub fn code(&self) -> i32 {
        match self {
            Error::JsonRpc { code, .. } => *code,
            Error::ToolNotFound(_) => -32601,  // Method not found
            Error::InvalidParams(_) => -32602, // Invalid params
            Error::PermissionDenied(_) => -32000, // Server error
            Error::Webpuppet(_) => -32001,
            Error::Serialization(_) => -32700, // Parse error
            Error::Io(_) => -32002,
            Error::Internal(_) => -32603, // Internal error
        }
    }

    /// Convert to JSON-RPC error response.
    pub fn to_json_rpc_error(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.code(),
            "message": self.to_string(),
        })
    }
}

/// Standard JSON-RPC error codes.
pub mod codes {
    /// Parse error.
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request.
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params.
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error.
    pub const INTERNAL_ERROR: i32 = -32603;
}
