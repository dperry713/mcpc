use crate::parser::load_spec;
use crate::errors::McpcError;
use crate::validator;

pub fn execute() -> Result<(), McpcError> {
    let spec = load_spec("mcp.spec.json")?;
    validator::validate(&spec)?;
    tracing::info!("[mcpc] valid spec with {} modules", spec.modules.len());
    Ok(())
}