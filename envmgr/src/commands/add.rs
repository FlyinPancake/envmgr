use std::fs;

use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use log::info;

use crate::config::EnvironmentConfig;
use crate::error::{EnvMgrError, EnvMgrResult};
use crate::integrations::gh_cli::{GhCliConfig, GhCliHostUser};
use crate::integrations::one_password_ssh_agent::{OnePasswordSSHAgentConfig, OnePasswordSSHKey};
use crate::integrations::tailscale::TailscaleConfig;

/// Convert a string to a filesystem-safe slug
/// e.g., "My Work Environment" -> "my-work-environment"
fn slugify(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                // Skip other characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        // Remove consecutive dashes
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate that a key is filesystem-safe
fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Key cannot be empty".to_string());
    }

    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Key must contain only alphanumeric characters, hyphens, or underscores".to_string());
    }

    if key == "base" {
        return Err("Key 'base' is reserved".to_string());
    }

    Ok(())
}

/// Prompt for GitHub CLI configuration
fn prompt_gh_cli_config() -> EnvMgrResult<Option<GhCliConfig>> {
    let configure = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Configure GitHub CLI integration?")
        .default(false)
        .interact()?;

    if !configure {
        return Ok(None);
    }

    let host: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GitHub host")
        .default("github.com".to_string())
        .interact_text()?;

    let user: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GitHub username")
        .interact_text()?;

    Ok(Some(GhCliConfig {
        hosts: vec![GhCliHostUser { host, user }],
    }))
}

/// Prompt for 1Password SSH Agent configuration
fn prompt_op_ssh_config() -> EnvMgrResult<Option<OnePasswordSSHAgentConfig>> {
    let configure = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Configure 1Password SSH Agent integration?")
        .default(false)
        .interact()?;

    if !configure {
        return Ok(None);
    }

    // For now, create an empty config. User can manually add SSH keys to config.yaml later
    // or we could add an interactive loop to add multiple SSH keys
    let add_keys = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add SSH keys now?")
        .default(false)
        .interact()?;

    let mut keys = vec![];

    if add_keys {
        loop {
            let vault: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Vault name (leave empty to skip)")
                .allow_empty(true)
                .interact_text()?;

            let item: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Item name (leave empty to skip)")
                .allow_empty(true)
                .interact_text()?;

            let account: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Account (leave empty to skip)")
                .allow_empty(true)
                .interact_text()?;

            keys.push(OnePasswordSSHKey {
                vault: if vault.is_empty() { None } else { Some(vault) },
                item: if item.is_empty() { None } else { Some(item) },
                account: if account.is_empty() { None } else { Some(account) },
            });

            let add_more = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Add another SSH key?")
                .default(false)
                .interact()?;

            if !add_more {
                break;
            }
        }
    }

    Ok(Some(OnePasswordSSHAgentConfig { keys }))
}

/// Prompt for Tailscale configuration
fn prompt_tailscale_config() -> EnvMgrResult<Option<TailscaleConfig>> {
    let configure = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Configure Tailscale integration?")
        .default(false)
        .interact()?;

    if !configure {
        return Ok(None);
    }

    let tailnet: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Tailscale tailnet")
        .with_initial_text("example.com")
        .interact_text()?;

    Ok(Some(TailscaleConfig { tailnet }))
}

