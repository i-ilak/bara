use thiserror::Error;

#[derive(Error, Debug)]
pub enum BaraError {
    #[error("Config error: {0}")]
    ConfigError(#[from] ConfigError),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("File not found")]
    FileNotFoundOrPermission(String),

    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Failed to deserialize YAML: {0}")]
    DeserializationError(String),
}
