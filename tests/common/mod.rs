//! Shared MCP stdio test client.
//!
//! # Failure posture
//!
//! This harness **panics** when it cannot reach the server. It deliberately does not
//! offer a "skip if unavailable" path.
//!
//! The previous harness returned `Err` from `spawn()` and every test answered that with
//! `eprintln!("Skipping test..."); return;`. Because the binary path it used was wrong
//! (`$CARGO_MANIFEST_DIR/../../target/release/webpuppet-mcp`, two directories above the
//! crate), `spawn()` always failed — so all nine tests skipped and the suite reported
//! `9 passed`. CI was green over zero coverage.
//!
//! The binary is now located with `CARGO_BIN_EXE_webpuppet-mcp`, which Cargo sets for
//! integration tests and points at the binary Cargo just built. That also removes the
//! nested `cargo build` the old harness shelled out to on every single test.

#![allow(dead_code)]

use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

/// Path to the server binary under test, supplied by Cargo.
pub const SERVER_BIN: &str = env!("CARGO_BIN_EXE_webpuppet-mcp");

/// How long to wait for a single response line.
pub const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    /// First text content item of a `tools/call` result.
    ///
    /// Panics if the response was an error or carried no text content, so a malformed
    /// reply fails the test instead of silently yielding `""` (the old harness used
    /// `.unwrap_or("")`, which made every content assertion vacuously skippable).
    pub fn text(&self) -> &str {
        if let Some(err) = &self.error {
            panic!(
                "expected a successful result, got JSON-RPC error {}: {}",
                err.code, err.message
            );
        }
        self.result
            .as_ref()
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or_else(|| panic!("response has no text content: {:?}", self.result))
    }

    /// Whether the tool result is flagged as an error.
    pub fn is_error_result(&self) -> bool {
        self.result
            .as_ref()
            .and_then(|r| r.get("isError"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false)
    }
}

/// MCP test client driving the real server binary over stdio.
pub struct McpTestClient {
    child: Child,
    stdin: ChildStdin,
    /// Persistent reader. Creating a fresh `BufReader` per request (as the old harness
    /// did) discards whatever the previous read buffered, which drops responses.
    stdout: BufReader<ChildStdout>,
}

impl McpTestClient {
    /// Spawn the server with the `secure` policy. Panics if it cannot be started.
    pub async fn spawn() -> Self {
        Self::spawn_with_args(&["--policy", "secure"]).await
    }

    /// Spawn the server with explicit CLI arguments. Panics if it cannot be started.
    pub async fn spawn_with_args(args: &[&str]) -> Self {
        let mut child = Command::new(SERVER_BIN)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Inherit stderr: server logs stay visible under `--nocapture`, and an
            // inherited fd cannot fill up and wedge the server the way a pipe can.
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap_or_else(|e| {
                panic!("failed to spawn MCP server at {SERVER_BIN}: {e}");
            });

        let stdin = child.stdin.take().expect("child stdin");
        let stdout = BufReader::new(child.stdout.take().expect("child stdout"));

        Self {
            child,
            stdin,
            stdout,
        }
    }

    /// Send a request and read one response line. Panics on I/O failure or timeout.
    pub async fn request(
        &mut self,
        id: u64,
        method: &str,
        params: Option<Value>,
    ) -> JsonRpcResponse {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id,
            method: method.into(),
            params,
        };
        let line = serde_json::to_string(&request).expect("serialize request");

        self.stdin
            .write_all(line.as_bytes())
            .await
            .expect("write request");
        self.stdin.write_all(b"\n").await.expect("write newline");
        self.stdin.flush().await.expect("flush request");

        let mut response_line = String::new();
        let read = timeout(RESPONSE_TIMEOUT, self.stdout.read_line(&mut response_line))
            .await
            .unwrap_or_else(|_| {
                panic!("timed out after {RESPONSE_TIMEOUT:?} waiting for a response to {method} (id {id})")
            })
            .expect("read response line");

        assert_ne!(
            read, 0,
            "server closed stdout while waiting for a response to {method} (id {id})"
        );

        serde_json::from_str(&response_line).unwrap_or_else(|e| {
            panic!("malformed response to {method} (id {id}): {e}\nraw: {response_line}")
        })
    }

    /// Perform the MCP handshake. Panics if it does not succeed.
    pub async fn initialize(&mut self) -> JsonRpcResponse {
        let response = self
            .request(
                1,
                "initialize",
                Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "webpuppet-mcp-tests", "version": "0.1.0"}
                })),
            )
            .await;

        assert!(
            response.error.is_none(),
            "initialize failed: {:?}",
            response.error
        );
        response
    }

    /// Call a tool by name.
    pub async fn call_tool(&mut self, id: u64, name: &str, arguments: Value) -> JsonRpcResponse {
        self.request(
            id,
            "tools/call",
            Some(json!({ "name": name, "arguments": arguments })),
        )
        .await
    }

    /// Kill the server process.
    pub async fn close(mut self) {
        let _ = self.child.kill().await;
    }
}

/// Whether a Chromium-family browser the library can drive is present on this machine.
///
/// Used only to produce an explicit, actionable panic message in browser-dependent
/// tests. Those tests are `#[ignore]`d by default so their absence is reported by the
/// test summary rather than hidden behind a silent early return.
pub fn browser_available() -> bool {
    [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "brave-browser",
        "microsoft-edge",
    ]
    .iter()
    .any(|bin| {
        std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(bin).is_file()))
            .unwrap_or(false)
    })
}
