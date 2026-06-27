use clap::Parser;
use crate::errors::McpcError;

#[derive(Parser)]
pub struct RunArgs {}

pub fn execute(_args: RunArgs) -> Result<(), McpcError> {
    tracing::info!("[mcpc] run");
    Ok(())
}