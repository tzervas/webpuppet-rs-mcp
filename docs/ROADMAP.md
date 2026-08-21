# webpuppet-rs-mcp — Product Roadmap

**Status:** Living (2026-07-08)  
**North star:** Thin, honest MCP façade over webpuppet-rs — durable sessions, real HITL, security hooks, no overclaimed tools.

Companion: [ASSESSMENT.md](ASSESSMENT.md).

---

## Waves

### Wave A — Tool honesty

| ID | Work | Status |
|----|------|--------|
| M-A1 | Audit each tool: implement or remove from manifest | Done |
| M-A2 | Screenshot real path or delete | Done |
| M-A3 | HITL: intervention_* actually pause prompt pipeline | Done |
| M-A4 | Session object persists across tools until `close` | Done |

### Wave B — Safety

| ID | Work |
|----|------|
| M-B1 | Inherit library ephemeral profile defaults |
| M-B2 | Optional security-mcp pre/post hooks (config) |
| M-B3 | Permission check tools reflect library hardwire |
| M-B4 | stdio-only default; HTTP only with token |

### Wave C — Release hygiene

| ID | Work |
|----|------|
| M-C1 | Single `dev` → `main` train; retire stale release/* |
| M-C2 | Lockstep version with webpuppet crate |
| M-C3 | MCP e2e tests with mocked browser where possible |

---

## API plan — MCP tools

### Stable target set

| Tool | Purpose | Args (core) |
|------|---------|-------------|
| `webpuppet_session_open` | Create session | `profile?`, `permissions?` |
| `webpuppet_session_close` | Destroy | `session_id` |
| `webpuppet_navigate` | Go to URL | `session_id`, `url` |
| `webpuppet_screenshot` | Capture | `session_id`, `path?` |
| `webpuppet_extract` | Text/CSS | `session_id`, `selector` |
| `webpuppet_prompt` | Provider prompt flow | `session_id`, `provider`, `prompt` |
| `webpuppet_list_providers` | Inventory | — |
| `webpuppet_check_permission` | Query allow | `action`, `target` |
| `webpuppet_intervention_status` | HITL | `session_id` |
| `webpuppet_intervention_complete` | Resume | `session_id` |
| `webpuppet_pause` / `webpuppet_resume` | Manual | `session_id` |

Deprecate free-floating tools that imply global singleton browser without `session_id`.

### Config

```toml
[security]
mode = "off" | "mcp"
security_mcp_command = ["security-mcp", "--stdio"]

[browser]
profile = "ephemeral"
host_allowlist = ["example.com"]
```

---

## PR plan

1. Docs assessment + roadmap  
2. Session_id everywhere  
3. HITL wiring  
4. Security hooks  
5. Branch train cleanup + 0.2.0  

---

## Non-goals

- Default enable in cabal  
- Unauthenticated network bind  
- ToS bypass features  
