# Testing Documentation

## Overview

This document describes the comprehensive testing strategy for `embeddenator-webpuppet-mcp` v0.1.0-alpha.3. All tests must pass before merging to production.

## Test Categories

### 1. Unit Tests
**Location**: `tests/mcp_validation.rs`  
**Command**: `cargo test`  
**Duration**: ~30 seconds

Validates core MCP protocol functionality:
- JSON-RPC 2.0 message handling
- Tool registration and invocation
- Error handling
- Permission system
- Intervention flow

**Success Criteria**: All 8 tests pass with no warnings

### 2. Integration Tests
**Location**: `scripts/test_mcp_server.sh`  
**Command**: `./scripts/test_mcp_server.sh`  
**Duration**: ~60 seconds

Validates end-to-end functionality:
- Clean build process
- Binary generation
- Runtime execution in isolated environment
- MCP protocol handshake
- Resource usage
- Log analysis

**Success Criteria**: 
- Zero runtime errors
- Successful initialize handshake
- Binary size < 100MB
- Clean exit

### 3. Security Audit
**Location**: `scripts/security_audit.sh`  
**Command**: `./scripts/security_audit.sh`  
**Duration**: ~2-5 minutes

Validates security posture:
- Dependency vulnerability scanning (cargo-audit)
- Outdated dependency detection
- License compliance
- Code quality (clippy)
- Unsafe code detection
- Secret scanning (gitleaks)
- SBOM generation

**Success Criteria**:
- No high/critical vulnerabilities
- No exposed secrets
- Minimal unsafe code usage
- All dependencies properly licensed

### 4. Manual Testing
**Location**: Local deployment  
**Duration**: 15-30 minutes

Validates real-world usage:
- Deploy MCP server locally
- Connect with MCP client (VS Code/Claude Desktop)
- Test each tool:
  - `webpuppet_prompt`
  - `webpuppet_screenshot`
  - `webpuppet_list_providers`
  - `webpuppet_provider_capabilities`
  - `webpuppet_detect_browsers`
  - `webpuppet_check_permission`
  - `webpuppet_intervention_status`
  - `webpuppet_intervention_complete`
  - `webpuppet_pause`
  - `webpuppet_resume`
- Verify permission policies (secure, permissive, readonly)
- Test visible browser mode

**Success Criteria**:
- All tools respond correctly
- No crashes or hangs
- Proper error messages
- Permission system enforced

## Test Environment Requirements

### Minimum Requirements
- Rust 1.75.0+
- Linux/macOS/Windows 10+
- 4GB RAM
- Chromium-based browser (Brave/Chrome/Chromium)

### Recommended Tools
```bash
# Install testing dependencies
cargo install cargo-audit
cargo install cargo-outdated
cargo install cargo-license
cargo install cargo-sbom

# Install gitleaks (secret scanning)
# https://github.com/gitleaks/gitleaks
```

## Running the Test Suite

### Quick Test (5 minutes)
```bash
# Run unit tests only
cargo test
```

### Standard Test (10 minutes)
```bash
# Run unit tests and integration tests
cargo test
./scripts/test_mcp_server.sh
```

### Comprehensive Test (20 minutes)
```bash
# Run all automated tests including security audit
cargo test
./scripts/test_mcp_server.sh
./scripts/security_audit.sh
```

### Full Validation (45 minutes)
```bash
# Automated + Manual testing
cargo test
./scripts/test_mcp_server.sh
./scripts/security_audit.sh
# + Manual deployment testing (see Manual Testing section)
```

## Test Logs

All test runs generate detailed logs in:
- `test-logs/` - Integration test logs
- `security-audits/` - Security audit reports

Log structure:
```
test-logs/
├── test_YYYYMMDD_HHMMSS.log          # Main test log
├── build_YYYYMMDD_HHMMSS.log         # Build output
├── runtime_YYYYMMDD_HHMMSS.log       # Runtime logs
├── response_YYYYMMDD_HHMMSS.json     # MCP responses
├── test_report_YYYYMMDD_HHMMSS.md    # Test summary report
└── sandbox_YYYYMMDD_HHMMSS/          # Isolated test environment

security-audits/
├── audit_YYYYMMDD_HHMMSS.log         # Security audit log
├── vulnerabilities_YYYYMMDD_HHMMSS.json
├── licenses_YYYYMMDD_HHMMSS.txt
└── sbom_YYYYMMDD_HHMMSS.json         # Software Bill of Materials
```

## Continuous Integration

### GitHub Actions
Automated tests run on:
- Every push to `dev`, `testing`, `main`
- Every pull request
- Scheduled: Daily at 00:00 UTC

**CI Pipeline**:
1. ✅ Checkout code
2. ✅ Setup Rust toolchain
3. ✅ Run `cargo check`
4. ✅ Run `cargo test`
5. ✅ Run `cargo clippy`
6. ✅ Run security audit (if cargo-audit available)
7. ✅ Build release binary
8. 📦 Archive build artifacts

## Test Failure Protocol

### If Unit Tests Fail
1. Review test output: `cargo test -- --nocapture`
2. Check for breaking changes in dependencies
3. Fix code or update tests
4. Re-run full test suite
5. Document changes in CHANGELOG.md

### If Integration Tests Fail
1. Review logs in `test-logs/`
2. Check system dependencies (browser, etc.)
3. Verify network connectivity (for git dependencies)
4. Run in debug mode: `RUST_LOG=debug ./scripts/test_mcp_server.sh`
5. Fix issues and re-test

### If Security Audit Fails
1. Review `security-audits/audit_*.log`
2. For vulnerabilities:
   - Check if patches are available
   - Update dependencies: `cargo update`
   - If no patch exists, document risk and create tracking issue
3. For license issues:
   - Review incompatible licenses
   - Replace or get legal approval
4. For secrets:
   - Remove immediately
   - Rotate any exposed credentials
   - Update `.gitignore`

### If Manual Tests Fail
1. Document failure scenario
2. Capture screenshots/logs
3. Create detailed bug report
4. Fix and re-test affected tools
5. Update test documentation

## Release Checklist

Before merging to `main`:

- [ ] All unit tests pass (8/8)
- [ ] Integration tests pass (0 errors)
- [ ] Security audit clean (0 high/critical issues)
- [ ] Manual testing complete (all 10 tools validated)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped appropriately
- [ ] Test logs reviewed and archived
- [ ] No regressions from previous version
- [ ] Performance acceptable (startup < 5s)
- [ ] Memory usage reasonable (< 500MB)

## Performance Benchmarks

Expected performance metrics:

| Metric | Target | Critical |
|--------|--------|----------|
| Startup time | < 3s | < 10s |
| Memory usage (idle) | < 100MB | < 500MB |
| Memory usage (active) | < 300MB | < 1GB |
| Binary size | < 50MB | < 150MB |
| Initialize response | < 100ms | < 1s |
| Tool execution | < 5s | < 30s |

## Known Issues

### Current Alpha Limitations
- Browser automation requires installed browser
- Some tools may need manual intervention (2FA, CAPTCHAs)
- Rate limiting may affect rapid consecutive requests
- Windows support is experimental

### Testing Limitations
- Automated tests cannot fully test browser automation
- Manual intervention features require human testing
- Some providers may have API rate limits
- Network-dependent tests may be flaky

## Support

For testing issues:
- Check existing issues: https://github.com/tzervas/webpuppet-rs-mcp/issues
- Review test logs in `test-logs/`
- Run tests with debug logging: `RUST_LOG=debug cargo test`
- Contact: tz-dev@vectorweight.com

## References

- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Security Audit](https://rustsec.org/)
- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
