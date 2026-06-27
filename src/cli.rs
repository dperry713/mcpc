use clap::{Parser, Subcommand};
use crate::commands::{build, run, validate, clean, worker};
use crate::errors::McpcError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub config: Option<String>,

    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Build(build::BuildArgs),
    Run(run::RunArgs),
    Validate(validate::ValidateArgs),
    Clean(clean::CleanArgs),
    Worker(worker::WorkerArgs),
}

pub fn run_cli() -> Result<(), McpcError> {
    let cli = Cli::parse();
    crate::logging::init(cli.verbose);

    match cli.command {
        Commands::Build(args) => build::execute(args)?,
        Commands::Run(args) => run::execute(args)?,
        Commands::Validate(args) => validate::execute(args)?,
        Commands::Clean(args) => clean::execute(args)?,
        Commands::Worker(args) => worker::run_worker(args)?,
    }

    Ok(())
}