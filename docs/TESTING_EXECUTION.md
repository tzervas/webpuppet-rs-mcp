# Testing Execution Guide

## Overview

This guide walks through the complete testing process for v0.1.0-alpha.3 before merging to main/production.

## Prerequisites

### Install Testing Tools

```bash
# Rust toolchain (minimum 1.75.0)
rustup update

# Security and analysis tools
cargo install cargo-audit
cargo install cargo-outdated  
cargo install cargo-license
cargo install cargo-sbom

# Optional: Secret scanning
# Install gitleaks from: https://github.com/gitleaks/gitleaks
```

### System Requirements

- **OS**: Linux, macOS, or Windows 10+
- **RAM**: 4GB minimum, 8GB recommended
- **Browser**: Chromium, Chrome, Brave, or Edge
- **Disk**: 2GB free space

## Step-by-Step Testing Process

### Phase 1: Automated Testing (15-20 minutes)

#### 1.1 Pull the Release Branch

```bash
cd /path/to/webpuppet-rs-mcp
git fetch origin
git checkout release/v0.1.0-alpha.3-to-main
```

#### 1.2 Run Unit Tests

```bash
# Run all unit tests with verbose output
cargo test -- --nocapture --test-threads=1

# Expected: 8/8 tests pass
# Duration: ~30 seconds
```

**Success Criteria**:
- ✅ All 8 tests pass
- ✅ No panics or crashes
- ✅ Test duration < 60 seconds

#### 1.3 Run Integration Test Suite

```bash
# Execute comprehensive automated test suite
./scripts/test_mcp_server.sh

# This will:
# - Validate environment
# - Clean build
# - Run unit tests
# - Security scan
# - Runtime testing
# - Generate report
```

**Success Criteria**:
- ✅ Script completes without errors (exit code 0)
- ✅ Build successful
- ✅ All tests pass
- ✅ Runtime test succeeds
- ✅ Report generated in test-logs/

**Review Output**:
```bash
# Check the generated test report
cat test-logs/test_report_*.md

# Look for:
# - "All tests passed successfully!"
# - Error count: 0
# - Warning count: low (<5)
```

#### 1.4 Run Security Audit

```bash
# Execute security audit
./scripts/security_audit.sh

# This will:
# - Scan for vulnerabilities
# - Check outdated dependencies
# - Verify licenses
# - Run clippy
# - Detect unsafe code
# - Scan for secrets
# - Generate SBOM
```

**Success Criteria**:
- ✅ No high or critical vulnerabilities
- ✅ All licenses compatible (MIT/Apache/BSD)
- ✅ Clippy passes with no errors
- ✅ No exposed secrets

**Review Output**:
```bash
# Check the security audit log
cat security-audits/audit_*.log

# Look for:
# - "0 vulnerabilities found" or acceptable known issues
# - "Clippy analysis complete" with no errors
# - No secret detections
```

### Phase 2: Manual Deployment Testing (30-45 minutes)

#### 2.1 Build Release Binary

```bash
# Clean build
cargo clean
cargo build --release

# Verify binary
ls -lh target/release/webpuppet-mcp
file target/release/webpuppet-mcp
```

**Success Criteria**:
- ✅ Binary created successfully
- ✅ Size < 100MB (typically ~40-50MB)
- ✅ Correct architecture (x86_64 or ARM64)

#### 2.2 Test Basic Execution

```bash
# Test help command
./target/release/webpuppet-mcp --help

# Expected: Help text displays without errors

# Test version
./target/release/webpuppet-mcp --version

# Expected: Shows v0.1.0-alpha.3
```

#### 2.3 Test MCP Protocol

```bash
# Start server in stdio mode with verbose logging
./target/release/webpuppet-mcp --stdio --policy secure --verbose 2> mcp-test.log &
SERVER_PID=$!

# Send initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/webpuppet-mcp --stdio

# Expected: JSON response with "result" field

# Clean up
kill $SERVER_PID

# Review logs
cat mcp-test.log
```

**Success Criteria**:
- ✅ Server starts without errors
- ✅ Initialize request returns valid response
- ✅ No errors in logs
- ✅ Server shuts down cleanly

#### 2.4 Deploy to MCP Client

**Option A: VS Code / GitHub Copilot**

