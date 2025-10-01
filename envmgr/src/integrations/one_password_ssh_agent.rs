use crate::{
    error::{EnvMgrError, EnvMgrResult},
    integrations::OnSwitchToPluginResult,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct OnePasswordSSHAgentConfig {
    pub keys: Vec<OnePasswordSSHKey>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OnePasswordSSHKey {
    pub vault: Option<String>,
    pub item: Option<String>,
    pub account: Option<String>,
}

pub struct OnePasswordSSHAgent;

#[derive(Debug, Clone, serde::Serialize)]
struct OPAgentFile {
    #[serde(rename = "ssh-keys")]
    ssh_keys: Vec<OnePasswordSSHKey>,
}

impl OnePasswordSSHAgent {
    fn op_ssh_agent_file_path() -> EnvMgrResult<std::path::PathBuf> {
        let path = dirs::config_dir()
            .ok_or(EnvMgrError::DirError(
                "Could not determine config directory".into(),
            ))?
            .join("1Password")
            .join("ssh")
            .join("agent.toml");
        Ok(path)
    }

    fn ensure_op_ssh_agent_dir_exists() -> EnvMgrResult<()> {
        let path = Self::op_ssh_agent_file_path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
    pub fn on_switch_to(
        config: &OnePasswordSSHAgentConfig,
    ) -> EnvMgrResult<OnSwitchToPluginResult> {
        if config.keys.is_empty() {
            return Ok(Default::default());
        }

        let content = toml::to_string_pretty(&OPAgentFile {
            ssh_keys: config.keys.clone(),
        })?;

        Self::ensure_op_ssh_agent_dir_exists()?;

        std::fs::write(Self::op_ssh_agent_file_path()?, content)?;

        Ok(Default::default())
    }
}
