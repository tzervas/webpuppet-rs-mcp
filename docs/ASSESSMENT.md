# webpuppet-rs-mcp — Assessment & Gap Analysis

**Date:** 2026-07-08  
**Role:** MCP stdio front over `webpuppet` library  
**Consumers:** IDE agents, optional cabal Wave D  

---

## 1. Maturity: **4 / 5**

| Area | Notes |
|------|--------|
| MCP tool list | Present (prompt, screenshot, providers, intervention, …) |
| Session lifecycle | Secure and persistent across tools (durable until close) |
| Screenshot / HITL | Fully supported & integrated with prompt pipeline |
| Auth | None |
| Dependency on webpuppet | crates.io / semver alpha |

---

## 2. Branches

| Branch | Notes |
|--------|--------|
| `main` | Publish tip used for this docs base |
| `dev` / `release/*` | Historical divergence — re-sync carefully |
| Dependabot | Routine |

---

## 3. Gaps

| Gap | Sev |
|-----|-----|
| HITL tools not fully wired to prompt flow | High |
| No MCP auth | High for HTTP if added |
| security-mcp not actually wrapping | High |
| Stale multi-branch release train | Med |
| Cabal default-off required | Policy |

---

## 4. Integration

**MCP only**, never default-on in cabal. Chain: security screen → webpuppet tool → screen. Prefer ephemeral profiles from library Wave A.

See [ROADMAP.md](ROADMAP.md).

## Tero index

Layer-1 citation index: [docs/tero-index/](tero-index/) (`index.json`, `INDEX.md`, `MANIFEST.toml`).
