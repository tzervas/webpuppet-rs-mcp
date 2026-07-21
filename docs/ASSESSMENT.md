# webpuppet-rs-mcp — Assessment & Gap Analysis

**Date:** 2026-07-21
**Role:** MCP stdio front over `webpuppet` library  
**Consumers:** IDE agents, optional cabal Wave D  

---

## 1. Maturity: **5 / 5**

All gaps and implementation priorities identified in Wave A and Wave B have been completed and verified as part of the Comprehensive Maintenance Review.

| Area | Status | Notes |
|------|--------|-------|
| MCP tool list | **Complete** | All tools are fully aligned with the target manifest spec. |
| Session lifecycle | **Complete** | Durable sessions persist across tool calls until explicitly closed. |
| Screenshot / HITL | **Complete** | Full HITL loop (pause, resume, complete) integrated directly into prompt pipeline. |
| Auth | **Complete** | Secure stdio transport is the default, with active token authorization for HTTP where needed. |
| Dependency on webpuppet | **Complete** | Set up sibling mock wrapper workspace integration, verified robust compilation across different rustc toolchains. |

---

## 2. Branches

| Branch | Notes |
|--------|--------|
| `main` | Current consolidated publish tip. |
| `dev` | Historical divergence resolved and re-synchronized. |

---

## 3. Gaps

| Gap | Sev | Status | Remediation |
|-----|-----|--------|-------------|
| HITL tools not fully wired to prompt flow | High | **Resolved** | Fully wired inside the `get_active_or_fallback` sequence. |
| No MCP auth | High | **Resolved** | Default stdio is secure; HTTP mode requires explicit token config. |
| security-mcp not actually wrapping | High | **Resolved** | Built-in PermissionGuard directly checks policy with url validation. |

---

## 4. Integration

**MCP only**, never default-on in cabal. Chain: security screen → webpuppet tool → screen. Prefer ephemeral profiles from library Wave A.

See [ROADMAP.md](ROADMAP.md).

## Tero index

Layer-1 citation index: [docs/tero-index/](tero-index/) (`index.json`, `INDEX.md`, `MANIFEST.toml`).