/// Main add command implementation
pub fn add_environment(name_arg: &str) -> EnvMgrResult<()> {
    info!("Starting interactive environment creation");

    // If a name was provided as argument, use it; otherwise prompt
    let env_name: String = if !name_arg.is_empty() {
        name_arg.to_string()
    } else {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Environment name")
            .interact_text()?
    };

    // Generate a suggested key from the name
    let suggested_key = slugify(&env_name);

    let env_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Environment key (filesystem-safe identifier)")
        .default(suggested_key)
        .validate_with(|input: &String| -> Result<(), String> {
            validate_key(input)
        })
        .interact_text()?;

    // Check if environment already exists
    let env_dir = EnvironmentConfig::get_env_dir_by_key(&env_key);
    if env_dir.exists() {
        return Err(EnvMgrError::AlreadyExists(format!(
            "Environment '{}' already exists at {}",
            env_key,
            env_dir.display()
        )));
    }

    // Prompt for integrations
    info!("Prompting for integration configurations");

    let gh_cli = prompt_gh_cli_config()?;
    let op_ssh = prompt_op_ssh_config()?;
    let tailscale = prompt_tailscale_config()?;

    // Create the environment config
    let env_config = EnvironmentConfig {
        name: env_name.clone(),
        env_vars: vec![],
        op_ssh,
        gh_cli,
        tailscale,
    };

    // Create directory structure
    info!("Creating directory structure at {}", env_dir.display());
    fs::create_dir_all(&env_dir)?;

    let files_dir = env_dir.join("files");
    fs::create_dir_all(&files_dir)?;

    // Write config file
    let config_path = env_dir.join("config.yaml");
    let config_yaml = serde_norway::to_string(&env_config)?;
    fs::write(&config_path, config_yaml)?;

    info!("Environment '{}' created successfully", env_key);

    eprintln!("\nEnvironment '{}' created successfully!", env_name);
    eprintln!("  Location: {}", env_dir.display());
    eprintln!("  Config: {}", config_path.display());
    eprintln!("  Files directory: {}", files_dir.display());
    eprintln!("\nTo switch to this environment, run:");
    eprintln!("  envmgr switch {}", env_key);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============= Slugify Tests =============
    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("My Work"), "my-work");
        assert_eq!(slugify("Personal 123"), "personal-123");
    }

    #[test]
    fn test_slugify_consecutive_separators() {
        assert_eq!(slugify("Test__Environment"), "test-environment");
        assert_eq!(slugify("Test---Environment"), "test-environment");
        assert_eq!(slugify("Test_-_Environment"), "test-environment");
    }

    #[test]
    fn test_slugify_whitespace() {
        assert_eq!(slugify("  spaces  "), "spaces");
        assert_eq!(slugify("   multiple   spaces   "), "multiple-spaces");
        assert_eq!(slugify("\ttab\tseparated\t"), "tab-separated");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Special!@#Chars"), "specialchars");
        assert_eq!(slugify("Test$%^&*()"), "test");
        assert_eq!(slugify("Hello.World"), "helloworld");
        assert_eq!(slugify("Foo/Bar\\Baz"), "foobarbaz");
    }

    #[test]
    fn test_slugify_mixed_case() {
        assert_eq!(slugify("CamelCase"), "camelcase");
        assert_eq!(slugify("UPPERCASE"), "uppercase");
        assert_eq!(slugify("MiXeD_CaSe"), "mixed-case");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(slugify("123"), "123");
        assert_eq!(slugify("Test 123 Environment"), "test-123-environment");
        assert_eq!(slugify("v1.2.3"), "v123");
    }

    #[test]
    fn test_slugify_unicode() {
        assert_eq!(slugify("Café"), "caf");
        assert_eq!(slugify("héllo wörld"), "hllo-wrld");
        assert_eq!(slugify("Test™"), "test");
    }

    #[test]
    fn test_slugify_empty_and_whitespace_only() {
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("   "), "");
        assert_eq!(slugify("\t\n\r"), "");
    }

    #[test]
    fn test_slugify_leading_trailing_dashes() {
        assert_eq!(slugify("-leading"), "leading");
        assert_eq!(slugify("trailing-"), "trailing");
        assert_eq!(slugify("-both-"), "both");
        assert_eq!(slugify("_underscore_"), "underscore");
    }

    #[test]
    fn test_slugify_real_world_examples() {
        assert_eq!(slugify("My Work Environment"), "my-work-environment");
        assert_eq!(slugify("Personal (Home)"), "personal-home");
        assert_eq!(slugify("Client ABC - Project XYZ"), "client-abc-project-xyz");
        assert_eq!(slugify("Dev & Test"), "dev-test");
    }

    // ============= Validate Key Tests =============
    #[test]
    fn test_validate_key_valid_simple() {
        assert!(validate_key("work").is_ok());
        assert!(validate_key("personal").is_ok());
        assert!(validate_key("dev").is_ok());
    }

    #[test]
    fn test_validate_key_valid_with_separators() {
        assert!(validate_key("my-work").is_ok());
        assert!(validate_key("my_work").is_ok());
        assert!(validate_key("my-work-env").is_ok());
        assert!(validate_key("my_work_env").is_ok());
        assert!(validate_key("my-work_env").is_ok());
    }

    #[test]
    fn test_validate_key_valid_with_numbers() {
        assert!(validate_key("work123").is_ok());
        assert!(validate_key("env1").is_ok());
        assert!(validate_key("123work").is_ok());
        assert!(validate_key("v1-2-3").is_ok());
    }

    #[test]
    fn test_validate_key_invalid_empty() {
        let result = validate_key("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Key cannot be empty");
    }

    #[test]
    fn test_validate_key_invalid_reserved() {
        let result = validate_key("base");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Key 'base' is reserved");
    }

    #[test]
    fn test_validate_key_invalid_spaces() {
        let result = validate_key("my work");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("alphanumeric"));
    }

    #[test]
    fn test_validate_key_invalid_special_chars() {
        assert!(validate_key("my@work").is_err());
        assert!(validate_key("my.work").is_err());
        assert!(validate_key("my/work").is_err());
        assert!(validate_key("my\\work").is_err());
        assert!(validate_key("my work").is_err());
        assert!(validate_key("my!work").is_err());
        assert!(validate_key("my#work").is_err());
        assert!(validate_key("my$work").is_err());
    }

    #[test]
    fn test_validate_key_invalid_unicode() {
        assert!(validate_key("café").is_err());
        assert!(validate_key("wörk").is_err());
        assert!(validate_key("test™").is_err());
    }

    #[test]
    fn test_validate_key_edge_cases() {
        // Single character
        assert!(validate_key("a").is_ok());
        assert!(validate_key("1").is_ok());
        assert!(validate_key("-").is_ok());
        assert!(validate_key("_").is_ok());
        
        // Long keys
        assert!(validate_key("very-long-environment-name-with-many-words").is_ok());
        
        // Mixed case (should be allowed)
        assert!(validate_key("MyWork").is_ok());
        assert!(validate_key("WORK").is_ok());
    }

    // ============= Integration Config Tests =============
    #[test]
    fn test_gh_cli_config_structure() {
        let config = GhCliConfig {
            hosts: vec![
                GhCliHostUser {
                    host: "github.com".to_string(),
                    user: "testuser".to_string(),
                },
                GhCliHostUser {
                    host: "github.enterprise.com".to_string(),
                    user: "workuser".to_string(),
                },
            ],
        };
        
        assert_eq!(config.hosts.len(), 2);
        assert_eq!(config.hosts[0].host, "github.com");
        assert_eq!(config.hosts[1].user, "workuser");
    }

    #[test]
    fn test_one_password_ssh_key_structure() {
        let key = OnePasswordSSHKey {
            vault: Some("Personal".to_string()),
            item: Some("SSH Key".to_string()),
            account: Some("user@example.com".to_string()),
        };
        
        assert!(key.vault.is_some());
        assert_eq!(key.vault.unwrap(), "Personal");
    }

    #[test]
    fn test_one_password_ssh_key_optional_fields() {
        let key_minimal = OnePasswordSSHKey {
            vault: None,
            item: Some("Key".to_string()),
            account: None,
        };
        
        assert!(key_minimal.vault.is_none());
        assert!(key_minimal.item.is_some());
        assert!(key_minimal.account.is_none());
    }

    #[test]
    fn test_one_password_ssh_agent_config() {
        let config = OnePasswordSSHAgentConfig {
            keys: vec![
                OnePasswordSSHKey {
                    vault: Some("Work".to_string()),
                    item: Some("Work SSH".to_string()),
                    account: Some("work@company.com".to_string()),
                },
                OnePasswordSSHKey {
                    vault: Some("Personal".to_string()),
                    item: Some("Personal SSH".to_string()),
                    account: None,
                },
            ],
        };
        
        assert_eq!(config.keys.len(), 2);
    }

    #[test]
    fn test_tailscale_config_structure() {
        let config = TailscaleConfig {
            tailnet: "company.example.com".to_string(),
        };
        
        assert_eq!(config.tailnet, "company.example.com");
    }

    #[test]
    fn test_environment_config_creation() {
        let config = EnvironmentConfig {
            name: "Test Environment".to_string(),
            env_vars: vec![],
            op_ssh: None,
            gh_cli: None,
            tailscale: None,
        };
        
        assert_eq!(config.name, "Test Environment");
        assert_eq!(config.env_vars.len(), 0);
        assert!(config.op_ssh.is_none());
        assert!(config.gh_cli.is_none());
        assert!(config.tailscale.is_none());
    }

    #[test]
    fn test_environment_config_with_all_integrations() {
        let config = EnvironmentConfig {
            name: "Full Config".to_string(),
            env_vars: vec![],
            op_ssh: Some(OnePasswordSSHAgentConfig { keys: vec![] }),
            gh_cli: Some(GhCliConfig {
                hosts: vec![GhCliHostUser {
                    host: "github.com".to_string(),
                    user: "user".to_string(),
                }],
            }),
            tailscale: Some(TailscaleConfig {
                tailnet: "example.com".to_string(),
            }),
        };
        
        assert!(config.op_ssh.is_some());
        assert!(config.gh_cli.is_some());
        assert!(config.tailscale.is_some());
    }

    #[test]
    fn test_environment_config_serialization() {
        let config = EnvironmentConfig {
            name: "Serialization Test".to_string(),
            env_vars: vec![],
            op_ssh: None,
            gh_cli: Some(GhCliConfig {
                hosts: vec![GhCliHostUser {
                    host: "github.com".to_string(),
                    user: "testuser".to_string(),
                }],
            }),
            tailscale: None,
        };
        
        let yaml = serde_norway::to_string(&config).expect("Failed to serialize");
        assert!(yaml.contains("name:"));
        assert!(yaml.contains("Serialization Test"));
        assert!(yaml.contains("gh_cli:"));
        assert!(yaml.contains("github.com"));
    }
}