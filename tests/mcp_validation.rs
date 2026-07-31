//! MCP server validation tests.
//!
//! Tests JSON-RPC 2.0 protocol compliance, tool execution, and error handling against
//! the real server binary over stdio.
//!
//! Every test here previously skipped silently (see `tests/common/mod.rs`). They now run
//! for real, and any transport failure panics. The one test that genuinely needs a
//! Chromium-family browser is `#[ignore]`d so its absence shows up in the summary as
//! `1 ignored` rather than as a pass.

mod common;

use serde_json::json;

use common::{browser_available, McpTestClient};

// ============================================================================
// Protocol Compliance Tests
// ============================================================================

#[tokio::test]
async fn test_initialize_handshake() {
    let mut client = McpTestClient::spawn().await;

    let response = client.initialize().await;

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, Some(1));

    let result = response.result.as_ref().expect("initialize result");
    assert_eq!(
        result.get("protocolVersion").and_then(|v| v.as_str()),
        Some("2024-11-05"),
        "unexpected protocol version: {result}"
    );
    let server_info = result.get("serverInfo").expect("serverInfo");
    assert_eq!(
        server_info.get("name").and_then(|v| v.as_str()),
        Some("webpuppet-mcp")
    );
    assert!(
        server_info
            .get("version")
            .and_then(|v| v.as_str())
            .is_some(),
        "serverInfo.version missing: {server_info}"
    );

    client.close().await;
}

#[tokio::test]
async fn test_list_tools() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    let response = client.request(2, "tools/list", None).await;
    assert!(
        response.error.is_none(),
        "tools/list errored: {:?}",
        response.error
    );

    let result = response.result.as_ref().expect("tools/list result");
    let tools = result
        .get("tools")
        .and_then(|t| t.as_array())
        .expect("tools array");

    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    // The full stable target set from docs/ROADMAP.md, plus the shipped extras.
    for expected in [
        "webpuppet_prompt",
        "webpuppet_detect_browsers",
        "webpuppet_check_permission",
        "webpuppet_intervention_status",
        "webpuppet_intervention_complete",
        "webpuppet_pause",
        "webpuppet_resume",
        "webpuppet_session_open",
        "webpuppet_session_close",
        "webpuppet_navigate",
        "webpuppet_extract",
        "webpuppet_screenshot",
        "webpuppet_list_providers",
    ] {
        assert!(
            names.contains(&expected),
            "tools/list is missing {expected}; got {names:?}"
        );
    }

    // Every advertised tool must carry a usable JSON Schema.
    for tool in tools {
        let name = tool
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("<unnamed>");
        let schema = tool
            .get("inputSchema")
            .unwrap_or_else(|| panic!("tool {name} has no inputSchema"));
        assert_eq!(
            schema.get("type").and_then(|t| t.as_str()),
            Some("object"),
            "tool {name} inputSchema is not an object schema: {schema}"
        );
        assert!(
            tool.get("description")
                .and_then(|d| d.as_str())
                .is_some_and(|d| !d.trim().is_empty()),
            "tool {name} has an empty description"
        );
    }

    client.close().await;
}

#[tokio::test]
async fn test_tool_call_detect_browsers() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    let response = client
        .call_tool(3, "webpuppet_detect_browsers", json!({}))
        .await;
    assert!(
        response.error.is_none(),
        "detect_browsers errored: {:?}",
        response.error
    );

    // Assert on the report's shape, not on which browsers this particular host has.
    let text = response.text();
    assert!(
        text.contains("Browser") || text.contains("browser"),
        "detect_browsers output does not look like a browser report:\n{text}"
    );

    client.close().await;
}

#[tokio::test]
async fn test_tool_call_check_permission() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    let allowed = client
        .call_tool(
            4,
            "webpuppet_check_permission",
            json!({"operation": "Navigate"}),
        )
        .await;
    assert!(
        allowed.error.is_none(),
        "check_permission errored: {:?}",
        allowed.error
    );
    assert!(
        allowed.text().contains("ALLOWED"),
        "Navigate should be allowed under the secure policy:\n{}",
        allowed.text()
    );

    let denied = client
        .call_tool(
            5,
            "webpuppet_check_permission",
            json!({"operation": "DeleteAccount"}),
        )
        .await;
    assert!(
        denied.error.is_none(),
        "check_permission errored: {:?}",
        denied.error
    );
    assert!(
        denied.text().contains("DENIED"),
        "DeleteAccount must be denied under the secure policy:\n{}",
        denied.text()
    );

    client.close().await;
}

