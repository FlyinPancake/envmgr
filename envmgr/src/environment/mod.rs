mod manager;

use log::{debug, info, warn};
pub use manager::EnvironmentManager;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    config::{BASE_ENV_NAME, EnvVarsConfig, EnvironmentConfig},
    error::{EnvMgrError, EnvMgrResult},
};

pub struct Environment {
    pub key: String,
    pub name: String,
    pub env_vars: Vec<EnvVarsConfig>,
    pub one_password_ssh:
        Option<crate::integrations::one_password_ssh_agent::OnePasswordSSHAgentConfig>,
    pub gh_cli: Option<crate::integrations::gh_cli::GhCliConfig>,
    pub tailscale: Option<crate::integrations::tailscale::TailscaleConfig>,
}

impl Environment {
    fn load_from_config(key: &str, config: &EnvironmentConfig) -> Self {
        debug!("Loading environment: {} ({key})", config.name);
        Self {
            key: key.to_string(),
            name: config.name.clone(),
            env_vars: config.env_vars.clone(),
            one_password_ssh: config.op_ssh.clone(),
            gh_cli: config.gh_cli.clone(),
            tailscale: config.tailscale.clone(),
        }
    }

    pub fn load_base_environment() -> EnvMgrResult<Self> {
        let base_env_config = EnvironmentConfig::load_base_config()?;
        Ok(Self::load_from_config(BASE_ENV_NAME, &base_env_config))
    }

    pub fn load_environment_by_key(key: &str) -> EnvMgrResult<Self> {
        let env_config = EnvironmentConfig::load_env_config_by_key(key)?;
        Ok(Self::load_from_config(key, &env_config))
    }

    fn env_dir(&self) -> PathBuf {
        if self.key == BASE_ENV_NAME {
            EnvironmentConfig::get_base_env_dir()
        } else {
            EnvironmentConfig::get_env_dir_by_key(&self.key)
        }
    }

    fn files_dir(&self) -> PathBuf {
        self.env_dir().join("files")
    }

    /// Returns a map of source file paths to target link paths for the environment
    ///
    /// Example: { "/home/user/.bashrc" => "/home/user/.config/envmgr/base/files/.bashrc" }
    pub fn files_to_link(&self) -> EnvMgrResult<HashMap<PathBuf, PathBuf>> {
        let mut file_map = HashMap::new();
        let files_dir = self.files_dir();
        if files_dir.exists() && files_dir.is_dir() {
            let files = discover_files_in_dir(&files_dir)?;
            for file in files {
                if let Ok(target_path) = file.strip_prefix(&files_dir) {
                    let target_full_path = dirs::home_dir()
                        .ok_or(EnvMgrError::DirError("home".into()))?
                        .join(target_path);
                    debug!(
                        "Mapping file for linking: {} -> {}",
                        target_full_path.display(),
                        file.display()
                    );
                    file_map.insert(target_full_path, file);
                } else {
                    warn!(
                        "File {} is not under the files directory {}",
                        file.display(),
                        files_dir.display()
                    );
                }
            }
        } else {
            info!(
                "No files directory found for environment {} (files dir: {})",
                self.key,
                files_dir.display()
            );
        }
        Ok(file_map)
    }
}

/// Utility function to discover files in a directory (recursively)
fn discover_files_in_dir(dir: &Path) -> EnvMgrResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.exists() && dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(discover_files_in_dir(&path)?);
            }
        }
    }
    Ok(files)
}
