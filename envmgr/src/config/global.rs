use super::envmgr_config_dir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GlobalConfig {}

#[expect(dead_code)]
impl GlobalConfig {
    pub fn get_config_file_path() -> std::path::PathBuf {
        envmgr_config_dir().join("global.yaml")
    }
}
