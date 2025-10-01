use std::path::PathBuf;

use saphyr::{LoadableYamlNode, Yaml, YamlEmitter};

use crate::{
    error::{EnvMgrError, EnvMgrResult},
    integrations::OnSwitchToPluginResult,
};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Default)]
pub struct GhCliConfig {
    pub hosts: Vec<GhCliHostUser>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Default)]
pub struct GhCliHostUser {
    pub host: String,
    pub user: String,
}

pub struct GhCli;

impl GhCli {
    fn gh_cli_hosts_file_path() -> EnvMgrResult<PathBuf> {
        let path = dirs::config_dir()
            .ok_or(EnvMgrError::DirError(
                "Could not determine config directory".into(),
            ))?
            .join("gh")
            .join("hosts.yml");
        Ok(path)
    }

    pub fn on_switch_to(config: &GhCliConfig) -> EnvMgrResult<OnSwitchToPluginResult> {
        let mut gh_cli_hosts_doc =
            if let Ok(content) = std::fs::read_to_string(Self::gh_cli_hosts_file_path()?) {
                Yaml::load_from_str(&content)?
            } else {
                vec![]
            };

        if gh_cli_hosts_doc.is_empty() {
            return Err(EnvMgrError::GhCliConfig(
                "GH CLI hosts file is empty or missing".into(),
            ));
        }

        let gh_cli_hosts = &mut gh_cli_hosts_doc[0];

        for GhCliHostUser { host, user } in &config.hosts {
            gh_cli_hosts
                .as_mapping_get_mut(host)
                .ok_or(EnvMgrError::GhCliConfig(format!(
                    "Host '{host}' not found in GH CLI hosts file"
                )))?
                .as_mapping_get_mut("users")
                .ok_or(EnvMgrError::GhCliConfig(format!(
                    "'users' section missing for host '{host}'"
                )))?
                .as_mapping_get_mut(user)
                .ok_or(EnvMgrError::GhCliConfig(format!(
                    "User '{user}' not found under host '{host}'"
                )))?;

            if let Some(u) = gh_cli_hosts
                .as_mapping_get_mut(host)
                .and_then(|h| h.as_mapping_get_mut("user"))
            {
                *u = Yaml::Value(saphyr::Scalar::String(user.clone().into()));
            }
        }
        let mut content = String::new();
        YamlEmitter::new(&mut content).dump(gh_cli_hosts)?;

        content.push('\n'); // Ensure file ends with a newline

        std::fs::write(Self::gh_cli_hosts_file_path()?, content)?;

        Ok(Default::default())
    }
}
