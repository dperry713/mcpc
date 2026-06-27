use crate::errors::McpcError;

pub fn execute() -> Result<(), McpcError> {
    tracing::info!("[mcpc] run");
    Ok(())
}