use std::fs;
use crate::errors::McpcError;

pub fn execute() -> Result<(), McpcError> {
    fs::remove_dir_all("automata-mcp").ok();
    tracing::info!("[mcpc] cleaned");
    Ok(())
}