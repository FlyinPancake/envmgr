mod environment;
mod global;

pub use environment::{BASE_ENV_NAME, EnvVarsConfig, EnvironmentConfig};
pub use global::GlobalConfig;

pub fn envmgr_config_dir() -> std::path::PathBuf {
    let config_local_dir = dirs::config_local_dir().expect("Could not determine home directory");
    config_local_dir.join("envmgr")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envmgr_config_dir_structure() {
        let config_dir = envmgr_config_dir();
        assert!(config_dir.ends_with("envmgr"));
        assert!(config_dir.is_absolute());
    }

    #[test]
    fn test_environment_config_paths() {
        let base_dir = EnvironmentConfig::get_base_env_dir();
        assert!(base_dir.ends_with(BASE_ENV_NAME));

        let env_dir = EnvironmentConfig::get_env_dir_by_key("test");
        assert!(env_dir.ends_with("environments/test"));

        let all_envs_dir = EnvironmentConfig::get_all_envs_dir();
        assert!(all_envs_dir.ends_with("environments"));
    }
}
