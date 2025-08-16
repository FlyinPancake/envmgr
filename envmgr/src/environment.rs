use anyhow::Result;
use std::collections::HashMap;
use std::fs;

use crate::config::{EnvMgrConfig, EnvironmentConfig};
use crate::dotfiles::DotfileManager;
use crate::plugins::PluginManager;

/// Environment manager for creating, switching, and managing environments
pub struct EnvironmentManager {
    config: EnvMgrConfig,
    pub dotfile_manager: DotfileManager,
    pub plugin_manager: PluginManager,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DetectedShell {
    Bash,
    Zsh,
    Fish,
}

impl EnvironmentManager {
    pub fn new(config: EnvMgrConfig) -> Self {
        let dotfile_manager = DotfileManager::new(config.clone());
        let plugin_manager = PluginManager::new(config.clone());

        Self {
            config,
            dotfile_manager,
            plugin_manager,
        }
    }

    /// Create a new environment
    pub async fn create_environment(&self, name: &str, base: Option<&str>) -> Result<()> {
        if self.config.env_exists(name) {
            anyhow::bail!("Environment '{}' already exists", name);
        }

        // Create environment directory structure
        let env_dir = self.config.env_dir(name);
        fs::create_dir_all(&env_dir)?;
        fs::create_dir_all(env_dir.join("dotfiles"))?;
        fs::create_dir_all(env_dir.join("plugins"))?;

        // Create environment config
        let env_config = EnvironmentConfig {
            name: name.to_string(),
            base: base.map(String::from),
            env_vars: HashMap::new(),
            plugins: HashMap::new(),
        };

        env_config.save(&self.config.config_dir)?;

        // Initialize plugins
        self.plugin_manager.on_add_environment(&env_config).await?;

        println!("Created environment '{}'", name);
        Ok(())
    }

    /// Remove an environment
    pub async fn remove_environment(&self, name: &str) -> Result<()> {
        if !self.config.env_exists(name) {
            anyhow::bail!("Environment '{}' does not exist", name);
        }

        if self.config.current_env.as_deref() == Some(name) {
            anyhow::bail!("Cannot remove currently active environment '{}'", name);
        }

        // Load environment config before removal
        let env_config = EnvironmentConfig::load(&self.config.config_dir, name)?;

        // Run plugin cleanup
        self.plugin_manager
            .on_remove_environment(&env_config)
            .await?;

        // Remove environment directory
        let env_dir = self.config.env_dir(name);
        fs::remove_dir_all(env_dir)?;

        println!("Removed environment '{}'", name);
        Ok(())
    }

    /// Switch to an environment
    pub async fn use_environment(&mut self, name: &str) -> Result<()> {
        if !self.config.env_exists(name) {
            anyhow::bail!("Environment '{}' does not exist", name);
        }

        // Load environment config
        let env_config = EnvironmentConfig::load(&self.config.config_dir, name)?;

        // Apply dotfiles
        self.dotfile_manager.apply_environment(&env_config).await?;

        // Run plugins
        self.plugin_manager.on_use_environment(&env_config).await?;

        // Update current environment
        self.config.set_current_env(Some(name))?;

        // Output environment variables for shell eval
        self.output_env_vars(&env_config)?;

        Ok(())
    }

    /// Output environment variables for shell evaluation
    fn output_env_vars(&self, env_config: &EnvironmentConfig) -> Result<()> {
        let sh = detect_shell();

        // Output base environment variables first
        if let Some(base_name) = &env_config.base {
            if self.config.env_exists(base_name) {
                let base_config = EnvironmentConfig::load(&self.config.config_dir, base_name)?;
                for (key, value) in &base_config.env_vars {
                    emit_set(&sh, key, value);
                }
            }
        }

        // Output environment-specific variables (can override base)
        for (key, value) in &env_config.env_vars {
            emit_set(&sh, key, value);
        }

        // Set ENVMGR_CURRENT_ENV
        emit_set(&sh, "ENVMGR_CURRENT_ENV", &env_config.name);

        Ok(())
    }

    /// List all environments with status
    pub async fn list_environments(&self) -> Result<()> {
        let envs = self.config.list_environments()?;

        if envs.is_empty() {
            println!("No environments found");
            return Ok(());
        }

        println!("Available environments:");
        for env_name in envs {
            let is_current = self.config.current_env.as_deref() == Some(&env_name);
            let marker = if is_current { "* " } else { "  " };

            // Get environment details
            let env_config = EnvironmentConfig::load(&self.config.config_dir, &env_name)?;
            let base_info = env_config
                .base
                .as_ref()
                .map(|b| format!(" (inherits from {})", b))
                .unwrap_or_default();

            println!("{}{}{}", marker, env_name, base_info);

            // Show plugin status
            let plugin_status = self
                .plugin_manager
                .get_environment_status(&env_config)
                .await?;
            if !plugin_status.is_empty() {
                for status in plugin_status {
                    println!("    {}", status);
                }
            }
        }

        Ok(())
    }

    /// Show current environment
    pub fn show_current(&self) -> Result<()> {
        match &self.config.current_env {
            Some(name) => println!("{}", name),
            None => println!("No environment is currently active"),
        }
        Ok(())
    }
}

fn detect_shell() -> DetectedShell {
    if std::env::var("ENVMGR_SHELL")
        .map(|s| s.to_lowercase())
        .map(|s| s.contains("fish"))
        .unwrap_or(false)
        || std::env::var("FISH_VERSION").is_ok()
        || std::env::var("SHELL")
            .map(|s| s.ends_with("fish"))
            .unwrap_or(false)
    {
        return DetectedShell::Fish;
    }
    if std::env::var("SHELL")
        .map(|s| s.ends_with("zsh"))
        .unwrap_or(false)
    {
        return DetectedShell::Zsh;
    }
    DetectedShell::Bash
}

fn emit_set(sh: &DetectedShell, key: &str, value: &str) {
    match sh {
        DetectedShell::Fish => println!("set -gx {} {}", key, fish_escape(value)),
        DetectedShell::Bash | DetectedShell::Zsh => {
            println!("export {}={}", key, shell_escape(value))
        }
    }
}

/// Escape a string for shell usage
fn shell_escape(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    }
}

fn fish_escape(s: &str) -> String {
    // Use single-quoted string, escape single quotes using the common POSIX trick
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    }
}
