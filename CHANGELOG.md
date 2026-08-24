# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added optional `session_id` argument to `webpuppet_browser_status` to return active session details or a comprehensive session summary.
- Added URL scheme validation (`http://` or `https://`) for `webpuppet_navigate` and `webpuppet_screenshot`.
- Added domain permission enforcement (`require_with_url`) for `webpuppet_navigate`.
- Added active session provider mismatch validation in `webpuppet_prompt`.

### Security & Reliability
- Guaranteed server shutdown cleanup (`self.tools.shutdown()`) across all stdio exit paths in `McpServer`.
- Serialized lazy initialization of fallback browser instance using a dedicated async `Mutex` in `ToolContext`.
- Unified human-in-the-loop (HITL) pause waiting loop for persistent and ephemeral sessions without holding lock across async boundaries.

## [0.1.0-alpha.3] - 2026-01-09

### Changed
- Updated all dependencies to match webpuppet v0.1.0-alpha.3
- Changed embeddenator-webpuppet dependency from local path to git tag
- Updated tokio to 1.49.0
- Updated serde ecosystem to latest versions
- Updated async and HTTP dependencies

### Infrastructure
- Aligned with upstream webpuppet-rs v0.1.0-alpha.3 release

## [0.1.0-alpha.2] - 2024-12-20

### Added
- Initial MCP server implementation
- Support for all webpuppet providers
- Stdio transport for MCP protocol
- Permission policy configuration
- Visible browser mode option

[Unreleased]: https://github.com/tzervas/webpuppet-rs-mcp/compare/v0.1.0-alpha.3...HEAD
[0.1.0-alpha.3]: https://github.com/tzervas/webpuppet-rs-mcp/compare/v0.1.0-alpha.2...v0.1.0-alpha.3
[0.1.0-alpha.2]: https://github.com/tzervas/webpuppet-rs-mcp/releases/tag/v0.1.0-alpha.2
