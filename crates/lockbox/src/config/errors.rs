#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("builder error: {0}")]
    Builder(#[from] config::ConfigError),
    #[error("validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
}
