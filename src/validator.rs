use crate::schema::MCPSpec;
use crate::errors::McpcError;
use crate::diagnostics::{Diagnostic, report_diagnostic_warning};
use std::collections::HashSet;

pub fn validate(spec: &MCPSpec) -> Result<(), McpcError> {
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
