use mcpc::parser;
use mcpc::schema::{MCPSpec, Module};
use std::path::{Path, PathBuf};

fn find_spec_path() -> Result<PathBuf, String> {
    let candidates = [
        "../../mcp.spec.json",
        "../mcp.spec.json",
        "mcp.spec.json"
    ];
    for c in candidates {
        let p = Path::new(c);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }
    Err("Could not find mcp.spec.json".into())
}

#[tauri::command]
fn get_spec() -> Result<MCPSpec, String> {
    let path = find_spec_path()?;
    parser::load_spec(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_spec(spec: MCPSpec) -> Result<(), String> {
    let path = find_spec_path()?;
    let json = serde_json::to_string_pretty(&spec).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    
    // Also run validation
    mcpc::validator::validate(&spec).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_graph() -> Result<Vec<Module>, String> {
    let spec = get_spec()?;
    let sorted_refs = mcpc::graph::build_graph(&spec).map_err(|e| e.to_string())?;
    Ok(sorted_refs.into_iter().cloned().collect())
}

#[tauri::command]
fn run_cli_command(command: &str) -> Result<String, String> {
    // In a full app, we could pipe tracing output here.
    // For now, we invoke the native library directly for fast, reliable execution.
    std::env::set_current_dir("../..").map_err(|e| e.to_string())?; // Ensure we are in workspace root
    
    match command {
        "build" => {
            let args = mcpc::commands::build::BuildArgs { remote: None, dry_run: false, watch: false };
            mcpc::commands::build::execute(args).map_err(|e| e.to_string())?;
            Ok("Build completed successfully.\nCheck your automata-mcp/ directory.".to_string())
        },
        "validate" => {
            let spec = mcpc::parser::load_spec("mcp.spec.json").map_err(|e| e.to_string())?;
            mcpc::validator::validate(&spec).map_err(|e| e.to_string())?;
            Ok("Validation passed. Spec is structurally sound.".to_string())
        },
        "clean" => {
            if std::path::Path::new("automata-mcp").exists() {
                std::fs::remove_dir_all("automata-mcp").map_err(|e| e.to_string())?;
                Ok("Workspace cleaned. Removed automata-mcp/".to_string())
            } else {
                Ok("Nothing to clean.".to_string())
            }
        },
        _ => Err(format!("Command '{}' not yet implemented in GUI.", command))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_spec, save_spec, get_graph, run_cli_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
