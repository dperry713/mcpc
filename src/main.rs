use mcpc::cli;
use miette::Result;

fn main() -> Result<()> {
    cli::run_cli()?;
    Ok(())
}