#[tokio::test]
async fn test_intervention_status() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    let response = client
        .call_tool(6, "webpuppet_intervention_status", json!({}))
        .await;
    assert!(
        response.error.is_none(),
        "intervention_status errored: {:?}",
        response.error
    );

    let text = response.text();
    assert!(
        text.contains("Running"),
        "a freshly started server must report Running, got:\n{text}"
    );

    client.close().await;
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_unknown_method_error() {
    let mut client = McpTestClient::spawn().await;

    let response = client.request(99, "nonexistent/method", None).await;

    let error = response
        .error
        .as_ref()
        .expect("unknown method must produce a JSON-RPC error, not a result");
    assert_eq!(
        error.code, -32601,
        "unknown method should be METHOD_NOT_FOUND (-32601), got {}: {}",
        error.code, error.message
    );

    client.close().await;
}

#[tokio::test]
async fn test_unknown_tool_error() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    let response = client.call_tool(100, "nonexistent_tool", json!({})).await;

    // Either a JSON-RPC error or an isError result is acceptable; silence is not.
    match response.error.as_ref() {
        Some(error) => assert_eq!(
            error.code, -32601,
            "unknown tool should map to -32601, got {}: {}",
            error.code, error.message
        ),
        None => assert!(
            response.is_error_result(),
            "unknown tool produced neither an error nor isError: {:?}",
            response.result
        ),
    }

    client.close().await;
}

// ============================================================================
// Pause/Resume Tests
// ============================================================================

#[tokio::test]
async fn test_pause_resume_workflow() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    let paused = client.call_tool(10, "webpuppet_pause", json!({})).await;
    assert!(
        paused.text().to_lowercase().contains("paused"),
        "pause did not report a pause:\n{}",
        paused.text()
    );

    // The pause must be observable, not just announced.
    let status = client
        .call_tool(11, "webpuppet_intervention_status", json!({}))
        .await;
    assert!(
        status.text().contains("Waiting for human"),
        "after webpuppet_pause the state must be WaitingForHuman, got:\n{}",
        status.text()
    );

    let resumed = client.call_tool(12, "webpuppet_resume", json!({})).await;
    assert!(
        resumed.text().to_lowercase().contains("resumed"),
        "resume did not report a resume:\n{}",
        resumed.text()
    );

    let status = client
        .call_tool(13, "webpuppet_intervention_status", json!({}))
        .await;
    assert!(
        !status.text().contains("Waiting for human"),
        "webpuppet_resume did not clear the pause:\n{}",
        status.text()
    );

    client.close().await;
}

// ============================================================================
// Persistent Session Tests
// ============================================================================

/// Full session lifecycle against a real browser.
///
/// Requires a Chromium-family browser and network access to x.com, so it is ignored by
/// default. Run with `cargo test -- --ignored`. It is `#[ignore]`d rather than
/// self-skipping so the test summary reports it as ignored instead of as a pass.
#[tokio::test]
#[ignore = "requires a Chromium-family browser and network access; run with --ignored"]
async fn test_persistent_session_workflow() {
    assert!(
        browser_available(),
        "no Chromium-family browser on PATH; install one before running --ignored tests"
    );

    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    let open = client
        .call_tool(
            12,
            "webpuppet_session_open",
            json!({"provider": "grok", "visible": false}),
        )
        .await;
    assert!(
        open.error.is_none(),
        "session_open errored: {:?}",
        open.error
    );

    let text = open.text().to_string();
    assert!(
        text.contains("Session Created"),
        "session_open output unexpected:\n{text}"
    );

    let session_id = open
        .result
        .as_ref()
        .and_then(|r| r.get("_meta"))
        .and_then(|m| m.get("session_id"))
        .and_then(|s| s.as_str())
        .map(str::to_string)
        .or_else(|| {
            text.split('`')
                .find(|part| {
                    part.len() == 36 && part.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
                })
                .map(str::to_string)
        })
        .expect("session_open must return a session_id");

    for (id, tool, args) in [
        (
            14,
            "webpuppet_navigate",
            json!({"session_id": session_id, "url": "https://x.com/i/grok"}),
        ),
        (
            15,
            "webpuppet_extract",
            json!({"session_id": session_id, "selector": "body"}),
        ),
        (
            17,
            "webpuppet_screenshot",
            json!({"session_id": session_id}),
        ),
    ] {
        let response = client.call_tool(id, tool, args).await;
        assert!(
            response.error.is_none(),
            "{tool} errored: {:?}",
            response.error
        );
    }

    let close = client
        .call_tool(
            13,
            "webpuppet_session_close",
            json!({"session_id": session_id}),
        )
        .await;
    assert!(
        close.error.is_none(),
        "session_close errored: {:?}",
        close.error
    );
    assert!(
        close.text().contains("closed successfully"),
        "session_close output unexpected:\n{}",
        close.text()
    );

    client.close().await;
}
