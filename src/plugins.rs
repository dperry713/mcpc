use crate::errors::McpcError;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginRequest {
    pub hook: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginResponse {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub path: PathBuf,
}

pub fn discover_plugins() -> Vec<Plugin> {
    let mut plugins = Vec::new();
    
    let plugin_dir = std::path::Path::new(".mcpc/plugins");
    if plugin_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if file_name.starts_with("mcpc-plugin-") {
                        plugins.push(Plugin {
                            name: file_name.to_string(),
                            path,
                        });
                    }
                }
            }
        }
    }
    plugins
}

pub fn run_plugin(plugin: &Plugin, hook: &str, payload: serde_json::Value) -> Result<PluginResponse, McpcError> {
    let request = PluginRequest {
        hook: hook.to_string(),
        payload,
    };
    
    let req_json = serde_json::to_string(&request).map_err(McpcError::Serialization)?;
    
    let mut cmd = if plugin.path.extension().and_then(|s| s.to_str()) == Some("py") {
        let mut c = Command::new("python");
        c.arg(&plugin.path);
        c
    } else {
        Command::new(&plugin.path)
    };

    cmd.stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::inherit());

    let mut child = cmd.spawn().map_err(|e| McpcError::Build(format!("Failed to start plugin {}: {}", plugin.name, e)))?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(req_json.as_bytes()).map_err(McpcError::Io)?;
    }

    let output = child.wait_with_output().map_err(McpcError::Io)?;
    
    if !output.status.success() {
        return Err(McpcError::Build(format!("Plugin {} exited with {}", plugin.name, output.status)));
    }

    let res_json = String::from_utf8_lossy(&output.stdout);
    
    let response: PluginResponse = serde_json::from_str(&res_json)
        .map_err(|e| McpcError::Build(format!("Failed to parse plugin output: {}", e)))?;
        
    Ok(response)
}
