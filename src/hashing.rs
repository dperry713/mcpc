use crate::schema::Module;
use crate::generator;
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
    
    hasher.update(canonical.as_bytes());

    // 2. Hash the generated output (BTreeMap keys are already sorted deterministically)
    if let Ok(output) = generator::generate_module(module) {
        for (rel_path, content) in &output {
            hasher.update(rel_path.as_bytes());
            hasher.update(content.as_bytes());
        }
    }

    format!("{:x}", hasher.finalize())
}
