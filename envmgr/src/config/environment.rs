use config::Config;

use crate::error::EnvMgrResult;

use super::envmgr_config_dir;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EnvironmentConfig {
    pub name: String,
    #[serde(default)]
    pub env_vars: Vec<EnvVarsConfig>,
    pub op_ssh: Option<crate::integrations::one_password_ssh_agent::OnePasswordSSHAgentConfig>,
    pub gh_cli: Option<crate::integrations::gh_cli::GhCliConfig>,
    pub tailscale: Option<crate::integrations::tailscale::TailscaleConfig>,
}

const ENVS_DIR_NAME: &str = "environments";
const ENV_CONFIG_FILE_NAME: &str = "config.yaml";
pub const BASE_ENV_NAME: &str = "base";

impl EnvironmentConfig {
    /// Get the directory path for the base environment
    /// e.g., ~/.config/envmgr/base
    pub fn get_base_env_dir() -> std::path::PathBuf {
        envmgr_config_dir().join(BASE_ENV_NAME)
    }
    /// Get the directory path for a specific environment by its key
    /// e.g., ~/.config/envmgr/environments/<key>
    pub fn get_env_dir_by_key(key: &str) -> std::path::PathBuf {
        Self::get_all_envs_dir().join(key)
    }
    /// Get the directory path where all environments are stored
    /// e.g., ~/.config/envmgr/environments
    pub fn get_all_envs_dir() -> std::path::PathBuf {
        envmgr_config_dir().join(ENVS_DIR_NAME)
    }

    fn load_from_file(config_dir: &Path) -> EnvMgrResult<Self> {
        let config: Self = Config::builder()
            .add_source(config::File::from(config_dir.join(ENV_CONFIG_FILE_NAME)))
            .build()?
            .try_deserialize()?;
        Ok(config)
    }

    pub fn load_base_config() -> EnvMgrResult<Self> {
        let base_env_path = Self::get_base_env_dir();
        Self::load_from_file(&base_env_path)
    }

    pub fn load_env_config_by_key(key: &str) -> EnvMgrResult<Self> {
        let env_path = Self::get_env_dir_by_key(key);
        Self::load_from_file(&env_path)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EnvVarsConfig {
    pub key: String,
    pub value: String,
}
