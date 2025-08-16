use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin configuration that can be serialized to/from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub data: HashMap<String, serde_yaml::Value>,
}

/// Result of plugin operations
pub type PluginResult<T> = anyhow::Result<T>;

/// Plugin trait that all envmgr plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the name of this plugin
    fn name(&self) -> &str;

    /// Get the schema/description of what configuration this plugin expects
    fn config_schema(&self) -> HashMap<String, String>;

    /// Called when an environment is activated
    async fn on_use(&self, config: &PluginConfig, env_name: &str) -> PluginResult<()>;

    /// Called when adding a new environment
    async fn on_add(&self, config: &PluginConfig, env_name: &str) -> PluginResult<()>;

    /// Called when removing an environment
    async fn on_remove(&self, config: &PluginConfig, env_name: &str) -> PluginResult<()>;

    /// Called when listing environments (for status info)
    async fn on_list(&self, config: &PluginConfig, env_name: &str) -> PluginResult<String>;

    /// Validate the plugin configuration
    fn validate_config(&self, config: &PluginConfig) -> PluginResult<()>;
}

// PluginRegistry has been moved to the envmgr crate.
