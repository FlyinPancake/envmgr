use std::collections::HashMap;

use log::{debug, info, warn};

use crate::{
    cli::Shell,
    config::{BASE_ENV_NAME, EnvVarsConfig, EnvironmentConfig},
    environment::Environment,
    error::EnvMgrResult,
    integrations::one_password_ssh_agent::OnePasswordSSHAgent,
    state::State,
};
pub struct EnvironmentManager {
    /// Shell environment variables to set
    pub shell: Shell,
}

impl EnvironmentManager {
    pub fn list_environments() -> EnvMgrResult<Vec<(bool, Environment)>> {
        let state = State::get_state()?;
        let envs_dir = EnvironmentConfig::get_all_envs_dir();
        if !envs_dir.exists() {
            return Ok(vec![]);
        }
        let base = Environment::load_base_environment()?;

        let mut environments = vec![(state.current_env_key == base.key, base)];
        for entry in std::fs::read_dir(envs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(env_key) = entry.file_name().to_str() {
                    let env = Environment::load_environment_by_key(env_key)?;
                    environments.push((state.current_env_key == env.key, env));
                }
            }
        }
        Ok(environments)
    }

    pub fn use_environment(&self) -> EnvMgrResult<()> {
        // Unset current environment variables
        let mut state = State::get_state()?;
        let target_env_key = state.current_env_key.clone();

        state.applied_env_vars.clear();
        // Set new environment variables
        let base_environment = Environment::load_base_environment()?;

        let mut new_vars = HashMap::new();

        for EnvVarsConfig { key, value } in base_environment.env_vars {
            new_vars.insert(key, value);
        }

        if target_env_key != BASE_ENV_NAME {
            let environment = Environment::load_environment_by_key(&target_env_key)?;
            state.current_env_key = environment.key.to_string();
            for EnvVarsConfig { key, value } in environment.env_vars {
                new_vars.insert(key, value);
            }
        } else {
            state.current_env_key = BASE_ENV_NAME.to_string();
        }

        // Remove keys that are no longer present
        let keys_to_remove: Vec<String> = state
            .applied_env_vars
            .keys()
            .filter(|k| !new_vars.contains_key(*k))
            .cloned()
            .collect();

        for key in keys_to_remove {
            println!("{}", self.shell.unset_env_var_cmd(&key));
            state.applied_env_vars.remove(&key);
        }

        // Set all new/updated variables
        for (key, value) in new_vars {
            println!("{}", self.shell.set_env_var_cmd(&key, &value));
            state.applied_env_vars.insert(key, value);
        }

        state.store_state()?;
        Ok(())
    }

    fn switch_environment(environment: &Environment) -> EnvMgrResult<()> {
        let mut state = State::get_state()?;
        if state.current_env_key == environment.key {
            // No change
            debug!("Environment {} is already active", environment.name);
            return Ok(());
        }
        info!(
            "Switching to environment: {} ({})",
            environment.name, environment.key
        );
        state.current_env_key = environment.key.to_string();

        // Integrations
        if let Some(op_ssh_config) = environment.one_password_ssh.as_ref() {
            OnePasswordSSHAgent::on_switch_to(op_ssh_config)?;
        }

        if let Some(gh_cli_config) = environment.gh_cli.as_ref() {
            crate::integrations::gh_cli::GhCli::on_switch_to(gh_cli_config)?;
        }

        if let Some(tailscale_config) = environment.tailscale.as_ref() {
            crate::integrations::tailscale::Tailscale::on_switch_to(tailscale_config)?;
        }

        state.store_state()?;
        Self::link_files()?;
        Ok(())
    }

    pub fn switch_environment_by_key(key: &str) -> EnvMgrResult<()> {
        let environment = Environment::load_environment_by_key(key)?;

        // Switch
        Self::switch_environment(&environment)?;

        Ok(())
    }

    pub fn switch_base_environment() -> EnvMgrResult<()> {
        let base_environment = Environment::load_base_environment()?;

        Self::switch_environment(&base_environment)?;

        Ok(())
    }

    pub fn link_files() -> EnvMgrResult<()> {
        let mut state = State::get_state()?;

        let base_environment = Environment::load_base_environment()?;
        let mut files_map = base_environment.files_to_link()?;

        if state.current_env_key != BASE_ENV_NAME {
            let environment = Environment::load_environment_by_key(&state.current_env_key)?;
            files_map.extend(environment.files_to_link()?);
        }

        for managed_file in state
            .managed_files
            .iter()
            .filter(|f| !files_map.contains_key(*f))
        {
            // Remove previously managed dangling symlink.
            if managed_file.is_symlink() {
                info!("Removing stale symlink: {}", managed_file.display());
                std::fs::remove_file(managed_file)?;
            } else if managed_file.exists() {
                warn!(
                    "Managed file exists and is not a symlink, skipping removal: {}",
                    managed_file.display()
                );
            }
        }

        state.managed_files.clear();

        for (target_path, source_path) in files_map {
            let mut need_link = true;

            if target_path.is_symlink() {
                // Handle both valid and dangling symlinks
                let existing_link = std::fs::read_link(&target_path)?;
                if existing_link == source_path {
                    debug!(
                        "Symlink already exists and is correct: {} -> {}",
                        target_path.display(),
                        source_path.display()
                    );
                    state.managed_files.push(target_path.clone());
                    need_link = false;
                } else {
                    info!(
                        "Updating symlink: {} (was {}) -> {}",
                        target_path.display(),
                        existing_link.display(),
                        source_path.display()
                    );
                    std::fs::remove_file(&target_path)?;
                }
            } else if target_path.exists() {
                // A real file/dir exists at the target and it's not a symlink – do not overwrite
                warn!(
                    "Target path exists and is not a symlink, skipping: {}",
                    target_path.display()
                );
                need_link = false;
            } else if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    info!("Creating parent directory: {}", parent.display());
                    std::fs::create_dir_all(parent)?;
                }
            }

            if need_link {
                info!(
                    "Creating symlink: {} -> {}",
                    target_path.display(),
                    source_path.display()
                );
                std::os::unix::fs::symlink(&source_path, &target_path)?;
                state.managed_files.push(target_path.clone());
            }
        }

        state.store_state()?;

        Ok(())
    }
}
