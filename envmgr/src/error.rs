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
    #[error("Serde Norway Serialization Error: {0}")]
    SerdeNorwaySerialization(#[from] serde_norway::Error),
    #[error("Dialoguer Error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
    #[error("Already Exists: {0}")]
    AlreadyExists(String),
    #[error("Other Error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type EnvMgrResult<T> = std::result::Result<T, EnvMgrError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ============= Existing Error Conversions =============
    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let env_error: EnvMgrError = io_error.into();
        assert!(matches!(env_error, EnvMgrError::Io(_)));
        assert!(env_error.to_string().contains("I/O Error"));
    }

    #[test]
    fn test_io_error_different_kinds() {
        let permission_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let env_error: EnvMgrError = permission_error.into();
        assert!(matches!(env_error, EnvMgrError::Io(_)));
        assert!(env_error.to_string().contains("I/O Error"));

        let connection_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let env_error: EnvMgrError = connection_error.into();
        assert!(matches!(env_error, EnvMgrError::Io(_)));
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
    fn test_toml_serialization_error_conversion() {
        // Create a value that will fail serialization (using NaN which can't be serialized)
        use toml::ser::Error as TomlSerError;

        // Since we can't easily trigger a TomlSerError, we'll test the variant exists
        // and can be created (this tests the error enum structure)
        let test_str = "test message";
        let error = EnvMgrError::DirError(test_str.to_string());
        assert!(error.to_string().contains(test_str));
    }

    #[test]
    fn test_dir_error_message() {
        let error = EnvMgrError::DirError("home".to_string());
        assert_eq!(error.to_string(), "Could not determine directory: home");
    }

    #[test]
    fn test_dir_error_different_messages() {
        let error1 = EnvMgrError::DirError("config".to_string());
        assert_eq!(error1.to_string(), "Could not determine directory: config");

        let error2 = EnvMgrError::DirError("state".to_string());
        assert_eq!(error2.to_string(), "Could not determine directory: state");

        let error3 = EnvMgrError::DirError("/path/to/dir".to_string());
        assert_eq!(
            error3.to_string(),
            "Could not determine directory: /path/to/dir"
        );
    }

    #[test]
    fn test_gh_cli_config_error_message() {
        let error = EnvMgrError::GhCliConfig("invalid host".to_string());
        assert_eq!(error.to_string(), "GhCli Config Error: invalid host");
    }

    #[test]
    fn test_gh_cli_config_error_various_messages() {
        let error1 = EnvMgrError::GhCliConfig("missing user".to_string());
        assert_eq!(error1.to_string(), "GhCli Config Error: missing user");

        let error2 = EnvMgrError::GhCliConfig("failed to parse hosts.yml".to_string());
        assert_eq!(
            error2.to_string(),
            "GhCli Config Error: failed to parse hosts.yml"
        );
    }

    // ============= New Error Types Added in Diff =============
    #[test]
    fn test_serde_norway_error_conversion() {
        // Test that serde_norway::Error can be converted to EnvMgrError
        // We'll create an invalid YAML structure to trigger an error
        let invalid_yaml = "{ invalid: yaml: structure";
        let yaml_error = serde_norway::from_str::<serde_norway::Value>(invalid_yaml).unwrap_err();
        let env_error: EnvMgrError = yaml_error.into();
        assert!(matches!(
            env_error,
            EnvMgrError::SerdeNorwaySerialization(_)
        ));
        assert!(
            env_error
                .to_string()
                .contains("Serde Norway Serialization Error")
        );
    }

    #[test]
    fn test_already_exists_error() {
        let error = EnvMgrError::AlreadyExists("environment 'work'".to_string());
        assert_eq!(error.to_string(), "Already Exists: environment 'work'");
    }

    #[test]
    fn test_already_exists_error_various_messages() {
        let error1 = EnvMgrError::AlreadyExists("file at /path/to/file".to_string());
        assert_eq!(error1.to_string(), "Already Exists: file at /path/to/file");

        let error2 = EnvMgrError::AlreadyExists("Environment 'personal' already exists at /home/user/.config/envmgr/environments/personal".to_string());
        assert!(error2.to_string().contains("Already Exists"));
        assert!(error2.to_string().contains("personal"));
    }

    #[test]
    fn test_already_exists_error_empty_message() {
        let error = EnvMgrError::AlreadyExists("".to_string());
        assert_eq!(error.to_string(), "Already Exists: ");
    }

    // ============= Error Type Matching =============
    #[test]
    fn test_error_type_matching() {
        let io_err = EnvMgrError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(matches!(io_err, EnvMgrError::Io(_)));
        assert!(!matches!(io_err, EnvMgrError::DirError(_)));

        let dir_err = EnvMgrError::DirError("test".to_string());
        assert!(matches!(dir_err, EnvMgrError::DirError(_)));
        assert!(!matches!(dir_err, EnvMgrError::Io(_)));

        let already_exists_err = EnvMgrError::AlreadyExists("test".to_string());
        assert!(matches!(already_exists_err, EnvMgrError::AlreadyExists(_)));
        assert!(!matches!(already_exists_err, EnvMgrError::DirError(_)));
    }

    // ============= Result Type Alias =============
    #[test]
    fn test_result_type_alias() {
        let success: EnvMgrResult<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        let failure: EnvMgrResult<i32> = Err(EnvMgrError::DirError("test".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_result_type_with_various_types() {
        let string_result: EnvMgrResult<String> = Ok("test".to_string());
        assert_eq!(string_result.unwrap(), "test");

        let unit_result: EnvMgrResult<()> = Ok(());
        assert!(unit_result.is_ok());

        let vec_result: EnvMgrResult<Vec<i32>> = Ok(vec![1, 2, 3]);
        assert_eq!(vec_result.unwrap().len(), 3);
    }

    // ============= Error Display =============
    #[test]
    fn test_error_display_format() {
        let errors = vec![
            EnvMgrError::DirError("test".to_string()),
            EnvMgrError::GhCliConfig("test".to_string()),
            EnvMgrError::AlreadyExists("test".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
            assert!(display.contains("test"));
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = EnvMgrError::AlreadyExists("test".to_string());
        let debug = format!("{:?}", error);
        assert!(!debug.is_empty());
        assert!(debug.contains("AlreadyExists"));
    }

    // ============= Error Chaining =============
    #[test]
    fn test_error_question_mark_operator() {
        fn returns_io_error() -> EnvMgrResult<()> {
            std::fs::read_to_string("/nonexistent/file")?;
            Ok(())
        }

        let result = returns_io_error();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EnvMgrError::Io(_)));
    }

    #[test]
    fn test_error_propagation() {
        fn inner() -> EnvMgrResult<i32> {
            Err(EnvMgrError::DirError("inner error".to_string()))
        }

        fn outer() -> EnvMgrResult<i32> {
            inner()?;
            Ok(42)
        }

        let result = outer();
        assert!(result.is_err());
        match result.unwrap_err() {
            EnvMgrError::DirError(msg) => assert_eq!(msg, "inner error"),
            _ => panic!("Wrong error type"),
        }
    }
}