1. Create or edit `.vscode/mcp.json`:
```json
{
  "mcpServers": {
    "webpuppet": {
      "command": "/path/to/webpuppet-rs-mcp/target/release/webpuppet-mcp",
      "args": ["--stdio", "--policy", "secure", "--verbose"]
    }
  }
}
```

2. Restart VS Code
3. Open Copilot Chat
4. Verify "webpuppet" server appears
5. Test command: "List available webpuppet tools"

**Option B: Claude Desktop**

1. Edit Claude Desktop config:
   - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
   - Linux: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "webpuppet": {
      "command": "/path/to/webpuppet-rs-mcp/target/release/webpuppet-mcp",
      "args": ["--stdio", "--policy", "secure"]
    }
  }
}
```

2. Restart Claude Desktop
3. Verify connection in settings
4. Test: Ask Claude to list webpuppet capabilities

#### 2.5 Test All 10 Tools

For each tool, verify it responds correctly:

1. **webpuppet_list_providers**
   - Command: "List available AI providers"
   - Expected: JSON list of providers (Claude, Gemini, ChatGPT, etc.)

2. **webpuppet_detect_browsers**
   - Command: "Detect installed browsers"
   - Expected: List of detected Chromium-based browsers

3. **webpuppet_check_permission**
   - Command: "Check permission for navigate to example.com"
   - Expected: Permission status based on policy

4. **webpuppet_provider_capabilities**
   - Command: "What are Claude's capabilities?"
   - Expected: Tool list and capabilities

5. **webpuppet_intervention_status**
   - Command: "Check intervention status"
   - Expected: No intervention needed

6. **webpuppet_pause**
   - Command: "Pause automation"
   - Expected: Confirmation message

7. **webpuppet_resume**
   - Command: "Resume automation"
   - Expected: Confirmation message

8. **webpuppet_intervention_complete**
   - Command: "Mark intervention complete"
   - Expected: Acknowledgment

9. **webpuppet_screenshot** *(requires browser)*
   - Command: "Take screenshot of example.com"
   - Expected: Base64 image data or error if no browser

10. **webpuppet_prompt** *(requires browser and provider)*
    - Command: "Send prompt 'hello' to ChatGPT"
    - Expected: Response or intervention needed

**Success Criteria**:
- ✅ All tools respond without server crashes
- ✅ Error messages are clear and helpful
- ✅ Permission system enforces policies correctly
- ✅ No memory leaks during tool execution

#### 2.6 Test Permission Policies

```bash
# Test secure policy (default)
./target/release/webpuppet-mcp --stdio --policy secure

# Test permissive policy
./target/release/webpuppet-mcp --stdio --policy permissive

# Test readonly policy
./target/release/webpuppet-mcp --stdio --policy readonly
```

**Test scenarios**:
- Secure: Navigation should require explicit permission
- Permissive: All operations allowed
- Readonly: Write operations blocked

#### 2.7 Test Visible Browser Mode

```bash
# Start with visible browser
./target/release/webpuppet-mcp --stdio --visible --verbose
```

**Success Criteria**:
- ✅ Browser window appears on screen
- ✅ Can see automation in action
- ✅ Browser closes cleanly on exit

### Phase 3: Extended Monitoring (24-48 hours)

#### 3.1 Long-Running Test

```bash
# Start server and leave running
./target/release/webpuppet-mcp --stdio --policy secure --verbose > mcp-long-test.log 2>&1 &
SERVER_PID=$!

# Monitor periodically
watch -n 60 'ps aux | grep webpuppet-mcp'

# Check memory usage
ps -o pid,rss,vsz,comm -p $SERVER_PID

# After 24-48 hours
kill $SERVER_PID

# Review logs for issues
grep -i "error\|panic\|crash" mcp-long-test.log
```

**Success Criteria**:
- ✅ No crashes or restarts
- ✅ Memory usage stable (no leaks)
- ✅ No error accumulation
- ✅ Clean shutdown

#### 3.2 Resource Monitoring

Monitor system resources during operation:

```bash
# CPU usage
top -p $SERVER_PID

# Memory usage over time
while true; do
  ps -o rss= -p $SERVER_PID >> memory-log.txt
  sleep 60
done

