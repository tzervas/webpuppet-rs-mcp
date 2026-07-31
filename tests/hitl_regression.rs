//! Regression tests for the HITL (human-in-the-loop) defects.
//!
//! Each test below is annotated with how it behaves against the pre-fix code.

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::sync::RwLock;

use webpuppet::{InterventionHandler, InterventionState};
use webpuppet_mcp::tools::{default_intervention_wait, wait_while_paused, INTERVENTION_WAIT_ENV};
use webpuppet_mcp::Error;

use common::McpTestClient;

// ============================================================================
// Defect 1: the HITL pause deadlocked the server (unbounded poll loop)
// ============================================================================

/// REGRESSION. Pre-fix this test does not compile, because the wait was an unbounded
/// `while handler.is_waiting() { sleep(500ms) }` loop inlined inside
/// `ToolContext::get_active_or_fallback` — unreachable without a live browser session,
/// and with no exit condition at all. See `old_unbounded_poll_loop_never_terminates`
/// below, which reproduces that loop verbatim and shows it never returns.
#[tokio::test]
async fn paused_session_wait_is_bounded_and_returns_session_paused() {
    let handler = Arc::new(RwLock::new(InterventionHandler::new()));
    handler.read().await.pause();
    assert!(handler.read().await.is_waiting());

    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        wait_while_paused(&handler, "session-under-test", Duration::from_millis(300)),
    )
    .await
    .expect("wait_while_paused never returned: the paused-session wait is unbounded again");

    match outcome {
        Err(Error::SessionPaused {
            session_id,
            reason,
            waited_secs,
        }) => {
            assert_eq!(session_id, "session-under-test");
            assert!(
                reason.to_lowercase().contains("paus"),
                "reason should name the pause, got: {reason}"
            );
            let _ = waited_secs;
        }
        other => panic!("expected Error::SessionPaused, got {other:?}"),
    }
}

/// The bounded wait must return immediately once the pause is cleared, not sit out the
/// remaining timeout.
#[tokio::test]
async fn paused_session_wait_returns_as_soon_as_pause_clears() {
    let handler = Arc::new(RwLock::new(InterventionHandler::new()));
    handler.read().await.pause();

    let releaser = Arc::clone(&handler);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        releaser.read().await.resume();
    });

    let started = Instant::now();
    let outcome = wait_while_paused(&handler, "session-under-test", Duration::from_secs(30)).await;
    let elapsed = started.elapsed();

    assert!(outcome.is_ok(), "expected Ok once resumed, got {outcome:?}");
    assert!(
        elapsed < Duration::from_secs(5),
        "wait should end when the pause clears, took {elapsed:?}"
    );
}

/// An unpaused handler must not delay the call at all.
#[tokio::test]
async fn unpaused_session_wait_is_a_no_op() {
    let handler = Arc::new(RwLock::new(InterventionHandler::new()));
    assert_eq!(handler.read().await.state(), InterventionState::Running);

    let started = Instant::now();
    let outcome = wait_while_paused(&handler, "s", Duration::from_secs(30)).await;

    assert!(outcome.is_ok(), "expected Ok, got {outcome:?}");
    assert!(started.elapsed() < Duration::from_secs(1));
}

