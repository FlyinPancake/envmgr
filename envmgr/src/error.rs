#[derive(thiserror::Error, Debug)]
pub enum EnvMgrError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration Error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Toml Deserialization Error: {0}")]
    TomlDeserialization(#[from] toml::de::Error),
    #[error("Toml Serialization Error: {0}")]
    TomlSerialization(#[from] toml::ser::Error),
    #[error("Could not determine directory: {0}")]
    DirError(String),
    #[error("GhCli Config Error: {0}")]
    GhCliConfig(String),
    #[error("Saphyr Scan Yaml Error: {0}")]
    SaphyrYaml(#[from] saphyr::ScanError),
    #[error("Saphyr Emit Yaml Error: {0}")]
    SaphyrEmitYaml(#[from] saphyr::EmitError),
    #[error("Other Error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type EnvMgrResult<T> = std::result::Result<T, EnvMgrError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let env_error: EnvMgrError = io_error.into();
        assert!(matches!(env_error, EnvMgrError::Io(_)));
        assert!(env_error.to_string().contains("I/O Error"));
    }

    #[test]
    fn test_toml_deserialization_error_conversion() {
        let toml_str = "invalid = [[[";
        let toml_error = toml::from_str::<toml::Value>(toml_str).unwrap_err();
        let env_error: EnvMgrError = toml_error.into();
        assert!(matches!(env_error, EnvMgrError::TomlDeserialization(_)));
        assert!(env_error.to_string().contains("Toml Deserialization Error"));
    }

    #[test]
    fn test_dir_error_message() {
        let error = EnvMgrError::DirError("home".to_string());
        assert_eq!(error.to_string(), "Could not determine directory: home");
    }

    #[test]
    fn test_gh_cli_config_error_message() {
        let error = EnvMgrError::GhCliConfig("invalid host".to_string());
        assert_eq!(error.to_string(), "GhCli Config Error: invalid host");
    }
}