# Network connections (should be minimal when idle)
netstat -an | grep $SERVER_PID
```

**Success Criteria**:
- ✅ CPU usage < 5% when idle
- ✅ Memory growth < 10% over 24 hours
- ✅ No unexpected network connections

### Phase 4: Final Validation

#### 4.1 Review All Logs

```bash
# Collect all test artifacts
ls -R test-logs/
ls -R security-audits/

# Check for any missed errors
find test-logs -name "*.log" -exec grep -l "ERROR\|CRITICAL" {} \;

# Review test report
cat test-logs/test_report_*.md
```

#### 4.2 Performance Validation

Verify performance metrics:

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| Startup time | < 5s | _____ | ⬜ |
| Memory (idle) | < 100MB | _____ | ⬜ |
| Memory (active) | < 500MB | _____ | ⬜ |
| Binary size | < 100MB | _____ | ⬜ |
| Initialize response | < 1s | _____ | ⬜ |

#### 4.3 Security Validation

```bash
# Final security check
cargo audit
cargo clippy -- -D warnings

# Verify no secrets
git log -p | grep -i "password\|secret\|api_key\|token"
```

#### 4.4 Documentation Review

Verify all documentation is accurate:

- [ ] README.md reflects current version
- [ ] CHANGELOG.md is complete
- [ ] TESTING.md is accurate
- [ ] All code comments are current
- [ ] Examples work as documented

## Test Results Recording

Create a test results document:

```markdown
# Test Results - v0.1.0-alpha.3

**Date**: [Date]
**Tester**: [Name]
**Environment**: [OS, Rust version, Browser]

## Automated Tests
- [ ] Unit tests: 8/8 passed
- [ ] Integration tests: PASS
- [ ] Security audit: PASS / WARNINGS / FAIL
- [ ] Notes: [Any issues]

## Manual Tests
- [ ] All 10 tools tested
- [ ] Permission policies validated
- [ ] Visible mode works
- [ ] MCP client integration successful
- [ ] Notes: [Any issues]

## Extended Monitoring
- [ ] 24-hour stability test
- [ ] No memory leaks detected
- [ ] Performance within targets
- [ ] Notes: [Any issues]

## Final Verdict
- [ ] ✅ APPROVED - Ready to merge
- [ ] ⚠️ APPROVED WITH CAVEATS - [List caveats]
- [ ] ❌ REJECTED - [List blocking issues]

**Signature**: _____________
**Date**: _____________
```

## Common Issues & Troubleshooting

### Issue: Tests Fail to Compile

```bash
# Solution: Clean and rebuild
cargo clean
rm -rf target/
cargo build --release
cargo test
```

### Issue: Browser Not Detected

```bash
# Solution: Install browser
# Ubuntu/Debian:
sudo apt-get install chromium-browser

# macOS:
brew install --cask google-chrome

# Or use Brave, Edge, etc.
```

### Issue: Permission Denied on Scripts

```bash
# Solution: Make scripts executable
chmod +x scripts/test_mcp_server.sh
chmod +x scripts/security_audit.sh
```

### Issue: MCP Client Can't Connect

1. Check binary path is correct
2. Verify stdio mode is used
3. Check logs for errors:
   ```bash
   tail -f ~/.vscode/logs/mcp.log
   # or
   tail -f ~/Library/Logs/Claude/mcp.log
   ```

### Issue: High Memory Usage

1. Monitor for leaks:
   ```bash
   valgrind --leak-check=full ./target/release/webpuppet-mcp --stdio
   ```
2. Check browser process isn't lingering:
   ```bash
   ps aux | grep chrome
   killall chrome
   ```

## Sign-Off Checklist

Before approving the PR, confirm:

- [ ] All automated tests pass
- [ ] Security audit clean
- [ ] All 10 tools manually tested
- [ ] Permission policies validated
- [ ] Extended monitoring complete (24h+)
- [ ] Performance metrics met
- [ ] Logs reviewed - no critical issues
- [ ] Documentation accurate
- [ ] Test results recorded

**Final Approval**:
- Tester: _______________
- Reviewer: _______________
- Date: _______________

## References

- [TESTING.md](../TESTING.md) - Complete testing documentation
- [PR_UPSTREAM_v0.1.0-alpha.3.md](../PR_UPSTREAM_v0.1.0-alpha.3.md) - PR details
- [scripts/test_mcp_server.sh](../scripts/test_mcp_server.sh) - Automated test script
- [scripts/security_audit.sh](../scripts/security_audit.sh) - Security audit script
