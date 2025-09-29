use std::{collections::HashMap, path::PathBuf};

use crate::error::EnvMgrResult;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct State {
    pub current_env_key: String,
    pub applied_env_vars: HashMap<String, String>,
    pub managed_files: Vec<PathBuf>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_env_key: crate::config::BASE_ENV_NAME.to_string(),
            applied_env_vars: HashMap::new(),
            managed_files: Vec::new(),
        }
    }
}

impl State {
    fn get_state_file_path() -> PathBuf {
        let envmgr_state_dir = dirs::state_dir()
            .expect("Could not determine state directory")
            .join("envmgr");
        if !envmgr_state_dir.exists() {
            std::fs::create_dir_all(&envmgr_state_dir).expect("Could not create state directory");
        }
        envmgr_state_dir.join("state.yaml")
    }

    pub fn get_state() -> EnvMgrResult<Self> {
        let state_file_path = Self::get_state_file_path();
        if !state_file_path.exists() {
            eprintln!("State file does not exist, returning default state");
            return Ok(State::default());
        }

        let state: State = toml::from_slice(&std::fs::read(state_file_path)?)?;

        Ok(state)
    }

    pub fn store_state(&self) -> EnvMgrResult<()> {
        let envmgr_state_dir = dirs::state_dir()
            .expect("Could not determine state directory")
            .join("envmgr");
        if !envmgr_state_dir.exists() {
            std::fs::create_dir_all(&envmgr_state_dir).expect("Could not create state directory");
        }

        let state_file_path = envmgr_state_dir.join("state.yaml");
        std::fs::write(state_file_path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}
