use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpcError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Build Error: {0}")]
    Build(String),

    #[error("Spec Error: {0}")]
    Spec(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
