# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Enhanced `webpuppet_browser_status` to accept an optional `session_id` parameter and report detailed persistent session state.

### Changed
- Improved URL scheme validation for `webpuppet_navigate` and `webpuppet_screenshot` to reject empty or non-HTTP(S) URLs.
- Converted intervention tool operations (`pause`, `resume`, `complete`) to use write-locking on `InterventionHandler` for thread safety.
- Concurrent persistent session cleanup during MCP server shutdown via `futures::future::join_all`.

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
