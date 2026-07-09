# webpuppet-rs-mcp — Tero Index (Layer 1)

> **Honesty:** Empirical/Declared — lite heading/line heuristic over markdown in webpuppet-rs-mcp via tero-mcp/scripts/generate_lite_index.py; source files are ground truth. Generated 2026-07-09.
> Use this index to find where to Read, not as authoritative ground truth.

- **Items:** 69
- **Flagged:** 0
- **item_tag:** `Empirical/Declared`
- **Machine index:** [`index.json`](./index.json)
- **Manifest:** [`MANIFEST.toml`](./MANIFEST.toml)

## doc (62 entries)

| Anchor | Kind | Id | Title | File:Line | Status | Summary |
|---|---|---|---|---|---|---|
| `agents` | other | — | AGENTS.md — webpuppet-rs-mcp | `AGENTS.md:2` | — | Use Tero + cabal-devmelopner for work here. |
| `agents--tero-layer-1-corpus-index` | section | — | Tero (Layer-1 corpus index) | `AGENTS.md:6` | — | Repo has docs/tero-index/index.json (generated/ refreshed via tero-mcp/scripts/generateliteindex.py). |
| `agents--agent-with-context` | other | — | agent with context: | `AGENTS.md:18` | — | uv run --project ../cabal-devmelopner cabal-devmelopner "task description here" --use-tero |
| `agents--working-with-cabal-devmelopner-agent-tool` | section | — | Working with cabal-devmelopner agent tool | `AGENTS.md:24` | — | This project is prepared for integration: |
| `agents--local-checks` | section | — | Local checks | `AGENTS.md:36` | — | Look for: |
| `agents--further-reading` | section | — | Further reading | `AGENTS.md:44` | — | - README.md |
| `contributing` | section | — | Contributing to This Project | `CONTRIBUTING.md:1` | — | Thank you for your interest in contributing! |
| `contributing--development-setup` | section | — | Development Setup | `CONTRIBUTING.md:5` | — | 1. Clone the repository |
| `contributing--pull-request-process` | section | — | Pull Request Process | `CONTRIBUTING.md:12` | — | 1. Fork the repository |
| `contributing--code-style` | section | — | Code Style | `CONTRIBUTING.md:20` | — | - Use cargo fmt for formatting |
| `contributing--license` | section | — | License | `CONTRIBUTING.md:27` | — | By contributing, you agree that your contributions will be licensed under the MIT License. |
| `readme` | other | — | webpuppet-rs-mcp | `README.md:1` | — | MCP (Model Context Protocol) server for webpuppet browser automation. |
| `readme--features` | section | — | Features | `README.md:7` | — | - MCP-compliant: Implements JSON-RPC 2.0 over stdio (standard MCP transport) |
| `readme--available-tools` | section | — | Available Tools | `README.md:16` | — | — |
| `readme--installation` | section | — | Installation | `README.md:31` | — | cargo install --path . |
| `readme--build-and-install` | other | — | Build and install | `README.md:34` | — | cargo install --path . |
| `readme--or-run-from-source` | other | — | Or run from source | `README.md:37` | — | cargo run -p webpuppet-mcp -- --stdio |
| `readme--usage-with-vs-code-github-copilot` | section | — | Usage with VS Code / GitHub Copilot | `README.md:41` | — | Add to your .vscode/mcp.json: |
| `readme--usage-with-claude-desktop` | section | — | Usage with Claude Desktop | `README.md:72` | — | Add to your claudedesktopconfig.json: |
| `readme--human-intervention-system` | section | — | Human Intervention System | `README.md:87` | — | The MCP server includes tools for human-in-the-loop workflows: |
| `readme--when-intervention-is-needed` | section | — | When Intervention is Needed | `README.md:91` | — | - Captcha: reCAPTCHA, hCaptcha, Cloudflare challenges |
| `readme--workflow` | section | — | Workflow | `README.md:98` | — | 1. Agent calls webpuppetinterventionstatus to check state |
| `readme--example` | section | — | Example | `README.md:106` | — | Agent: "I need to send a prompt to Claude but see a captcha..." |
| `readme--security-model` | section | — | Security Model | `README.md:118` | — | All operations are subject to the webpuppet permission system: |
| `readme--default-secure-policy` | section | — | Default (Secure) Policy | `README.md:122` | — | - ✅ Allowed: Navigate, ReadContent, SendPrompt, ReadResponse, NewConversation, ContinueConversation, Screenshot |
| `readme--permission-policies` | section | — | Permission Policies | `README.md:130` | — | webpuppet-mcp --policy secure |
| `readme--secure-default-blocks-destructive-operations-allows-ai-interaction` | other | — | Secure (default) - blocks destructive operations, allows AI interaction | `README.md:133` | — | webpuppet-mcp --policy secure |
| `readme--read-only-only-allows-reading-no-prompts-or-modifications` | other | — | Read-only - only allows reading, no prompts or modifications | `README.md:136` | — | webpuppet-mcp --policy readonly |
| `readme--permissive-allows-most-non-destructive-operations-use-with-caution` | other | — | Permissive - allows most non-destructive operations (use with caution) | `README.md:139` | — | webpuppet-mcp --policy permissive |
| `readme--response-screening` | section | — | Response Screening | `README.md:143` | — | All AI responses are automatically screened for: |
| `readme--example-tool-calls` | section | — | Example Tool Calls | `README.md:154` | — | { |
| `readme--send-a-prompt` | section | — | Send a Prompt | `README.md:156` | — | { |
| `readme--check-permission` | section | — | Check Permission | `README.md:174` | — | { |
| `readme--architecture` | section | — | Architecture | `README.md:205` | — | ┌─────────────────────────────────────────────────┐ |
| `readme--license` | section | — | License | `README.md:241` | — | MIT |
| `readme--status-roadmap` | section | — | Status & roadmap | `README.md:245` | — | - [Assessment & gaps](docs/ASSESSMENT.md) |
| `assessment` | note | — | webpuppet-rs-mcp — Assessment & Gap Analysis | `docs/ASSESSMENT.md:1` | — | Date: 2026-07-08 |
| `assessment--1.-maturity-2-5` | section | — | 1. Maturity: **2 / 5** | `docs/ASSESSMENT.md:9` | — | — |
| `assessment--2.-branches` | section | — | 2. Branches | `docs/ASSESSMENT.md:21` | — | — |
| `assessment--3.-gaps` | section | — | 3. Gaps | `docs/ASSESSMENT.md:31` | — | — |
| `assessment--4.-integration` | section | — | 4. Integration | `docs/ASSESSMENT.md:43` | — | MCP only, never default-on in cabal. Chain: security screen → webpuppet tool → screen. Prefer ephemeral profiles from library Wave A. |
| `assessment--tero-index` | section | — | Tero index | `docs/ASSESSMENT.md:49` | — | Layer-1 citation index: [docs/tero-index/](tero-index/) (index.json, INDEX.md, MANIFEST.toml). |
| `localchecks` | section | — | Local checks (CI parity) | `docs/LOCAL_CHECKS.md:1` | — | GitHub Actions workflows in this repo are manual only (workflowdispatch). |
| `localchecks--run-everything-the-remote-job-would-run` | section | — | Run everything the remote job would run | `docs/LOCAL_CHECKS.md:6` | — | ./scripts/check.sh |
| `localchecks--tero-index` | section | — | Tero index | `docs/LOCAL_CHECKS.md:19` | — | python3 ../tero-mcp/scripts/generateliteindex.py --root "$(pwd)" |
| `localchecks--from-a-checkout-that-can-see-the-generator-sibling-tero-mcp-recommended` | other | — | from a checkout that can see the generator (sibling tero-mcp recommended): | `docs/LOCAL_CHECKS.md:22` | — | python3 ../tero-mcp/scripts/generateliteindex.py --root "$(pwd)" |
| `localchecks--or` | other | — | or: | `docs/LOCAL_CHECKS.md:24` | — | python3 scripts/generateteroindex.sh   # if present as a thin wrapper |
| `localchecks--remote-optional` | section | — | Remote (optional) | `docs/LOCAL_CHECKS.md:30` | — | In GitHub: Actions → CI → Run workflow. |
| `roadmap` | note | — | webpuppet-rs-mcp — Product Roadmap | `docs/ROADMAP.md:1` | Living (2026-07-08) | Status: Living (2026-07-08) |
| `roadmap--waves` | section | — | Waves | `docs/ROADMAP.md:10` | — | — |
| `roadmap--wave-a-tool-honesty` | section | — | Wave A — Tool honesty | `docs/ROADMAP.md:12` | — | — |
| `roadmap--wave-b-safety` | section | — | Wave B — Safety | `docs/ROADMAP.md:21` | — | — |
| `roadmap--wave-c-release-hygiene` | section | — | Wave C — Release hygiene | `docs/ROADMAP.md:30` | — | — |
| `roadmap--api-plan-mcp-tools` | section | — | API plan — MCP tools | `docs/ROADMAP.md:40` | — | — |
| `roadmap--stable-target-set` | section | — | Stable target set | `docs/ROADMAP.md:42` | — | — |
| `roadmap--config` | section | — | Config | `docs/ROADMAP.md:60` | — | [security] |
| `roadmap--pr-plan` | section | — | PR plan | `docs/ROADMAP.md:74` | — | 1. Docs assessment + roadmap |
| `roadmap--non-goals` | section | — | Non-goals | `docs/ROADMAP.md:84` | — | - Default enable in cabal |
| `readme-2` | other | — | Tero index (Layer 1) | `docs/tero-index/README.md:1` | — | Machine + human citation index for this repository. |
| `readme--regenerate` | section | — | Regenerate | `docs/tero-index/README.md:13` | — | python3 /path/to/tero-mcp/scripts/generateliteindex.py --root $(pwd) |
| `readme--or-if-tero-mcp-is-a-sibling` | other | — | or if tero-mcp is a sibling: | `docs/tero-index/README.md:17` | — | python3 ../tero-mcp/scripts/generateliteindex.py --root $(pwd) |
| `readme--serve-locally` | section | — | Serve locally | `docs/tero-index/README.md:21` | — | export TEROTOKENS=local-dev:refresh |

## changelog (7 entries)

| Anchor | Kind | Id | Title | File:Line | Status | Summary |
|---|---|---|---|---|---|---|
| `changelog` | entry | — | Changelog | `CHANGELOG.md:1` | — | All notable changes to this project will be documented in this file. |
| `changelog--unreleased` | section | — | [Unreleased] | `CHANGELOG.md:8` | — | - Updated all dependencies to match webpuppet v0.1.0-alpha.3 |
| `changelog--0.1.0-alpha.3-2026-01-09` | section | — | [0.1.0-alpha.3] - 2026-01-09 | `CHANGELOG.md:10` | — | - Updated all dependencies to match webpuppet v0.1.0-alpha.3 |
| `changelog--changed` | section | — | Changed | `CHANGELOG.md:12` | — | - Updated all dependencies to match webpuppet v0.1.0-alpha.3 |
| `changelog--infrastructure` | section | — | Infrastructure | `CHANGELOG.md:19` | — | - Aligned with upstream webpuppet-rs v0.1.0-alpha.3 release |
| `changelog--0.1.0-alpha.2-2024-12-20` | section | — | [0.1.0-alpha.2] - 2024-12-20 | `CHANGELOG.md:22` | — | - Initial MCP server implementation |
| `changelog--added` | section | — | Added | `CHANGELOG.md:24` | — | - Initial MCP server implementation |