/// EVIDENCE (not a regression test): reproduces the pre-fix loop verbatim and shows it
/// never terminates while the handler is paused. This is what wedged the server.
#[tokio::test]
async fn old_unbounded_poll_loop_never_terminates() {
    let handler = Arc::new(RwLock::new(InterventionHandler::new()));
    handler.read().await.pause();

    let hung = tokio::time::timeout(Duration::from_millis(1500), async {
        // Verbatim shape of the removed code.
        while handler.read().await.is_waiting() {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
    .await
    .is_err();

    assert!(
        hung,
        "the old loop terminated on its own; the deadlock diagnosis would be wrong"
    );
}

/// The wait bound is configurable, and defaults to a value short enough that an MCP
/// client gets an answer rather than an open request.
#[test]
fn intervention_wait_is_configurable() {
    // Default (env unset in the normal test environment).
    if std::env::var_os(INTERVENTION_WAIT_ENV).is_none() {
        assert_eq!(default_intervention_wait(), Duration::from_secs(30));
    }
    assert!(
        default_intervention_wait() <= Duration::from_secs(120),
        "default wait must stay well under a typical MCP client request timeout"
    );
}

// ============================================================================
// Defect 2: webpuppet_intervention_complete did not clear a pause
// ============================================================================

/// REGRESSION. Pre-fix this fails at the final assertion: `intervention_complete`
/// replied "Automation will now resume" while `intervention_status` still reported
/// "Waiting for human", because `InterventionHandler::complete()` only `try_send`s on a
/// capacity-1 channel that has no receiver unless `request_intervention()` is being
/// awaited. Measured pre-fix output of the last call:
///
/// ```text
/// # Intervention Status
/// **State**: 🟡 Waiting for human
/// **Reason**: Automation paused
/// ```
#[tokio::test]
async fn intervention_complete_clears_a_pause() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    client.call_tool(2, "webpuppet_pause", json!({})).await;

    let status = client
        .call_tool(3, "webpuppet_intervention_status", json!({}))
        .await;
    assert!(
        status.text().contains("Waiting for human"),
        "precondition failed, pause did not take effect:\n{}",
        status.text()
    );
    // The server tells the user to call exactly this tool; that instruction must work.
    assert!(
        status.text().contains("webpuppet_intervention_complete"),
        "status no longer directs the user to intervention_complete:\n{}",
        status.text()
    );

    let complete = client
        .call_tool(
            4,
            "webpuppet_intervention_complete",
            json!({"success": true, "message": "solved the captcha"}),
        )
        .await;
    assert!(complete.error.is_none(), "{:?}", complete.error);

    let status = client
        .call_tool(5, "webpuppet_intervention_status", json!({}))
        .await;
    assert!(
        !status.text().contains("Waiting for human"),
        "webpuppet_intervention_complete did not clear the pause:\n{}",
        status.text()
    );
    assert!(
        status.text().contains("Running") || status.text().contains("Resuming"),
        "unexpected state after intervention_complete:\n{}",
        status.text()
    );

    client.close().await;
}

/// REGRESSION. A failed intervention must not read as a success. Pre-fix,
/// `success: false` still returned `is_error: false` and the text
/// "Automation will now resume", and left the state untouched at WaitingForHuman.
#[tokio::test]
async fn failed_intervention_is_reported_as_an_error_and_does_not_resume() {
    let mut client = McpTestClient::spawn().await;
    client.initialize().await;

    client.call_tool(2, "webpuppet_pause", json!({})).await;

    let complete = client
        .call_tool(
            3,
            "webpuppet_intervention_complete",
            json!({"success": false, "message": "could not solve it"}),
        )
        .await;

    assert!(
        complete.is_error_result(),
        "a failed intervention must set isError, got: {:?}",
        complete.result
    );
    assert!(
        !complete.text().contains("Automation will now resume"),
        "a failed intervention must not claim automation resumes:\n{}",
        complete.text()
    );

    let status = client
        .call_tool(4, "webpuppet_intervention_status", json!({}))
        .await;
    assert!(
        status.text().contains("Cancelled"),
        "a failed intervention should leave the run cancelled, got:\n{}",
        status.text()
    );

    client.close().await;
}

// ============================================================================
// Transport: concurrent dispatch must not break the handshake ordering
// ============================================================================

/// GUARD. `run_stdio` now dispatches requests concurrently so that a blocked HITL call
/// cannot stop the reader from consuming the `webpuppet_resume` that would release it.
/// Concurrency must not let a pipelined first tool call race past `initialize`: a naive
/// spawn-everything version answered this pause with "server not initialized".
#[tokio::test]
async fn pipelined_initialize_and_tool_call_are_ordered() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut child = tokio::process::Command::new(common::SERVER_BIN)
        .args(["--policy", "secure"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("spawn server");

    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));

    // Write both requests before reading anything.
    let batch = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"webpuppet_pause","arguments":{}}}"#,
        "\n",
    );
    stdin
        .write_all(batch.as_bytes())
        .await
        .expect("write batch");
    stdin.flush().await.expect("flush");

    let mut seen = std::collections::HashMap::new();
    for _ in 0..2 {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(10), stdout.read_line(&mut line))
            .await
            .expect("timed out reading pipelined responses")
            .expect("read line");
        assert_ne!(n, 0, "server closed stdout early");
        let value: serde_json::Value = serde_json::from_str(&line).expect("parse response");
        let id = value.get("id").and_then(|i| i.as_u64()).expect("id");
        seen.insert(id, value);
    }

    for id in [1, 2] {
        let value = seen
            .get(&id)
            .unwrap_or_else(|| panic!("no response for id {id}"));
        assert!(
            value.get("error").is_none() || value["error"].is_null(),
            "pipelined request {id} errored: {value}"
        );
    }

    let _ = child.kill().await;
}
