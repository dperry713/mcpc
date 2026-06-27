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

    Ok(())
}
