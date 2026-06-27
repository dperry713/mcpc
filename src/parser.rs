use std::fs;
use crate::schema::MCPSpec;
use crate::errors::McpcError;

pub fn load_spec(path: &str) -> Result<MCPSpec, McpcError> {
    let data = fs::read_to_string(path).map_err(McpcError::Io)?;
    let spec: MCPSpec = serde_json::from_str(&data).map_err(McpcError::Serialization)?;
    Ok(spec)
}