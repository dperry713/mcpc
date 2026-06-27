use mcpc::logging;
use mcpc::cli;

fn main() {
    logging::init();

    if let Err(err) = cli::run_cli() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}