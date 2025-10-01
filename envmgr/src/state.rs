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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = State::default();
        assert_eq!(state.current_env_key, crate::config::BASE_ENV_NAME);
        assert_eq!(state.applied_env_vars.len(), 0);
        assert_eq!(state.managed_files.len(), 0);
    }

    #[test]
    fn test_state_serialization_roundtrip() {
        let mut state = State {
            current_env_key: "test_env".to_string(),
            ..State::default()
        };
        state
            .applied_env_vars
            .insert("KEY1".to_string(), "value1".to_string());
        state
            .applied_env_vars
            .insert("KEY2".to_string(), "value2".to_string());
        state
            .managed_files
            .push(PathBuf::from("/home/user/.config"));

        let serialized = toml::to_string(&state).unwrap();
        let deserialized: State = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.current_env_key, "test_env");
        assert_eq!(deserialized.applied_env_vars.len(), 2);
        assert_eq!(
            deserialized.applied_env_vars.get("KEY1"),
            Some(&"value1".to_string())
        );
        assert_eq!(deserialized.managed_files.len(), 1);
    }

    #[test]
    fn test_state_empty_serialization() {
        let state = State::default();
        let serialized = toml::to_string(&state).unwrap();
        let deserialized: State = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.current_env_key, crate::config::BASE_ENV_NAME);
        assert!(deserialized.applied_env_vars.is_empty());
        assert!(deserialized.managed_files.is_empty());
    }
}
