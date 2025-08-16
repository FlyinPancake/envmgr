use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration for envmgr
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvMgrConfig {
    pub config_dir: PathBuf,
    pub current_env: Option<String>,
}

impl EnvMgrConfig {
    /// Load configuration from the default location
    pub fn load() -> Result<Self> {
        let config_dir = Self::default_config_dir()?;
        
        // Ensure the config directory exists
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(config_dir.join("base/dotfiles"))?;
        fs::create_dir_all(config_dir.join("plugins/available"))?;
        fs::create_dir_all(config_dir.join("plugins/enabled"))?;

        let current_env = Self::load_current_env(&config_dir)?;

        Ok(Self {
            config_dir,
            current_env,
        })
    }

    /// Get the default configuration directory (XDG compliant)
    pub fn default_config_dir() -> Result<PathBuf> {
        // Use XDG_CONFIG_HOME if set, otherwise fall back to ~/.config
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .map(Ok)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|home| home.join(".config"))
                    .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))
            })?;

        Ok(config_home.join("envmgr"))
    }

    /// Load the current environment from the state file
    fn load_current_env(config_dir: &Path) -> Result<Option<String>> {
        let current_file = config_dir.join("current");
        if current_file.exists() {
            let content = fs::read_to_string(current_file)?;
            Ok(Some(content.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    /// Set the current environment
    pub fn set_current_env(&mut self, env_name: Option<&str>) -> Result<()> {
        let current_file = self.config_dir.join("current");
        
        if let Some(name) = env_name {
            fs::write(current_file, name)?;
            self.current_env = Some(name.to_string());
        } else {
            if current_file.exists() {
                fs::remove_file(current_file)?;
            }
            self.current_env = None;
        }
        
        Ok(())
    }

    /// Get the base configuration directory
    pub fn base_dir(&self) -> PathBuf {
        self.config_dir.join("base")
    }

    /// Get the environment directory for a specific environment
    pub fn env_dir(&self, env_name: &str) -> PathBuf {
        self.config_dir.join(env_name)
    }

    /// Get the plugins directory
    pub fn plugins_dir(&self) -> PathBuf {
        self.config_dir.join("plugins")
    }

    /// Check if an environment exists
    pub fn env_exists(&self, env_name: &str) -> bool {
        self.env_dir(env_name).exists()
    }

    /// List all available environments
    pub fn list_environments(&self) -> Result<Vec<String>> {
        let mut envs = Vec::new();
        
        for entry in fs::read_dir(&self.config_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // Skip special directories
                if !matches!(name, "base" | "plugins" | "current") {
                    envs.push(name.to_string());
                }
            }
        }
        
        envs.sort();
        Ok(envs)
    }
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub base: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub plugins: HashMap<String, serde_yaml::Value>,
}

impl EnvironmentConfig {
    /// Load environment configuration from file
    pub fn load(config_dir: &Path, env_name: &str) -> Result<Self> {
        let config_file = config_dir.join(env_name).join("config.yaml");
        
        if config_file.exists() {
            let content = fs::read_to_string(config_file)?;
            serde_yaml::from_str(&content).context("Failed to parse environment config")
        } else {
            // Create default config
            Ok(Self {
                name: env_name.to_string(),
                base: None,
                env_vars: HashMap::new(),
                plugins: HashMap::new(),
            })
        }
    }

    /// Save environment configuration to file
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        let env_dir = config_dir.join(&self.name);
        fs::create_dir_all(&env_dir)?;
        
        let config_file = env_dir.join("config.yaml");
        let content = serde_yaml::to_string(self)?;
        fs::write(config_file, content)?;
        
        Ok(())
    }

    /// Get the dotfiles directory for this environment
    pub fn dotfiles_dir(&self, config_dir: &Path) -> PathBuf {
        config_dir.join(&self.name).join("dotfiles")
    }

    /// Get the plugins directory for this environment
    pub fn plugins_dir(&self, config_dir: &Path) -> PathBuf {
        config_dir.join(&self.name).join("plugins")
    }
}
