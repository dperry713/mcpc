use crate::schema::Module;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn compute_module_hash(module: &Module) -> String {
    let mut hasher = DefaultHasher::new();
    // For now, we hash the JSON representation of the module to detect specification changes.
    let serialized = serde_json::to_string(module).unwrap_or_default();
    serialized.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
