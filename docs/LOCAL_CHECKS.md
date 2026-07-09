# Local checks (CI parity)

GitHub Actions workflows in this repo are **manual only** (`workflow_dispatch`).
Day-to-day quality gates run **locally** so remote CI is not the only source of truth.

## Run everything the remote job would run

```bash
./scripts/check.sh
```

Optional:

```bash
./scripts/check.sh --quick   # skip slower steps (bench/audit when applicable)
./scripts/check.sh --fix  # apply formatters instead of --check
```

## Tero index

```bash
# from a checkout that can see the generator (sibling tero-mcp recommended):
python3 ../tero-mcp/scripts/generate_lite_index.py --root "$(pwd)"
# or:
python3 scripts/generate_tero_index.sh   # if present as a thin wrapper
```

Artifacts land in `docs/tero-index/` (`index.json`, `INDEX.md`, `MANIFEST.toml`, `README.md`).

## Remote (optional)

In GitHub: **Actions → CI → Run workflow**.
