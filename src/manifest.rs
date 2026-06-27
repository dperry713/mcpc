use crate::planner::BuildPlan;
use crate::errors::McpcError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub timestamp: u64,
    pub modules: Vec<String>,
    pub hashes: HashMap<String, String>,
}

pub fn generate_manifest(plan: &BuildPlan) -> Result<(), McpcError> {
    let mut modules = Vec::new();
    let mut hashes = HashMap::new();

    for (name, hash) in &plan.new_cache {
        modules.push(name.clone());
        hashes.insert(name.clone(), hash.clone());
    }

    modules.sort();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let manifest = Manifest {
        version: 1,
        timestamp,
        modules,
        hashes,
    };

    let data = serde_json::to_string_pretty(&manifest).map_err(McpcError::Serialization)?;
    fs::write("automata-mcp/manifest.json", data).map_err(McpcError::Io)?;

    Ok(())
}
