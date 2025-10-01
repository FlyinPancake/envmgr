mod environment;
mod global;

pub use environment::{BASE_ENV_NAME, EnvVarsConfig, EnvironmentConfig};
#[expect(unused_imports)]
pub use global::GlobalConfig;

pub fn envmgr_config_dir() -> std::path::PathBuf {
    let config_local_dir = dirs::config_local_dir().expect("Could not determine home directory");
    config_local_dir.join("envmgr")
}
