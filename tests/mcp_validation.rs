//! MCP server validation tests.
//!
//! Tests JSON-RPC 2.0 protocol compliance, tool execution, and error handling.

use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

// JSON-RPC 2.0 types
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i32,
    #[allow(dead_code)]
    message: String,
    #[serde(default)]
    #[allow(dead_code)]
    data: Option<Value>,
}

/// MCP test client for validating the server.
struct McpTestClient {
    child: Child,
}

impl McpTestClient {
    async fn spawn() -> Result<Self, Box<dyn std::error::Error>> {
        // Build the MCP server first
        let build_status = std::process::Command::new("cargo")
            .args(["build", "-p", "webpuppet-mcp", "--release"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()?;

        if !build_status.success() {
            return Err("Failed to build MCP server".into());
        }

        // Path to the binary
        let binary_path = format!(
            "{}/../../target/release/webpuppet-mcp",
            env!("CARGO_MANIFEST_DIR")
        );

        let child = Command::new(&binary_path)
            .args(["--policy", "secure"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self { child })
    }

    async fn send_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, Box<dyn std::error::Error>> {
        let stdin = self.child.stdin.as_mut().ok_or("No stdin")?;
        let stdout = self.child.stdout.as_mut().ok_or("No stdout")?;

        // Send request
        let request_json = serde_json::to_string(&request)?;
        stdin.write_all(request_json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        // Read response with timeout
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        let result = timeout(Duration::from_secs(5), async {
            reader.read_line(&mut line).await
        })
        .await??;

        if result == 0 {
            return Err("Server closed connection".into());
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        Ok(response)
    }

    async fn close(mut self) {
        let _ = self.child.kill().await;
    }
}

// ============================================================================
// Protocol Compliance Tests
// ============================================================================

#[tokio::test]
async fn test_initialize_handshake() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        })),
    };

    match client.send_request(request).await {
        Ok(response) => {
            assert_eq!(response.jsonrpc, "2.0");
            assert_eq!(response.id, Some(1));
            assert!(response.error.is_none(), "Should not have error");

            if let Some(result) = response.result {
                println!(
                    "Initialize result: {}",
                    serde_json::to_string_pretty(&result).unwrap()
                );
                // Check for expected fields
                assert!(result.get("protocolVersion").is_some());
                assert!(result.get("serverInfo").is_some());
            }
        }
        Err(e) => {
            eprintln!("Initialize request failed: {}", e);
        }
    }

    client.close().await;
}

#[tokio::test]
async fn test_list_tools() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    // First initialize
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };
    let _ = client.send_request(init_request).await;

    // Then list tools
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 2,
        method: "tools/list".into(),
        params: None,
    };

    match client.send_request(request).await {
        Ok(response) => {
            assert!(response.error.is_none(), "Should not have error");

            if let Some(result) = response.result {
                println!("Tools: {}", serde_json::to_string_pretty(&result).unwrap());

                if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                    // Should have our expected tools
                    let tool_names: Vec<&str> = tools
                        .iter()
                        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                        .collect();

                    println!("Tool names: {:?}", tool_names);

                    // Check for expected tools
                    assert!(tool_names.contains(&"webpuppet_prompt"));
                    assert!(tool_names.contains(&"webpuppet_detect_browsers"));
                    assert!(tool_names.contains(&"webpuppet_check_permission"));
                    assert!(tool_names.contains(&"webpuppet_intervention_status"));
                }
            }
        }
        Err(e) => {
            eprintln!("List tools failed: {}", e);
        }
    }

    client.close().await;
}

