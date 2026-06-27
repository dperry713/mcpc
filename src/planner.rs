use crate::schema::Module;
use crate::cache::Cache;
use crate::hashing::compute_module_hash;
use std::collections::HashSet;

pub struct BuildPlan<'a> {
    pub build: Vec<&'a Module>,
    pub skip: Vec<&'a Module>,
    pub new_cache: Cache,
}

pub fn plan<'a>(sorted_modules: &[&'a Module], old_cache: &Cache) -> BuildPlan<'a> {
    let mut build = Vec::new();
    let mut skip = Vec::new();
    let mut new_cache = Cache::new();
    
    let mut rebuilt_modules = HashSet::new();

    for &module in sorted_modules {
        let current_hash = compute_module_hash(module);
        new_cache.insert(module.name.clone(), current_hash.clone());

        let mut needs_build = false;
        
        if old_cache.get(&module.name) != Some(&current_hash) {
            needs_build = true;
        }

        if !needs_build {
            for dep in &module.dependencies {
                if rebuilt_modules.contains(dep) {
                    needs_build = true;
                    break;
                }
            }
        }

        if needs_build {
            build.push(module);
            rebuilt_modules.insert(module.name.clone());
        } else {
            skip.push(module);
        }
    }

    BuildPlan {
        build,
        skip,
        new_cache,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_incremental() {
        let module_a = Module { name: "A".into(), module_type: None, entry: None, features: vec![], dependencies: vec![] };
        let module_b = Module { name: "B".into(), module_type: None, entry: None, features: vec![], dependencies: vec!["A".into()] };
        
        let sorted = vec![&module_a, &module_b];
        
        // Initial build
        let cache1 = Cache::new();
        let plan1 = plan(&sorted, &cache1);
        assert_eq!(plan1.build.len(), 2);
        assert_eq!(plan1.skip.len(), 0);
        
        // Unchanged build
        let plan2 = plan(&sorted, &plan1.new_cache);
        assert_eq!(plan2.build.len(), 0);
        assert_eq!(plan2.skip.len(), 2);
    }
}
