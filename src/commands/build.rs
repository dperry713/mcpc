use clap::Parser;
use crate::parser::load_spec;
use crate::cache::{load_cache, save_cache};
use crate::errors::McpcError;
use crate::validator;
use crate::graph;
use crate::planner;
use crate::executor;
use crate::manifest;
use crate::plugins;
use notify::{Watcher, RecursiveMode, EventKind};
use std::sync::mpsc;

#[derive(Parser)]
pub struct BuildArgs {
    #[arg(short, long)]
    pub remote: Option<String>,

    #[arg(short, long)]
    pub dry_run: bool,

    #[arg(short, long)]
    pub watch: bool,
}

pub fn execute(args: BuildArgs) -> Result<(), McpcError> {
    if args.watch {
        run_watch_loop(&args)?;
        Ok(())
    } else {
        run_build(args.remote.clone(), args.dry_run)
    }
}

fn run_build(remote: Option<String>, dry_run: bool) -> Result<(), McpcError> {
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

    executor::execute(&plan, remote, dry_run)?;

    if !dry_run {
        save_cache(&plan.new_cache)?;
        
        manifest::generate_manifest(&plan)?;

        let all_modules: Vec<_> = plan.build.iter().chain(plan.skip.iter()).copied().collect();
        
        // Generate docker-compose.yml
        let compose_content = crate::compose::render_docker_compose(all_modules.iter().copied());
        std::fs::write("automata-mcp/docker-compose.yml", compose_content)
            .map_err(|e| McpcError::Io(e))?;

        // Generate workspace Cargo.toml
        let mut workspace_toml = String::from("[workspace]\nresolver = \"2\"\nmembers = [\n");
        for m in &all_modules {
            workspace_toml.push_str(&format!("    \"{}\",\n", m.name));
        }
        workspace_toml.push_str("]\n");
        std::fs::write("automata-mcp/Cargo.toml", workspace_toml)
            .map_err(|e| McpcError::Io(e))?;
    } else {
        tracing::info!("[mcpc] DRY RUN: skipping manifest and workspace generation");
    }

    tracing::info!("[mcpc] build complete");
    Ok(())
}

fn run_watch_loop(args: &BuildArgs) -> Result<(), McpcError> {
    // Initial build
    if let Err(e) = run_build(args.remote.clone(), args.dry_run) {
        tracing::error!("[mcpc] initial build failed: {}", e);
    }

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).map_err(|e| McpcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    watcher.watch(std::path::Path::new("mcp.spec.json"), RecursiveMode::NonRecursive)
        .map_err(|e| McpcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
    let templates_path = std::path::Path::new(".mcpc/templates");
    if templates_path.exists() {
        watcher.watch(templates_path, RecursiveMode::Recursive)
            .map_err(|e| McpcError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    }

    tracing::info!("[mcpc] watching for changes...");

    for res in rx {
        match res {
            Ok(event) => {
                // Ignore Access/Any events, mostly look for Modify/Create/Remove
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                        tracing::info!("[mcpc] detected change, rebuilding...");
                        if let Err(e) = run_build(args.remote.clone(), args.dry_run) {
                            tracing::error!("[mcpc] build failed: {}", e);
                        }
                    },
                    _ => {}
                }
            }
            Err(e) => tracing::error!("[mcpc] watch error: {:?}", e),
        }
    }

    Ok(())
}