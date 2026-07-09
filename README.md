# webpuppet-rs-mcp

MCP (Model Context Protocol) server for webpuppet browser automation.

This crate provides a standards-compliant MCP server that exposes webpuppet functionality as tools for AI assistants like GitHub Copilot, Claude Desktop, and other MCP-compatible clients.

## Features

- **MCP-compliant**: Implements JSON-RPC 2.0 over stdio (standard MCP transport)
- **Tool exposure**: Exposes AI prompting, screenshot, and research capabilities
- **Security guardrails**: Inherits webpuppet's permission system
- **Response screening**: Filters prompt injections and malicious content
- **Browser detection**: Automatically finds Brave/Chrome/Chromium
- **Human intervention**: Pause/resume for captchas, 2FA, and manual steps

## Available Tools

| Tool | Description |
|------|-------------|
| `webpuppet_prompt` | Send a prompt through browser automation (providers + tools) |
| `webpuppet_screenshot` | Take screenshots of web pages |
| `webpuppet_list_providers` | List available AI providers |
| `webpuppet_provider_capabilities` | Get declared capabilities for a provider/tool |
| `webpuppet_detect_browsers` | Detect installed browsers |
| `webpuppet_check_permission` | Check if an operation is allowed |
| `webpuppet_intervention_status` | Check if human intervention is needed |
| `webpuppet_intervention_complete` | Signal that intervention is done |
| `webpuppet_pause` | Pause automation for manual interaction |
| `webpuppet_resume` | Resume automation after pause |

## Installation

```bash
# Build and install
cargo install --path .

# Or run from source
cargo run -p webpuppet-mcp -- --stdio
```

## Usage with VS Code / GitHub Copilot

Add to your `.vscode/mcp.json`:

```json
{
  "servers": {
    "webpuppet": {
      "command": "webpuppet-mcp",
      "args": ["--stdio"],
      "env": {}
    }
  }
}
```

Or if running from cargo:

```json
{
  "servers": {
    "webpuppet": {
      "command": "cargo",
      "args": ["run", "-p", "webpuppet-mcp", "--", "--stdio"],
      "cwd": "/path/to/webpuppet",
      "env": {}
    }
  }
}
```

## Usage with Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "webpuppet": {
      "command": "webpuppet-mcp",
      "args": ["--stdio"]
    }
  }
}
```

## Human Intervention System

The MCP server includes tools for human-in-the-loop workflows:

### When Intervention is Needed

- **Captcha**: reCAPTCHA, hCaptcha, Cloudflare challenges
- **Two-Factor Auth**: SMS codes, TOTP, email verification
- **Login**: Session expired, auth required
- **Rate Limits**: Too many requests

### Workflow

1. Agent calls `webpuppet_intervention_status` to check state
2. If intervention needed, agent notifies user
3. User completes manual task in visible browser
4. User/agent calls `webpuppet_intervention_complete` with `success=true`
5. Automation resumes

### Example

```
Agent: "I need to send a prompt to Claude but see a captcha..."
Agent: [calls webpuppet_intervention_status]
Agent: "⚠️ A captcha is displayed. Please complete it in the browser."
User: [solves captcha manually]
User: "Done!"
Agent: [calls webpuppet_intervention_complete with success=true]
Agent: "Thank you! Continuing..."
```

## Security Model

All operations are subject to the webpuppet permission system:

### Default (Secure) Policy

- ✅ **Allowed**: Navigate, ReadContent, SendPrompt, ReadResponse, NewConversation, ContinueConversation, Screenshot
- ❌ **Blocked**: DeleteAccount, ChangePassword, ModifyPayment, RevokeTokens, FileSystemAccess, etc.
- 🌐 **Domains**: Only AI provider domains (claude.ai, x.com, gemini.google.com)
- 🌐 **HTTPS-only**: In secure mode, `http://` URLs are denied
- ⚠️ **Risk Threshold**: Max risk level 5 (out of 10)

### Permission Policies

```bash
# Secure (default) - blocks destructive operations, allows AI interaction
webpuppet-mcp --policy secure

# Read-only - only allows reading, no prompts or modifications
webpuppet-mcp --policy readonly

# Permissive - allows most non-destructive operations (use with caution)
webpuppet-mcp --policy permissive
```

## Response Screening

All AI responses are automatically screened for:

- **Invisible text**: Zero-width characters, 1pt fonts
- **Prompt injections**: "Ignore previous instructions" patterns
- **Encoded payloads**: Base64, hex encoded content
- **Hidden elements**: CSS display:none, opacity:0

If screening detects issues, the response is sanitized and a warning is included.

## Example Tool Calls

### Send a Prompt

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "webpuppet_prompt",
    "arguments": {
      "provider": "claude",
      "message": "Explain how io_uring works in Rust",
      "context": "Focus on memory safety"
    }
  }
}
```

### Check Permission

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "webpuppet_check_permission",
    "arguments": {
      "operation": "DeleteAccount"
    }
  }
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [{
      "type": "text",
      "text": "# Permission Check\n\n**Operation**: `DeleteAccount`\n**Status**: ❌ DENIED\n**Reason**: Operation explicitly denied by policy\n**Risk Level**: 10/10"
    }],
    "isError": false
  }
}
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                MCP Client                        │
│  (VS Code, Claude Desktop, etc.)                 │
└─────────────────────┬───────────────────────────┘
                      │ JSON-RPC 2.0 / stdio
┌─────────────────────▼───────────────────────────┐
│            webpuppet-mcp Server                  │
│  ┌───────────────────────────────────────────┐  │
│  │            Permission Guard               │  │
│  │  - Operation allowlist/blocklist         │  │
│  │  - Domain restrictions                    │  │
│  │  - Risk level enforcement                 │  │
│  │  - Audit logging                          │  │
│  └───────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────┐  │
│  │            Tool Registry                  │  │
│  │  - webpuppet_prompt                       │  │
│  │  - webpuppet_screenshot                   │  │
│  │  - webpuppet_list_providers               │  │
│  │  - webpuppet_detect_browsers              │  │
│  │  - webpuppet_check_permission             │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────┬───────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────┐
│         webpuppet                   │
│  - Browser automation (Brave/Chrome)             │
│  - AI provider integration                       │
│  - Content screening                             │
│  - Session management                            │
└─────────────────────────────────────────────────┘
```

## License

MIT

## Status & roadmap

- [Assessment & gaps](docs/ASSESSMENT.md)
- [Product roadmap & API plans](docs/ROADMAP.md)
