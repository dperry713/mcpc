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

use std::time::Duration;

pub fn run_plugin(plugin: &Plugin, hook: &str, payload: serde_json::Value) -> Result<PluginResponse, McpcError> {
    let request = PluginRequest {
        hook: hook.to_string(),
        payload,
    };
    
    if plugin.path.extension().and_then(|s| s.to_str()) == Some("wasm") {
        return run_wasm_plugin(&plugin.path, &request);
    }

    run_host_plugin(&plugin.path, &plugin.name, &request)
}

fn run_wasm_plugin(plugin_path: &std::path::Path, request: &PluginRequest) -> Result<PluginResponse, McpcError> {
    use wasmi::{Engine, Linker, Module, Store};

    let wasm_bytes = std::fs::read(plugin_path).map_err(McpcError::Io)?;

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes)
        .map_err(|e| McpcError::Build(format!("Failed to compile WASM module: {}", e)))?;

    // Store is configured with empty state.
    // Linker has no imports (no filesystem, env, process or network access).
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);

    let instance = linker.instantiate(&mut store, &module)
        .map_err(|e| McpcError::Build(format!("Failed to instantiate WASM: {}", e)))?
        .start(&mut store)
        .map_err(|e| McpcError::Build(format!("WASM start failed: {}", e)))?;

    let memory = instance.get_memory(&store, "memory")
        .ok_or_else(|| McpcError::Build("WASM plugin does not export 'memory'".into()))?;

    let malloc_func = instance.get_typed_func::<i32, i32>(&store, "malloc")
        .map_err(|e| McpcError::Build(format!("WASM plugin does not export 'malloc': {}", e)))?;

    let run_plugin_func = instance.get_typed_func::<(i32, i32), i64>(&store, "run_plugin")
        .map_err(|e| McpcError::Build(format!("WASM plugin does not export 'run_plugin': {}", e)))?;

    let request_json = serde_json::to_string(request).map_err(McpcError::Serialization)?;
    let request_bytes = request_json.as_bytes();
    let request_len = request_bytes.len() as i32;

    let request_ptr = malloc_func.call(&mut store, request_len)
        .map_err(|e| McpcError::Build(format!("WASM malloc failed: {}", e)))?;

    memory.write(&mut store, request_ptr as usize, request_bytes)
        .map_err(|e| McpcError::Build(format!("Failed to write to WASM memory: {}", e)))?;

    let result = run_plugin_func.call(&mut store, (request_ptr, request_len))
        .map_err(|e| McpcError::Build(format!("WASM run_plugin execution failed: {}", e)))?;

    let response_ptr = ((result as u64) >> 32) as usize;
    let response_len = (result & 0xFFFF_FFFF) as usize;

    let mut response_bytes = vec![0u8; response_len];
    memory.read(&store, response_ptr, &mut response_bytes)
        .map_err(|e| McpcError::Build(format!("Failed to read from WASM memory: {}", e)))?;

    let response_str = String::from_utf8(response_bytes)
        .map_err(|e| McpcError::Build(format!("WASM returned invalid UTF-8: {}", e)))?;

    let response: PluginResponse = serde_json::from_str(&response_str)
        .map_err(|e| McpcError::Build(format!("Failed to parse WASM plugin response: {}", e)))?;

    Ok(response)
}

fn run_host_plugin(plugin_path: &std::path::Path, plugin_name: &str, request: &PluginRequest) -> Result<PluginResponse, McpcError> {
    let req_json = serde_json::to_string(request).map_err(McpcError::Serialization)?;
    
    let mut cmd = if plugin_path.extension().and_then(|s| s.to_str()) == Some("py") {
        let mut c = Command::new("python");
        c.arg(plugin_path);
        c
    } else {
        Command::new(plugin_path)
    };

    // ISOLATION: Clear environment variables
    cmd.env_clear();
    
    cmd.stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| McpcError::Build(format!("Failed to start plugin {}: {}", plugin_name, e)))?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(req_json.as_bytes()).map_err(McpcError::Io)?;
    }

    let mut stdout = child.stdout.take().ok_or_else(|| McpcError::Build("Failed to open stdout".into()))?;
    let (tx, rx) = std::sync::mpsc::channel();
    let max_bytes = 10 * 1024 * 1024; // 10MB
    
    use std::io::Read;
    std::thread::spawn(move || {
        let mut output_bytes = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            match stdout.read(&mut buffer) {
                Ok(0) => {
                    let _ = tx.send(Ok(output_bytes));
                    break;
                }
                Ok(n) => {
                    if output_bytes.len() + n > max_bytes {
                        let _ = tx.send(Err(McpcError::Build("Plugin exceeded maximum output size limit of 10MB".into())));
                        break;
                    }
                    output_bytes.extend_from_slice(&buffer[..n]);
                }
                Err(e) => {
                    let _ = tx.send(Err(McpcError::Io(e)));
                    break;
                }
            }
        }
    });

    let result = match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(res) => res,
        Err(_) => {
            let _ = child.kill();
            return Err(McpcError::Build(format!("Plugin {} timed out after 5 seconds", plugin_name)));
        }
    };
    
    let output_bytes = result?;
    
    let status = child.wait().map_err(McpcError::Io)?;
    if !status.success() {
        let mut stderr_content = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut stderr_content);
        }
        return Err(McpcError::Build(format!(
            "Plugin {} exited with status {}. Stderr: {}",
            plugin_name, status, stderr_content
        )));
    }

    let res_json = String::from_utf8(output_bytes)
        .map_err(|e| McpcError::Build(format!("Plugin output is invalid UTF-8: {}", e)))?;
    
    let response: PluginResponse = serde_json::from_str(&res_json)
        .map_err(|e| McpcError::Build(format!("Failed to parse plugin output: {}", e)))?;
        
    Ok(response)
}
