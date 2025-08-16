use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::config::{EnvMgrConfig, EnvironmentConfig};
use envmgr_plugin::{Plugin, PluginConfig};

/// Runtime plugin registry owned by envmgr
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages plugins and their lifecycle
pub struct PluginManager {
    config: EnvMgrConfig,
    registry: PluginRegistry,
}

impl PluginManager {
    pub fn new(config: EnvMgrConfig) -> Self {
        let registry = PluginRegistry::new();
        // No built-in plugins. External plugin discovery/registration to be implemented.
        Self { config, registry }
    }

    /// Handle environment activation
    pub async fn on_use_environment(&self, env_config: &EnvironmentConfig) -> Result<()> {
        for (plugin_name, plugin_data) in &env_config.plugins {
            if let Some(plugin) = self.registry.get(plugin_name) {
                let plugin_config = PluginConfig {
                    name: plugin_name.clone(),
                    data: if let serde_yaml::Value::Mapping(map) = plugin_data {
                        map.iter()
                            .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
                            .collect()
                    } else {
                        HashMap::new()
                    },
                };

                plugin
                    .on_use(&plugin_config, &env_config.name)
                    .await
                    .with_context(|| {
                        format!(
                            "Plugin '{}' failed during environment activation",
                            plugin_name
                        )
                    })?;
            }
        }

        Ok(())
    }

    /// Handle environment creation
    pub async fn on_add_environment(&self, env_config: &EnvironmentConfig) -> Result<()> {
        for (plugin_name, plugin_data) in &env_config.plugins {
            if let Some(plugin) = self.registry.get(plugin_name) {
                let plugin_config = PluginConfig {
                    name: plugin_name.clone(),
                    data: if let serde_yaml::Value::Mapping(map) = plugin_data {
                        map.iter()
                            .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
                            .collect()
                    } else {
                        HashMap::new()
                    },
                };

                plugin
                    .on_add(&plugin_config, &env_config.name)
                    .await
                    .with_context(|| {
                        format!(
                            "Plugin '{}' failed during environment creation",
                            plugin_name
                        )
                    })?;
            }
        }

        Ok(())
    }

    /// Handle environment removal
    pub async fn on_remove_environment(&self, env_config: &EnvironmentConfig) -> Result<()> {
        for (plugin_name, plugin_data) in &env_config.plugins {
            if let Some(plugin) = self.registry.get(plugin_name) {
                let plugin_config = PluginConfig {
                    name: plugin_name.clone(),
                    data: if let serde_yaml::Value::Mapping(map) = plugin_data {
                        map.iter()
                            .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
                            .collect()
                    } else {
                        HashMap::new()
                    },
                };

                plugin
                    .on_remove(&plugin_config, &env_config.name)
                    .await
                    .with_context(|| {
                        format!("Plugin '{}' failed during environment removal", plugin_name)
                    })?;
            }
        }

        Ok(())
    }

    /// Get status information for environment plugins
    pub async fn get_environment_status(
        &self,
        env_config: &EnvironmentConfig,
    ) -> Result<Vec<String>> {
        let mut status = Vec::new();

        for (plugin_name, plugin_data) in &env_config.plugins {
            if let Some(plugin) = self.registry.get(plugin_name) {
                let plugin_config = PluginConfig {
                    name: plugin_name.clone(),
                    data: if let serde_yaml::Value::Mapping(map) = plugin_data {
                        map.iter()
                            .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
                            .collect()
                    } else {
                        HashMap::new()
                    },
                };

                match plugin.on_list(&plugin_config, &env_config.name).await {
                    Ok(plugin_status) => status.push(format!("{}: {}", plugin_name, plugin_status)),
                    Err(e) => status.push(format!("{}: Error - {}", plugin_name, e)),
                }
            }
        }

        Ok(status)
    }

    /// List available plugins (from registry). With no built-ins, this will be empty
    pub async fn list_plugins(&self) -> Result<()> {
        let names = self.registry.list();
        println!("Available plugins:");
        if names.is_empty() {
            println!("  (none discovered; external plugin discovery not implemented yet)");
            return Ok(());
        }
        for plugin_name in names {
            if let Some(plugin) = self.registry.get(plugin_name) {
                println!("  {}", plugin_name);
                let schema = plugin.config_schema();
                for (key, description) in schema {
                    println!("    {}: {}", key, description);
                }
            }
        }

        Ok(())
    }

    /// Enable a plugin for an environment
    /// Note: We no longer require the plugin to be present in the registry; this just
    /// records the plugin name in the environment config. If a matching plugin implementation
    /// is discovered later, its hooks will run on use/add/remove.
    pub async fn enable_plugin(&self, plugin_name: &str, env_name: &str) -> Result<()> {
        if !self.config.env_exists(env_name) {
            anyhow::bail!("Environment '{}' does not exist", env_name);
        }

        let mut env_config = EnvironmentConfig::load(&self.config.config_dir, env_name)?;

        if !env_config.plugins.contains_key(plugin_name) {
            env_config.plugins.insert(
                plugin_name.to_string(),
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
            env_config.save(&self.config.config_dir)?;
            println!(
                "Enabled plugin '{}' for environment '{}' (no implementation discovered)",
                plugin_name, env_name
            );
        } else {
            println!(
                "Plugin '{}' is already enabled for environment '{}'",
                plugin_name, env_name
            );
        }

        Ok(())
    }

    /// Disable a plugin for an environment
    pub async fn disable_plugin(&self, plugin_name: &str, env_name: &str) -> Result<()> {
        if !self.config.env_exists(env_name) {
            anyhow::bail!("Environment '{}' does not exist", env_name);
        }

        let mut env_config = EnvironmentConfig::load(&self.config.config_dir, env_name)?;

        if env_config.plugins.remove(plugin_name).is_some() {
            env_config.save(&self.config.config_dir)?;
            println!(
                "Disabled plugin '{}' for environment '{}'",
                plugin_name, env_name
            );
        } else {
            println!(
                "Plugin '{}' is not enabled for environment '{}'",
                plugin_name, env_name
            );
        }

        Ok(())
    }
}

// No built-in plugin implementations. External plugins will be supported via a discovery/registration
// mechanism in a future iteration.
