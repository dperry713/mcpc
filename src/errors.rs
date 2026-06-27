use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum McpcError {
    #[error("I/O Error: {0}")]
    #[diagnostic(code(mcpc::io))]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    #[diagnostic(code(mcpc::serialization))]
    Serialization(#[from] serde_json::Error),

    #[error("Validation Error: {0}")]
    #[diagnostic(code(mcpc::validation))]
    Validation(String),

    #[error("Build Error: {0}")]
    #[diagnostic(code(mcpc::build))]
    Build(String),

    #[error("Spec Error: {0}")]
    #[diagnostic(code(mcpc::spec))]
    Spec(String),

    #[error(transparent)]
    #[diagnostic(code(mcpc::other))]
    Other(#[from] anyhow::Error),
}