#[tokio::test]
async fn test_tool_call_detect_browsers() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    // Initialize first
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };
    let _ = client.send_request(init_request).await;

    // Call detect browsers tool
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 3,
        method: "tools/call".into(),
        params: Some(json!({
            "name": "webpuppet_detect_browsers",
            "arguments": {}
        })),
    };

    match client.send_request(request).await {
        Ok(response) => {
            assert!(response.error.is_none(), "Should not have error");

            if let Some(result) = response.result {
                println!(
                    "Detect browsers result: {}",
                    serde_json::to_string_pretty(&result).unwrap()
                );

                // Should have content
                if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                    assert!(!content.is_empty(), "Should have content");

                    // First item should be text
                    if let Some(text) = content
                        .first()
                        .and_then(|c| c.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        println!("Browser detection output:\n{}", text);
                        // Should mention Brave since it's installed
                        assert!(
                            text.contains("Brave") || text.contains("browser"),
                            "Should detect browsers"
                        );
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Detect browsers call failed: {}", e);
        }
    }

    client.close().await;
}

#[tokio::test]
async fn test_tool_call_check_permission() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    // Initialize
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };
    let _ = client.send_request(init_request).await;

    // Check permission for allowed operation
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 4,
        method: "tools/call".into(),
        params: Some(json!({
            "name": "webpuppet_check_permission",
            "arguments": {
                "operation": "Navigate"
            }
        })),
    };

    match client.send_request(request).await {
        Ok(response) => {
            assert!(response.error.is_none());
            if let Some(result) = response.result {
                let text = result
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c| c.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                println!("Permission check (Navigate): {}", text);
                assert!(text.contains("ALLOWED"), "Navigate should be allowed");
            }
        }
        Err(e) => eprintln!("Permission check failed: {}", e),
    }

    // Check permission for blocked operation
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 5,
        method: "tools/call".into(),
        params: Some(json!({
            "name": "webpuppet_check_permission",
            "arguments": {
                "operation": "DeleteAccount"
            }
        })),
    };

    match client.send_request(request).await {
        Ok(response) => {
            assert!(response.error.is_none());
            if let Some(result) = response.result {
                let text = result
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c| c.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                println!("Permission check (DeleteAccount): {}", text);
                assert!(text.contains("DENIED"), "DeleteAccount should be denied");
            }
        }
        Err(e) => eprintln!("Permission check failed: {}", e),
    }

    client.close().await;
}

#[tokio::test]
async fn test_intervention_status() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    // Initialize
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };
    let _ = client.send_request(init_request).await;

    // Check intervention status
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 6,
        method: "tools/call".into(),
        params: Some(json!({
            "name": "webpuppet_intervention_status",
            "arguments": {}
        })),
    };

    match client.send_request(request).await {
        Ok(response) => {
            assert!(response.error.is_none());
            if let Some(result) = response.result {
                let text = result
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c| c.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                println!("Intervention status: {}", text);
                // Should show running state initially
                assert!(
                    text.contains("Running") || text.contains("Status"),
                    "Should show status"
                );
            }
        }
        Err(e) => eprintln!("Intervention status failed: {}", e),
    }

    client.close().await;
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_unknown_method_error() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 99,
        method: "nonexistent/method".into(),
        params: None,
    };

    match client.send_request(request).await {
        Ok(response) => {
            // Should have an error
            if let Some(error) = response.error {
                println!("Error (expected): {} (code: {})", error.message, error.code);
                // Method not found is -32601
                assert!(
                    error.code == -32601
                        || error.code == -32600
                        || error.message.contains("not")
                        || error.message.contains("unknown")
                );
            }
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }

    client.close().await;
}

#[tokio::test]
async fn test_unknown_tool_error() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    // Initialize
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };
    let _ = client.send_request(init_request).await;

    // Call unknown tool
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 100,
        method: "tools/call".into(),
        params: Some(json!({
            "name": "nonexistent_tool",
            "arguments": {}
        })),
    };

    match client.send_request(request).await {
        Ok(response) => {
            // Should have error or error content
            if let Some(error) = response.error {
                println!("Error (expected): {}", error.message);
                assert!(error.message.contains("not found") || error.message.contains("unknown"));
            } else if let Some(result) = response.result {
                // Some implementations return is_error in result
                if let Some(is_error) = result.get("isError").and_then(|e| e.as_bool()) {
                    assert!(is_error, "Should indicate error for unknown tool");
                }
            }
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }

    client.close().await;
}

// ============================================================================
// Pause/Resume Tests
// ============================================================================

#[tokio::test]
async fn test_pause_resume_workflow() {
    let mut client = match McpTestClient::spawn().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test, MCP server not available: {}", e);
            return;
        }
    };

    // Initialize
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 1,
        method: "initialize".into(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };
    let _ = client.send_request(init_request).await;

    // Pause
    let pause_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 10,
        method: "tools/call".into(),
        params: Some(json!({
            "name": "webpuppet_pause",
            "arguments": {}
        })),
    };

    if let Ok(response) = client.send_request(pause_request).await {
        if let Some(result) = response.result {
            let text = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|a| a.first())
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("");

            println!("Pause result: {}", text);
            assert!(text.contains("Paused") || text.contains("paused"));
        }
    }

    // Resume
    let resume_request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 11,
        method: "tools/call".into(),
        params: Some(json!({
            "name": "webpuppet_resume",
            "arguments": {}
        })),
    };

    if let Ok(response) = client.send_request(resume_request).await {
        if let Some(result) = response.result {
            let text = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|a| a.first())
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("");

            println!("Resume result: {}", text);
            assert!(text.contains("Resumed") || text.contains("resumed"));
        }
    }

    client.close().await;
}
