use crate::errors::McpcError;
use std::collections::HashMap;
use std::fs;

const CACHE_PATH: &str = "automata-mcp/.mcpc/cache.json";

pub type Cache = HashMap<String, String>;

pub fn load_cache() -> Cache {
    if let Ok(data) = fs::read_to_string(CACHE_PATH) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn save_cache(cache: &Cache) -> Result<(), McpcError> {
    fs::create_dir_all("automata-mcp/.mcpc").map_err(McpcError::Io)?;
    let data = serde_json::to_string_pretty(cache).map_err(McpcError::Serialization)?;
    fs::write(CACHE_PATH, data).map_err(McpcError::Io)?;
    Ok(())
}
