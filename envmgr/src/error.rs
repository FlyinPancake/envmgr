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
    #[error("Other Error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type EnvMgrResult<T> = std::result::Result<T, EnvMgrError>;
