use crate::schema::MCPSpec;
use crate::errors::McpcError;
use crate::diagnostics::{Diagnostic, report_diagnostic_warning};
use std::collections::HashSet;
use jsonschema::{JSONSchema, Draft};
use serde_json::json;

pub fn validate(spec: &MCPSpec) -> Result<(), McpcError> {
    // 1. JSON Schema Validation
    let schema_json = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["project", "modules"],
        "properties": {
            "project": { "type": "string" },
            "modules": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
                        "type": { "type": "string", "enum": ["api", "worker", "agent", "default"] },
                        "entry": { "type": ["string", "null"] },
                        "features": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "dependencies": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    }
                }
            }
        }
    });

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| McpcError::Validation(format!("Invalid schema: {}", e)))?;

    let spec_json = serde_json::to_value(spec).map_err(McpcError::Serialization)?;
    if let Err(errors) = compiled.validate(&spec_json) {
        let mut err_msg = String::from("JSON Schema validation failed:\n");
        for error in errors {
            err_msg.push_str(&format!("  - {}\n", error));
        }
        return Err(McpcError::Validation(err_msg));
    }

    // 2. Custom rules
    if spec.modules.is_empty() {
        return Err(McpcError::Validation("Spec must contain at least one module".into()));
    }

    let mut names = HashSet::new();
    for module in &spec.modules {
        if !names.insert(&module.name) {
            return Err(McpcError::Validation(format!("Duplicate module name detected: {}", module.name)));
        }
        
        for feature in &module.features {
            if feature.is_empty() {
                return Err(McpcError::Validation(format!("Module '{}' contains an empty feature string", module.name)));
            }
        }

        if module.features.is_empty() {
            report_diagnostic_warning(&Diagnostic {
                message: format!("Module '{}' has no features configured", module.name),
                file: Some("mcp.spec.json".into()),
                help: Some("Consider adding features or double checking the spec".into()),
            });
        }
    }

    // 3. Strict DAG Validation (Orphan check)
    for module in &spec.modules {
        for dep in &module.dependencies {
            if !names.contains(dep) {
                return Err(McpcError::Validation(format!(
                    "Orphaned dependency detected: Module '{}' depends on undefined module '{}'",
                    module.name, dep
                )));
            }
        }
    }

    // 4. Strict DAG Validation (Cycle check)
    let mut adj = std::collections::BTreeMap::new();
    for module in &spec.modules {
        adj.insert(module.name.clone(), &module.dependencies);
    }

    let mut visiting = std::collections::BTreeSet::new();
    let mut visited = std::collections::BTreeSet::new();
    let mut path = Vec::new();

    for module in &spec.modules {
        if !visited.contains(&module.name) {
            check_cycle(&module.name, &adj, &mut visiting, &mut visited, &mut path)?;
        }
    }

    Ok(())
}

fn check_cycle(
    node: &str,
    adj: &std::collections::BTreeMap<String, &Vec<String>>,
    visiting: &mut std::collections::BTreeSet<String>,
    visited: &mut std::collections::BTreeSet<String>,
    path: &mut Vec<String>,
) -> Result<(), McpcError> {
    if visiting.contains(node) {
        let start_idx = path.iter().position(|x| x == node).unwrap_or(0);
        let cycle = &path[start_idx..];
        return Err(McpcError::Validation(format!(
            "Circular dependency detected: {} -> {}",
            cycle.join(" -> "),
            node
        )));
    }
    if visited.contains(node) {
        return Ok(());
    }

    visiting.insert(node.to_string());
    path.push(node.to_string());

    if let Some(&deps) = adj.get(node) {
        for dep in deps {
            check_cycle(dep, adj, visiting, visited, path)?;
        }
    }

    path.pop();
    visiting.remove(node);
    visited.insert(node.to_string());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Module;

    #[test]
    fn test_validator_detects_duplicates() {
        let spec = MCPSpec {
            project: "test".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec![] },
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec![] },
            ],
        };
        let err = validate(&spec).unwrap_err();
        assert!(err.to_string().contains("Duplicate module name detected"));
    }

    #[test]
    fn test_validator_detects_orphans() {
        let spec = MCPSpec {
            project: "test".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["B".into()] },
            ],
        };
        let err = validate(&spec).unwrap_err();
        assert!(err.to_string().contains("Orphaned dependency detected"));
    }

    #[test]
    fn test_validator_detects_cycles() {
        let spec = MCPSpec {
            project: "test".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["B".into()] },
                Module { name: "B".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["C".into()] },
                Module { name: "C".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["A".into()] },
            ],
        };
        let err = validate(&spec).unwrap_err();
        assert!(err.to_string().contains("Circular dependency detected"));
    }
}
