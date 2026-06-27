use crate::planner::BuildPlan;
use crate::generator;
use crate::errors::McpcError;
use std::fs;
use std::path::{Path, PathBuf};

const OUTPUT_DIR: &str = "automata-mcp";

fn ensure_dirs() -> Result<(), McpcError> {
    fs::create_dir_all(format!("{}/.mcpc", OUTPUT_DIR))
        .map_err(|e| McpcError::Build(format!("Failed to create output dir: {}", e)))?;
    Ok(())
}

fn check_sandbox_boundary(base: &Path, rel_path: &str) -> Result<PathBuf, McpcError> {
    let path = Path::new(rel_path);
    if path.is_absolute() || rel_path.starts_with('/') || rel_path.starts_with('\\') {
        return Err(McpcError::Build(format!("Security Violation: Path '{}' is absolute", rel_path)));
    }

    let mut depth = 0;
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) => depth += 1,
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(McpcError::Build(format!(
                        "Security Violation: Path '{}' attempts to escape sandbox directory '{}'",
                        rel_path, base.display()
                    )));
                }
            }
            std::path::Component::CurDir => {}
            _ => {}
        }
    }

    Ok(base.join(path))
}

pub fn execute(plan: &BuildPlan, remote: Option<String>, dry_run: bool) -> Result<(), McpcError> {
    if !dry_run {
        ensure_dirs()?;
    }

    let client = if remote.is_some() {
        Some(reqwest::blocking::Client::new())
    } else {
        None
    };

    for module in &plan.build {
        tracing::info!("  -> generating {}", module.name);
        
        let module_output = if let (Some(url), Some(client)) = (&remote, &client) {
            tracing::info!("     (dispatching to remote builder at {})", url);
            let endpoint = format!("{}/build", url);
            
            let res = client.post(&endpoint)
                .json(module)
                .send()
                .map_err(|e| McpcError::Build(format!("Remote build failed: {}", e)))?;

            if !res.status().is_success() {
                return Err(McpcError::Build(format!("Remote builder returned error: {}", res.status())));
            }

            res.json::<generator::ModuleOutput>()
                .map_err(|e| McpcError::Build(format!("Failed to parse remote response: {}", e)))?
        } else {
            generator::generate_module(module)?
        };

        for (rel_path, content) in module_output {
            let full_path = check_sandbox_boundary(Path::new(OUTPUT_DIR), &rel_path)?;
            
            if dry_run {
                tracing::info!("     [DRY RUN] Would write {}", full_path.display());
                continue;
            }

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| McpcError::Build(format!("Failed to create dir for {}: {}", rel_path, e)))?;
            }

            fs::write(&full_path, content)
                .map_err(|e| McpcError::Build(format!("Failed to write {}: {}", rel_path, e)))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_path_absolute() {
        let base = Path::new("automata-mcp");
        let err = check_sandbox_boundary(base, "/etc/passwd").unwrap_err();
        assert!(err.to_string().contains("Security Violation"));
        assert!(err.to_string().contains("absolute"));
    }

    #[test]
    fn test_sandbox_path_traversal() {
        let base = Path::new("automata-mcp");
        let err = check_sandbox_boundary(base, "../escaping/file").unwrap_err();
        assert!(err.to_string().contains("Security Violation"));
        assert!(err.to_string().contains("escape"));
    }

    #[test]
    fn test_sandbox_path_nested_traversal() {
        let base = Path::new("automata-mcp");
        // Nested that goes up but stays inside: ok
        let path = check_sandbox_boundary(base, "a/b/../../c").unwrap();
        assert_eq!(path, base.join("a/b/../../c"));
    }

    #[test]
    fn test_sandbox_path_deep_escaping() {
        let base = Path::new("automata-mcp");
        // Deep escaping: error
        let err = check_sandbox_boundary(base, "a/../../../escaping").unwrap_err();
        assert!(err.to_string().contains("Security Violation"));
    }
}
