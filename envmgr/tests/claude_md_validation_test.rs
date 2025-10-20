//! Validation tests for CLAUDE.md documentation
//! 
//! These tests ensure that the CLAUDE.md file:
//! - Contains required sections
//! - References actual code structures correctly
//! - Has consistent formatting
//! - Documents existing commands and features

use std::fs;
use std::path::Path;

fn get_claude_md_content() -> String {
    let claude_md_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("CLAUDE.md");
    fs::read_to_string(claude_md_path).expect("Failed to read CLAUDE.md")
}

#[test]
fn test_claude_md_exists() {
    let claude_md_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("CLAUDE.md");
    assert!(claude_md_path.exists(), "CLAUDE.md should exist in repository root");
}

#[test]
fn test_claude_md_has_required_sections() {
    let content = get_claude_md_content();
    
    // Check for major sections
    assert!(content.contains("# CLAUDE.md"), "Should have main header");
    assert!(content.contains("## Project Overview"), "Should have Project Overview section");
    assert!(content.contains("## Development Commands"), "Should have Development Commands section");
    assert!(content.contains("## Architecture Overview"), "Should have Architecture Overview section");
    assert!(content.contains("### Core Data Flow"), "Should document core data flow");
    assert!(content.contains("### Key Components"), "Should document key components");
    assert!(content.contains("## Important Implementation Details"), "Should have implementation details");
    assert!(content.contains("## Testing Strategy"), "Should document testing strategy");
}

#[test]
fn test_claude_md_documents_test_commands() {
    let content = get_claude_md_content();
    
    // Should document how to run tests
    assert!(content.contains("cargo test"), "Should mention cargo test");
    assert!(content.contains("--lib"), "Should mention library tests");
    assert!(content.contains("--test integration_tests"), "Should mention integration tests");
}

#[test]
fn test_claude_md_documents_build_commands() {
    let content = get_claude_md_content();
    
    assert!(content.contains("cargo build"), "Should mention cargo build");
    assert!(content.contains("cargo run"), "Should mention cargo run");
    assert!(content.contains("--release"), "Should mention release builds");
}

#[test]
fn test_claude_md_documents_code_quality_tools() {
    let content = get_claude_md_content();
    
    assert!(content.contains("cargo fmt"), "Should mention cargo fmt");
    assert!(content.contains("cargo clippy"), "Should mention cargo clippy");
}

#[test]
fn test_claude_md_documents_core_components() {
    let content = get_claude_md_content();
    
    // Key modules and components
    assert!(content.contains("EnvironmentManager"), "Should document EnvironmentManager");
    assert!(content.contains("State"), "Should document State system");
    assert!(content.contains("Integration"), "Should document Integration system");
    assert!(content.contains("Fish"), "Should document Fish shell integration");
}

#[test]
fn test_claude_md_documents_file_paths() {
    let content = get_claude_md_content();
    
    assert!(content.contains("~/.config/envmgr"), "Should document config directory");
    assert!(content.contains("~/.local/state/envmgr"), "Should document state directory");
    assert!(content.contains("config.yaml"), "Should mention config files");
}

#[test]
fn test_claude_md_documents_error_handling() {
    let content = get_claude_md_content();
    
    assert!(content.contains("EnvMgrError") || content.contains("error"), "Should mention error handling");
    assert!(content.contains("EnvMgrResult") || content.contains("Result"), "Should mention result types");
}

#[test]
fn test_claude_md_documents_commands() {
    let content = get_claude_md_content();
    
    // Should document available commands
    assert!(content.contains("init") || content.contains("Init"), "Should document init command");
    assert!(content.contains("add") || content.contains("Add"), "Should document add command");
    assert!(content.contains("list") || content.contains("List"), "Should document list command");
    assert!(content.contains("switch") || content.contains("Switch"), "Should document switch command");
    assert!(content.contains("use") || content.contains("Use"), "Should document use command");
}

#[test]
fn test_claude_md_documents_integrations() {
    let content = get_claude_md_content();
    
    assert!(content.contains("GitHub CLI") || content.contains("gh_cli"), "Should document GitHub CLI integration");
    assert!(content.contains("1Password") || content.contains("op_ssh"), "Should document 1Password integration");
    assert!(content.contains("Tailscale") || content.contains("tailscale"), "Should document Tailscale integration");
}

#[test]
fn test_claude_md_has_security_considerations() {
    let content = get_claude_md_content();
    
    assert!(content.contains("Security") || content.contains("security"), "Should have security section");
}

#[test]
fn test_claude_md_has_directory_conventions() {
    let content = get_claude_md_content();
    
    assert!(content.contains("Directory") || content.contains("directory"), "Should document directory structure");
    assert!(content.contains("XDG"), "Should mention XDG directories");
}

#[test]
fn test_claude_md_documents_state_persistence() {
    let content = get_claude_md_content();
    
    assert!(content.contains("state.yaml") || content.contains("State"), "Should document state persistence");
    assert!(content.contains("persist") || content.contains("store"), "Should mention state storage");
}

