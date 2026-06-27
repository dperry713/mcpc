use clap::Parser;
use std::fs;
use crate::errors::McpcError;

#[derive(Parser)]
pub struct CleanArgs {}

pub fn execute(_args: CleanArgs) -> Result<(), McpcError> {
    fs::remove_dir_all("automata-mcp").ok();
    tracing::info!("[mcpc] cleaned");
    Ok(())
}