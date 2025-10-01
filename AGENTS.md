# Agent Guidelines for envmgr

## Build, Lint, Test Commands
- Build: `cargo build`
- Test all: `cargo test`
- Test single: `cargo test <test_name>`
- Lint: `cargo clippy -- -D warnings`
- Format check: `cargo fmt --all -- --check`
- Format: `cargo fmt --all`

## Code Style
- **Imports**: Standard library first, then external crates (workspace deps), then internal modules
- **Error Handling**: Use `thiserror::Error` for custom errors, return `EnvMgrResult<T>` type alias
- **Naming**: Snake_case for functions/variables, PascalCase for types/structs/enums
- **Types**: Use explicit type annotations for public APIs, derive `Debug, Clone, serde::{Serialize, Deserialize}, schemars::JsonSchema` for config structs
- **TOML Formatting**: Use taplo settings (2-space indent, align entries/tables/arrays, trailing commas, reorder keys)
- **Logging**: Use `log` crate macros (`info!`, `error!`, etc.), not println/eprintln for diagnostics
- **File Paths**: Use `std::path::PathBuf` and `Path`, helper functions in `config` module for config directories
- **Rust Edition**: 2024 edition features enabled
- **Comments**: Prefer doc comments (`///`) for public APIs, use `//` sparingly for complex logic only
