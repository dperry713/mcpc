use crate::schema::Module;
use sha2::{Sha256, Digest};

pub fn compute_module_hash(module: &Module) -> String {
    let mut hasher = Sha256::new();
    
    // 1. Hash a canonical representation of the module metadata
    let mut canonical = String::new();
    canonical.push_str(&format!("name:{}\n", module.name));
    canonical.push_str(&format!("type:{}\n", module.module_type.as_deref().unwrap_or("default")));
    canonical.push_str(&format!("entry:{}\n", module.entry.as_deref().unwrap_or("null")));
    
    let mut sorted_features = module.features.clone();
    sorted_features.sort();
    canonical.push_str(&format!("features:{:?}\n", sorted_features));
    
    let mut sorted_deps = module.dependencies.clone();
    sorted_deps.sort();
    canonical.push_str(&format!("dependencies:{:?}\n", sorted_deps));
    
    // Hash _meta if present to detect metadata changes
    if let Some(ref meta) = module.meta {
        if let Ok(meta_str) = serde_json::to_string(meta) {
            canonical.push_str(&format!("meta:{}\n", meta_str));
        }
    }
    
    hasher.update(canonical.as_bytes());

    format!("{:x}", hasher.finalize())
}
