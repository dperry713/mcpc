use clap::{Parser, Subcommand};
use crate::commands::{build, run, validate, clean, worker};
use crate::errors::McpcError;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mcpc")]
#[command(version, about = "The MCP Compiler (mcpc) for generating and orchestrating cloud-native architectures.", long_about = None)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Optional config file path
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Dry run (do not execute or modify files)
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Builds the MCP specification into rust modules and docker/helm configurations
    Build {
        /// Optional URL of a remote builder node (e.g. http://127.0.0.1:50051)
        #[arg(long)]
        remote: Option<String>,
    },
    /// Runs the default module or orchestration
    Run,
    /// Validates the mcp.spec.json against schemas and constraints
    Validate,
    /// Cleans the local cache and generated outputs
    Clean,
    /// Starts a background worker node for distributed compilation on port 50051
    Worker,
}

pub fn run_cli() -> Result<(), McpcError> {
    let cli = Cli::parse();
    crate::logging::init(cli.verbose);

    match cli.command {
        Commands::Build { remote } => build::execute(remote, cli.dry_run)?,
        Commands::Run => run::execute()?,
        Commands::Validate => validate::execute()?,
        Commands::Clean => clean::execute()?,
        Commands::Worker => worker::run_worker(50051)?,
    }

    Ok(())
}