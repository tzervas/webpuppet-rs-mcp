# Tero index (Layer 1)

Machine + human citation index for this repository.

| File | Purpose |
|------|---------|
| [index.json](./index.json) | Consumed by `tero-mcp-lite --index …` |
| [INDEX.md](./INDEX.md) | Human/agent table of anchors |
| [MANIFEST.toml](./MANIFEST.toml) | Generator metadata / regenerate command |

**Honesty:** Empirical/Declared heading heuristic — open the cited `file:line`, do not treat summaries as ground truth.

## Regenerate

```bash
python3 /path/to/tero-mcp/scripts/generate_lite_index.py --root $(pwd)
# or if tero-mcp is a sibling:
python3 ../tero-mcp/scripts/generate_lite_index.py --root $(pwd)
```

## Serve locally

```bash
export TERO_TOKENS=local-dev:refresh
uv run --project ../tero-mcp tero-mcp-lite --index docs/tero-index/index.json
```