#[test]
fn test_claude_md_documents_environment_structure() {
    let content = get_claude_md_content();
    
    assert!(content.contains("base") || content.contains("Base"), "Should document base environment");
    assert!(content.contains("environments/"), "Should document environment directory structure");
    assert!(content.contains("files/"), "Should document files directory");
}

#[test]
fn test_claude_md_line_count_reasonable() {
    let content = get_claude_md_content();
    let line_count = content.lines().count();
    
    // Should be substantial documentation (at least 50 lines)
    assert!(line_count >= 50, "CLAUDE.md should have at least 50 lines of documentation, found {}", line_count);
    
    // But not excessively long (less than 1000 lines is reasonable)
    assert!(line_count < 1000, "CLAUDE.md should be concise (< 1000 lines), found {}", line_count);
}

#[test]
fn test_claude_md_has_code_examples() {
    let content = get_claude_md_content();
    
    // Should have code blocks
    assert!(content.contains("```"), "Should have code examples in markdown code blocks");
    
    // Count code blocks
    let code_block_count = content.matches("```").count();
    assert!(code_block_count >= 4, "Should have at least 2 code blocks (pairs), found {} markers", code_block_count);
    assert!(code_block_count % 2 == 0, "Code blocks should be properly closed (even number of markers)");
}

#[test]
fn test_claude_md_documents_dialoguer() {
    let content = get_claude_md_content();
    
    // Should document the interactive prompts added in this feature
    assert!(content.contains("dialoguer") || content.contains("interactive"), "Should mention interactive prompts");
}

#[test]
fn test_claude_md_markdown_headers_formatted_correctly() {
    let content = get_claude_md_content();
    
    // Check that headers use proper markdown format
    for line in content.lines() {
        if line.starts_with('#') {
            // Headers should have space after #
            if line.len() > 1 {
                assert!(
                    line.starts_with("# ") || line.starts_with("##") && line.chars().nth(line.chars().take_while(|&c| c == '#').count()).unwrap_or(' ') == ' ',
                    "Header should have space after # symbols: '{}'",
                    line
                );
            }
        }
    }
}

#[test]
fn test_claude_md_no_broken_internal_references() {
    let content = get_claude_md_content();
    
    // Check for common file references that should exist
    if content.contains("error.rs") {
        let error_rs_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/error.rs");
        assert!(error_rs_path.exists(), "error.rs referenced in CLAUDE.md should exist");
    }
    
    if content.contains("main.rs") {
        let main_rs_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
        assert!(main_rs_path.exists(), "main.rs referenced in CLAUDE.md should exist");
    }
    
    if content.contains("cli.rs") {
        let cli_rs_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli.rs");
        assert!(cli_rs_path.exists(), "cli.rs referenced in CLAUDE.md should exist");
    }
}

#[test]
fn test_claude_md_documents_workspace_structure() {
    let content = get_claude_md_content();
    
    assert!(content.contains("workspace") || content.contains("Workspace"), "Should document workspace structure");
    assert!(content.contains("Cargo.toml"), "Should mention Cargo.toml files");
}

#[test]
fn test_claude_md_mentions_platform_support() {
    let content = get_claude_md_content();
    
    assert!(content.contains("Linux") || content.contains("platform"), "Should document platform support");
    assert!(content.contains("Fish") || content.contains("fish"), "Should document Fish shell support");
}

#[test]
fn test_claude_md_documents_adding_new_commands() {
    let content = get_claude_md_content();
    
    // Should provide guidance on adding new commands
    if content.contains("Adding New Commands") || content.contains("add") {
        assert!(content.contains("commands/"), "Should reference commands directory");
    }
}

#[test]
fn test_claude_md_documents_adding_integrations() {
    let content = get_claude_md_content();
    
    // Should provide guidance on adding new integrations
    if content.contains("Integration") {
        assert!(content.contains("integrations/") || content.contains("plugin"), "Should reference integrations system");
    }
}

#[test]
fn test_claude_md_consistent_terminology() {
    let content = get_claude_md_content();
    
    // Check for consistent terminology (not mixing different terms for same concept)
    let env_count = content.matches("environment").count();
    let config_count = content.matches("configuration").count();
    
    // Should use terms frequently (good documentation)
    assert!(env_count > 5, "Should frequently mention 'environment'");
    assert!(config_count > 3, "Should mention 'configuration'");
}

#[test]
fn test_claude_md_no_trailing_whitespace_on_headers() {
    let content = get_claude_md_content();
    
    for line in content.lines() {
        if line.starts_with('#') {
            assert_eq!(line.trim_end(), line, "Headers should not have trailing whitespace: '{}'", line);
        }
    }
}

#[test]
fn test_claude_md_documents_toml_vs_yaml() {
    let content = get_claude_md_content();
    
    // Should clarify the TOML vs YAML situation mentioned in the architecture
    assert!(content.contains("TOML") || content.contains("toml"), "Should mention TOML");
    assert!(content.contains("YAML") || content.contains("yaml"), "Should mention YAML");
}

#[test]
fn test_claude_md_not_empty() {
    let content = get_claude_md_content();
    assert!(!content.is_empty(), "CLAUDE.md should not be empty");
    assert!(content.len() > 100, "CLAUDE.md should have substantial content");
}