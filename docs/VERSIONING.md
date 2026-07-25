# Versioning and releases

## The policy

`embeddenator-webpuppet-mcp` is versioned **0.x.y** and stays there. Commitizen enforces this with
`major_version_zero = true` in [`.cz.toml`](../.cz.toml). Moving to **1.x.x requires an explicit
human authorization** — full production readiness, hardening, and a maintainer decision. **No
agent, and no automation, may cut or propose a 1.x.x release.**

## Under `major_version_zero`, MINOR is the breaking position

This is the detail most often got wrong. While the major is pinned at 0:

| Change                        | Bump      | Example           |
| ----------------------------- | --------- | ----------------- |
| `fix:`                        | PATCH     | 0.1.5 → 0.1.6     |
| `feat:`                       | PATCH     | 0.1.5 → 0.1.6     |
| `feat!:` / `BREAKING CHANGE:` | **MINOR** | 0.1.5 → **0.2.0** |

A consumer pinning "latest compatible" therefore pins the **minor** — `"0.1"`, or a moving tag
`v0.1`. Never `"1"`, and never a bare `v1` tag: under this scheme `0.1` and `0.2` are
*incompatible* releases, exactly as `1.x` and `2.x` would be after a 1.0 cut.

With `major_version_zero` **absent**, commitizen treats a breaking change as MAJOR and mints
`1.0.0` on the first `feat!:` — a version nobody authorized.

## Version files

[`.cz.toml`](../.cz.toml) lists every place the version appears under `version_files`, so
`cz bump` moves them together and they cannot drift:

- `Cargo.toml` — `[package] version`
- `.cz.toml` itself — `version`

Do not hand-edit any of these — run the tool:

```bash
cz bump --yes --dry-run     # show what would happen, change nothing
cz bump                     # move every version file + create the tag
cz version --project        # what this project currently claims to be
```

The `version` key in `.cz.toml` is the version cz bumps *from*, so it must track the newest
released tag. **This repository has already been bitten by that**: `Cargo.toml` sat at
`0.1.0-alpha.3` while `v0.1.4-alpha` and `v0.1.5-alpha` were already tagged, so a `cz bump` would
have re-minted a version that already had a tag.

## Tag history — read before trusting the tag list

The tags here are **not in date order**, and the newest tag is not the highest version:

| Tag | Created |
| --- | --- |
| `v0.1.0` | 2026-07-09 |
| `v0.1.5-alpha` | 2026-01-10 |
| `v0.1.4-alpha` | 2026-01-10 |

`v0.1.0` was cut **six months after** `v0.1.5-alpha`, so "latest tag" and "highest version"
disagree. Ordering by semver, the highest released version is **`0.1.5-alpha`**, and that is what
`.cz.toml` is pinned to — versions never move backwards.

Treat `v0.1.0` as a stray. Retiring or annotating it is a maintainer decision.

## The `-alpha` suffix: read this before your first `cz bump`

Under `version_scheme = "semver"`, commitizen reads `0.1.5-alpha` and normalises the spelling:

```
$ cz version --project
0.1.5-a0
$ cargo metadata --no-deps --format-version 1 | jq -r '.packages[]|"\(.name) \(.version)"'
embeddenator-webpuppet-mcp 0.1.5-alpha
```

Same version, two spellings for the prerelease identifier.

More importantly, **a plain `cz bump` finalises the prerelease** — a prerelease sorts *before* its
release, so any increment resolves to plain `0.1.5`, not to another alpha. To stay on the alpha
line pass `--prerelease alpha`, which produces `0.1.5-a1` — note that is **not** the `-alpha`
spelling the existing tags use.

**Recommendation:** finalise off the prerelease line at the next release and drop `-alpha`
entirely. `0.x` already communicates "no compatibility promise"; the suffix adds nothing and means
fighting the tool on every bump. Maintainer decision, not an agent one.

## A GitHub Release is not a registry publication

These are two different things:

- **A git tag / GitHub Release** is a marker plus notes. It publishes nothing consumable.
- **A crates.io publication** is the artifact dependents actually resolve.

The crate name `embeddenator-webpuppet-mcp` has **never been published to crates.io**. Every tag
in the list above is source-only. When you claim a version is released, say *where*.

## Release steps

1. Land work on `dev` via a work branch — never straight to `main`.
2. `cz bump` on the release branch: this moves every version file and creates the tag locally.
3. Open the release PR `dev` → `main`. Merge with a **merge commit**, never a squash.
4. Push the tag; the `release` workflow (`workflow_dispatch`) builds the GitHub Release.
5. **Publishing to crates.io is a separate, deliberate step.** It is not automatic, and until it
   runs, the version is not released to consumers.
