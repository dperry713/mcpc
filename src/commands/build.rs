use crate::parser::load_spec;
use crate::cache::{load_cache, save_cache};
use crate::errors::McpcError;
use crate::validator;
use crate::graph;
use crate::planner;
use crate::executor;
use crate::manifest;
use crate::plugins;

pub fn execute(remote: Option<String>) -> Result<(), McpcError> {
    tracing::info!("[mcpc] build started");

    let spec = load_spec("mcp.spec.json")?;
    
    let discovered_plugins = plugins::discover_plugins();
    let spec_json = serde_json::to_value(&spec).map_err(McpcError::Serialization)?;
    for plugin in &discovered_plugins {
        tracing::info!("[mcpc] Running pre_validate hook for plugin: {}", plugin.name);
        let res = plugins::run_plugin(plugin, "pre_validate", spec_json.clone())?;
        if res.success {
            if let Some(data) = res.data {
                tracing::info!("[mcpc] Plugin {} returned: {}", plugin.name, data);
            }
        } else if let Some(err) = res.error {
            tracing::error!("[mcpc] Plugin {} error: {}", plugin.name, err);
            return Err(McpcError::Validation(format!("Plugin {} rejected spec: {}", plugin.name, err)));
        }
    }

    validator::validate(&spec)?;

    let sorted_modules = graph::build_graph(&spec)?;

    let old_cache = load_cache();
    let plan = planner::plan(&sorted_modules, &old_cache);

    if plan.build.is_empty() {
        tracing::info!("[mcpc] no changes detected");
        return Ok(());
    }

    tracing::info!("[mcpc] building {} modules, skipping {}", plan.build.len(), plan.skip.len());

    executor::execute(&plan, remote)?;

    save_cache(&plan.new_cache)?;
    
    manifest::generate_manifest(&plan)?;

    let all_modules = plan.build.iter().chain(plan.skip.iter()).copied();
    let compose_content = crate::compose::render_docker_compose(all_modules);
    std::fs::write("automata-mcp/docker-compose.yml", compose_content)
        .map_err(|e| McpcError::Io(e))?;

    tracing::info!("[mcpc] build complete");

    Ok(())
}