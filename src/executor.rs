use crate::planner::BuildPlan;
use crate::generator;
use crate::errors::McpcError;
use std::fs;
use std::path::Path;

const OUTPUT_DIR: &str = "automata-mcp";

fn ensure_dirs() -> Result<(), McpcError> {
    fs::create_dir_all(format!("{}/.mcpc", OUTPUT_DIR))
        .map_err(|e| McpcError::Build(format!("Failed to create output dir: {}", e)))?;
    Ok(())
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
            let full_path = format!("{}/{}", OUTPUT_DIR, rel_path);
            let path = Path::new(&full_path);
            
            if dry_run {
                tracing::info!("     [DRY RUN] Would write {}", full_path);
                continue;
            }

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| McpcError::Build(format!("Failed to create dir for {}: {}", rel_path, e)))?;
            }

            fs::write(&full_path, content)
                .map_err(|e| McpcError::Build(format!("Failed to write {}: {}", rel_path, e)))?;
        }
    }

    Ok(())
}
