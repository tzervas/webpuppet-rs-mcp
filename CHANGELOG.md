# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added optional `session_id` filter and status reporting to `webpuppet_browser_status`.

### Security
- Enforced URL scheme validation (`http://` / `https://`) in `webpuppet_navigate` and `webpuppet_screenshot`.
- Enforced domain-specific permission checks via `require_with_url` in `webpuppet_navigate`.

### Reliability & Performance
- Serialized fallback `WebPuppet` lazy initialization using an async mutex to prevent concurrent browser launches.
- Concurrent active session cleanup during server shutdown using `futures::future::join_all`.
- Guaranteed server cleanup execution in stdio transport loop to prevent background process leaks.
- Acquired write locks on intervention handler state mutations.

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
