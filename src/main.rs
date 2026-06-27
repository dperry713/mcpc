use mcpc::logging;
use mcpc::cli;
use mcpc::diagnostics;

fn main() {
    logging::init();
    
    if let Err(err) = cli::run_cli() {
        diagnostics::report_error(&err);
        std::process::exit(1);
    }
}
