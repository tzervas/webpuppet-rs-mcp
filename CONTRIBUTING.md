# Contributing to This Project

Thank you for your interest in contributing!

## Development Setup

1. Clone the repository
2. Ensure you have Rust 1.84+ installed
3. Run `cargo build` to build
4. Run `cargo test` to run tests

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## Code Style

- Use `cargo fmt` for formatting
- No clippy warnings (`cargo clippy -- -D warnings`)
- Add tests for new functionality
- Update documentation as needed

## Versioning and releases

Full rules: **[docs/VERSIONING.md](docs/VERSIONING.md)**. The short version:

- Conventional Commits, enforced by commitizen ([`.cz.toml`](.cz.toml)).
- This crate is **0.x.y** and stays there. `major_version_zero = true`. **No agent may cut or
  propose a 1.x.x release** — that needs explicit human authorization.
- **MINOR is the breaking position** while the major is 0: `feat!:` takes 0.1.5 → **0.2.0**, not
  1.0.0. Consumers pin the minor (`"0.1"` / `v0.1`), never `v1`.
- Never hand-edit a version. `cz bump` moves every file listed in `version_files` together.
- The tag list is **not in date order** — `v0.1.0` was cut after `v0.1.5-alpha`. Highest version
  wins, not newest tag.
- A GitHub Release is **not** a crates.io publication. This crate has never been published.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
