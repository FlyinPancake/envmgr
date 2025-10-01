# Contributing to envmgr

Thanks for your interest in improving `envmgr`! This guide outlines how to get set up, the standards we follow, and the checks to run before you submit changes. Following these steps helps keep the project healthy and easy to maintain.

## 📜 Code of Conduct

By participating, you agree to uphold our [Contributor Covenant Code of Conduct](./CODE_OF_CONDUCT.md). Please review it before engaging in discussions or submitting contributions.

## 💡 Ways to Contribute

- **Bug reports** – Search existing issues first, then open a new issue with clear reproduction steps and environment details.
- **Feature requests** – Explain the problem you are trying to solve and how the feature would behave. Include alternatives you considered.
- **Documentation** – Improve READMEs, examples, and inline docs whenever behavior changes or clarification would help future readers.
- **Code changes** – Fix bugs, add features, improve tests, or streamline tooling.

If you are unsure whether an idea fits, feel free to open a discussion issue before starting implementation.

## 🛠️ Local Development Setup

1. Install the latest stable Rust toolchain using [`rustup`](https://rustup.rs/).
2. Clone the repository and install dependencies:
   ```fish
   git clone https://github.com/flyinpancake/envmgr.git
   cd envmgr
   cargo fetch
   ```
3. Install the formatting and linting components:
   ```fish
   rustup component add rustfmt clippy
   ```

## 🧭 Workflow Overview

1. Fork the repository and create a feature branch:
   ```fish
   git checkout -b feature/my-feature
   ```
2. Make focused commits with clear messages describing the change.
3. Keep your branch up to date with `main` to minimize merge conflicts.
4. Open a pull request once the checklist below passes.

## 🎯 Coding Standards

Please align with the guidance in [`AGENTS.md`](./AGENTS.md):

- **Imports** – Group by standard library, external crates, internal modules.
- **Error handling** – Prefer `thiserror::Error` for custom error types and return the `EnvMgrResult<T>` alias.
- **Naming** – Use `snake_case` for functions and variables, `PascalCase` for types/enums.
- **Types** – Public configuration structs should derive `Debug`, `Clone`, `serde::{Serialize, Deserialize}`, and `schemars::JsonSchema`.
- **Paths** – Use `PathBuf`/`Path` and helpers in the `config` module for file-system access.
- **Logging** – Use the `log` macros (`info!`, `error!`, etc.) instead of `println!`/`eprintln!`.
- **Docs** – Prefer `///` doc comments for public APIs.

Consistency matters more than perfection—mirror the surrounding code when in doubt.

## ✅ Required Checks

Before opening a pull request, run the following commands and ensure they succeed:

```fish
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```

For larger changes, consider adding or updating integration tests under `tests/`.

## 🧪 Testing Guidelines

- Add unit tests alongside new modules or functions.
- Expand integration tests when behavior spans multiple components (e.g., CLI interactions, environment management).
- Use the examples in `examples/simple_config/` to manually validate configuration flows when appropriate.

## 📝 Documentation Updates

- Update `README.md`, `examples/`, and inline docs when behavior or setup steps change.
- Include usage notes for new integrations, commands, or configuration fields.
- Screenshots or terminal snippets are welcome when they clarify instructions.

## 📦 Pull Request Checklist

Before submitting your PR, double-check that:

- [ ] Code is formatted (`cargo fmt --all`).
- [ ] Clippy passes with warnings treated as errors.
- [ ] Tests pass (`cargo test`) and new coverage is added where needed.
- [ ] Documentation reflects the change (README, examples, API docs).
- [ ] Commits are scoped, and the PR description links relevant issues.

Thanks again for helping make `envmgr` better. We appreciate your time and thoughtful contributions! 💚
