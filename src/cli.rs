use clap::Parser;
use crate::commands::{build, run, validate, clean, worker};
use crate::errors::McpcError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Build(build::BuildArgs),
    Run,
    Validate,
    Clean,
    Worker,
}

pub fn run_cli() -> Result<(), McpcError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => build::execute(args)?,
        Commands::Run => run::execute()?,
        Commands::Validate => validate::execute()?,
        Commands::Clean => clean::execute()?,
        Commands::Worker => worker::run_worker(50051)?,
    }

    Ok(())
}