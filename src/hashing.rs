use crate::schema::Module;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::generator;

pub fn compute_module_hash(module: &Module) -> String {
    let mut hasher = DefaultHasher::new();
    
    // Hash the module definition
    let serialized = serde_json::to_string(module).unwrap_or_default();
    serialized.hash(&mut hasher);

    // Hash the generated output (which captures all relevant template contents and features)
    if let Ok(output) = generator::generate_module(module) {
        let mut keys: Vec<&String> = output.keys().collect();
        keys.sort(); // Ensure deterministic order
        for k in keys {
            k.hash(&mut hasher);
            output[k].hash(&mut hasher);
        }
    }

    format!("{:x}", hasher.finish())
}
