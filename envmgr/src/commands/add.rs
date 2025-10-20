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

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("My Work"), "my-work");
        assert_eq!(slugify("Personal 123"), "personal-123");
        assert_eq!(slugify("Test__Environment"), "test-environment");
        assert_eq!(slugify("  spaces  "), "spaces");
        assert_eq!(slugify("Special!@#Chars"), "specialchars");
    }

    #[test]
    fn test_validate_key() {
        assert!(validate_key("work").is_ok());
        assert!(validate_key("my-work").is_ok());
        assert!(validate_key("my_work").is_ok());
        assert!(validate_key("work123").is_ok());

        assert!(validate_key("").is_err());
        assert!(validate_key("base").is_err());
        assert!(validate_key("my work").is_err());
        assert!(validate_key("my@work").is_err());
    }
}
