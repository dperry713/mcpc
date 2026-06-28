use crate::schema::{MCPSpec, Module};
use crate::errors::McpcError;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_graph(spec: &MCPSpec) -> Result<Vec<&Module>, McpcError> {
    let mut module_map = BTreeMap::new();
    for module in &spec.modules {
        module_map.insert(module.name.clone(), module);
    }

    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    let mut sorted = Vec::new();

    // Iterate in deterministic sorted name order
    for name in module_map.keys() {
        if !visited.contains(name) {
            dfs(name, &module_map, &mut visited, &mut visiting, &mut sorted)?;
        }
    }

    Ok(sorted)
}

fn dfs<'a>(
    node: &str,
    module_map: &BTreeMap<String, &'a Module>,
    visited: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    sorted: &mut Vec<&'a Module>,
) -> Result<(), McpcError> {
    if visiting.contains(node) {
        return Err(McpcError::Build(format!("Circular dependency detected involving module: {}", node)));
    }
    if visited.contains(node) {
        return Ok(());
    }

    visiting.insert(node.to_string());

    if let Some(&module) = module_map.get(node) {
        // Traverse dependencies in deterministic sorted order
        let mut sorted_deps = module.dependencies.clone();
        sorted_deps.sort();

        for dep in &sorted_deps {
            if !module_map.contains_key(dep) {
                return Err(McpcError::Build(format!("Module '{}' depends on unknown module '{}'", node, dep)));
            }
            dfs(dep, module_map, visited, visiting, sorted)?;
        }
        
        visiting.remove(node);
        visited.insert(node.to_string());
        sorted.push(module);
        
        Ok(())
    } else {
        Err(McpcError::Build(format!("Module not found: {}", node)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_graph() {
        let spec = MCPSpec {
            project: "test".into(),
            stage: "development".into(),
            modules: vec![
                Module { name: "A".into(), module_type: None, entry: None, features: vec![], dependencies: vec!["B".into()], dependents: vec![], meta: None },
                Module { name: "B".into(), module_type: None, entry: None, features: vec![], dependencies: vec![], dependents: vec![], meta: None },
            ],
            meta: None,
            connections: vec![],
        };
        let sorted = build_graph(&spec).unwrap();
        assert_eq!(sorted[0].name, "B");
        assert_eq!(sorted[1].name, "A");
    }

    #[test]
    fn test_circular_dependency() {
        let spec = MCPSpec {
            project: "test".into(),
            stage: "development".into(),
            modules: vec![
                Module { name: "A".into(), module_type: None, entry: None, features: vec![], dependencies: vec!["B".into()], dependents: vec![], meta: None },
                Module { name: "B".into(), module_type: None, entry: None, features: vec![], dependencies: vec!["A".into()], dependents: vec![], meta: None },
            ],
            meta: None,
            connections: vec![],
        };
        let err = build_graph(&spec).unwrap_err();
        assert!(err.to_string().contains("Circular dependency detected"));
    }
}