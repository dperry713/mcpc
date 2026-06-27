use mcpc::cli;
use miette::Result;

fn main() -> Result<()> {
    if let Err(err) = cli::run_cli() {
        return Err(err.into());
    }
    
    Ok(())
}
