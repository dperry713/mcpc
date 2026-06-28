use crate::schema::MCPSpec;
use crate::errors::McpcError;
use crate::diagnostics::{Diagnostic, report_diagnostic_warning};
use std::collections::HashSet;
use jsonschema::{JSONSchema, Draft};
use serde_json::json;

fn validate_meta_value(value: &serde_json::Value, path: &str) -> Result<(), McpcError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let current_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                
                let check_key = key.to_lowercase();
                if check_key.contains("tenant") || 
                   check_key.contains("role") || 
                   check_key.contains("privilege") || 
                   check_key.contains("admin") || 
                   check_key.contains("permission") || 
                   check_key.contains("state") || 
                   check_key.contains("security") || 
                   check_key.contains("auth") {
                    
                    match val {
                        serde_json::Value::String(s) => {
                            let check_val = s.to_lowercase();
                            if check_val.contains("admin") || 
                               check_val.contains("superuser") || 
                               check_val.contains("root") || 
                               check_val.contains("system") || 
                               check_val.contains("write-all") {
                                return Err(McpcError::Validation(format!(
                                    "Security Violation: Unauthorized state change injection in _meta key '{}' = '{}'",
                                    current_path, s
                                )));
                            }
                        }
                        serde_json::Value::Bool(b) => {
                            if *b && (check_key.contains("admin") || check_key.contains("privilege") || check_key.contains("auth") || check_key.contains("security")) {
                                return Err(McpcError::Validation(format!(
                                    "Security Violation: Unauthorized state change injection in _meta key '{}' = true",
                                    current_path
                                )));
                            }
                        }
                        _ => {}
                    }
                }
                validate_meta_value(val, &current_path)?;
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                validate_meta_value(val, &format!("{}[{}]", path, i))?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn validate(spec: &MCPSpec) -> Result<(), McpcError> {
    // 1. JSON Schema Validation
    let schema_json = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["project", "modules"],
        "properties": {
            "project": { "type": "string" },
            "stage": { "type": ["string", "null"], "enum": ["development", "testing", "production", null] },
            "_meta": { "type": ["object", "null"] },
            "connections": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["name", "url", "auth_flow", "pkce", "audience", "scope"],
                    "properties": {
                        "name": { "type": "string" },
                        "url": { "type": "string" },
                        "auth_flow": { "type": "string" },
                        "pkce": { "type": "boolean" },
                        "audience": { "type": "string" },
                        "scope": { "type": "string" },
                        "jit_escalation": { "type": "boolean" }
                    }
                }
            },
            "modules": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
                        "type": { "type": ["string", "null"], "enum": ["api", "worker", "agent", "default", "plugin", "tool", "app", "unknown", null] },
                        "entry": { "type": ["string", "null"] },
                        "features": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "dependencies": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "_meta": { "type": ["object", "null"] }
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

    // 5. Remote Connection Validation
    for conn in &spec.connections {
        if conn.auth_flow != "oauth-2.1" && conn.auth_flow != "oauth-2.1-pkce" {
            return Err(McpcError::Validation(format!(
                "Security Violation: Remote connection '{}' uses non-compliant auth flow '{}'. OAuth 2.1 is mandated.",
                conn.name, conn.auth_flow
            )));
        }
        if !conn.pkce {
            return Err(McpcError::Validation(format!(
                "Security Violation: Remote connection '{}' does not enable PKCE. PKCE is mandated for all remote connections.",
                conn.name
            )));
        }
        if conn.audience.is_empty() || conn.audience == "*" {
            return Err(McpcError::Validation(format!(
                "Security Violation: Remote connection '{}' has legacy broad/wildcard audience '{}'. Per-tool audience-bound tokens are required.",
                conn.name, conn.audience
            )));
        }
        
        let high_privilege_scopes = ["admin", "write", "delete", "sudo", "root"];
        let needs_jit = high_privilege_scopes.iter().any(|&s| conn.scope.contains(s));
        if needs_jit && !conn.jit_escalation {
            return Err(McpcError::Validation(format!(
                "Security Violation: High-privilege connection '{}' (scope: '{}') requires Just-In-Time (JIT) permission escalation (jit_escalation: true).",
                conn.name, conn.scope
            )));
        }
    }

    // 6. Metadata Sanitization & Validation
    if let Some(ref meta) = spec.meta {
        validate_meta_value(meta, "")?;
    }
    for module in &spec.modules {
        if let Some(ref meta) = module.meta {
            validate_meta_value(meta, &format!("module['{}']._meta", module.name))?;
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
    use crate::schema::{Module, RemoteConnection};
    use serde_json::json;

    #[test]
    fn test_validator_detects_duplicates() {
        let spec = MCPSpec {
            project: "test".into(),
            stage: "development".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec![], meta: None },
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec![], meta: None },
            ],
            meta: None,
            connections: vec![],
        };
        let err = validate(&spec).unwrap_err();
        assert!(err.to_string().contains("Duplicate module name detected"));
    }

    #[test]
    fn test_validator_detects_orphans() {
        let spec = MCPSpec {
            project: "test".into(),
            stage: "development".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["B".into()], meta: None },
            ],
            meta: None,
            connections: vec![],
        };
        let err = validate(&spec).unwrap_err();
        assert!(err.to_string().contains("Orphaned dependency detected"));
    }

    #[test]
    fn test_validator_detects_cycles() {
        let spec = MCPSpec {
            project: "test".into(),
            stage: "development".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["B".into()], meta: None },
                Module { name: "B".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["C".into()], meta: None },
                Module { name: "C".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec!["A".into()], meta: None },
            ],
            meta: None,
            connections: vec![],
        };
        let err = validate(&spec).unwrap_err();
        assert!(err.to_string().contains("Circular dependency detected"));
    }

    #[test]
    fn test_validator_connections_security() {
        // 1. Safe connection
        let mut spec = MCPSpec {
            project: "test".into(),
            stage: "development".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec![], meta: None },
            ],
            meta: None,
            connections: vec![
                RemoteConnection {
                    name: "conn1".into(),
                    url: "https://api.test".into(),
                    auth_flow: "oauth-2.1".into(),
                    pkce: true,
                    audience: "some-tool-aud".into(),
                    scope: "admin:read".into(),
                    jit_escalation: true,
                }
            ],
        };
        assert!(validate(&spec).is_ok());

        // 2. Legacy auth flow rejected
        spec.connections[0].auth_flow = "implicit-legacy".into();
        assert!(validate(&spec).is_err());
        spec.connections[0].auth_flow = "oauth-2.1".into();

        // 3. PKCE false rejected
        spec.connections[0].pkce = false;
        assert!(validate(&spec).is_err());
        spec.connections[0].pkce = true;

        // 4. Wildcard audience rejected
        spec.connections[0].audience = "*".into();
        assert!(validate(&spec).is_err());
        spec.connections[0].audience = "some-tool-aud".into();

        // 5. High-privilege scope without JIT rejected
        spec.connections[0].scope = "sudo:write".into();
        spec.connections[0].jit_escalation = false;
        assert!(validate(&spec).is_err());
        spec.connections[0].jit_escalation = true;
        assert!(validate(&spec).is_ok());
    }

    #[test]
    fn test_validator_metadata_sanitization() {
        let mut spec = MCPSpec {
            project: "test".into(),
            stage: "development".into(),
            modules: vec![
                Module { name: "A".into(), module_type: Some("default".into()), entry: None, features: vec![], dependencies: vec![], meta: None },
            ],
            meta: None,
            connections: vec![],
        };

        // 1. Safe metadata
        spec.meta = Some(json!({
            "tenant": "client-123",
            "environment": "sandbox",
            "debug": true
        }));
        assert!(validate(&spec).is_ok());

        // 2. Malicious root level _meta: tenant = admin
        spec.meta = Some(json!({
            "tenant": "admin",
            "environment": "sandbox"
        }));
        let err = validate(&spec).unwrap_err();
        assert!(err.to_string().contains("Security Violation: Unauthorized state change injection"));

        // Reset safe
        spec.meta = Some(json!({ "tenant": "safe-tenant" }));
        assert!(validate(&spec).is_ok());

        // 3. Malicious nested admin privileges
        spec.meta = Some(json!({
            "nested": {
                "super_privileges": {
                    "role": "SuperUser"
                }
            }
        }));
        let err2 = validate(&spec).unwrap_err();
        assert!(err2.to_string().contains("Security Violation: Unauthorized state change injection"));

        // Reset safe
        spec.meta = None;

        // 4. Malicious module level admin: true
        spec.modules[0].meta = Some(json!({
            "admin": true
        }));
        let err3 = validate(&spec).unwrap_err();
        assert!(err3.to_string().contains("Security Violation: Unauthorized state change injection"));
    }
}
