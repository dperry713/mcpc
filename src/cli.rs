use std::env;
use crate::commands::{build, run, validate, clean, worker};
use crate::errors::McpcError;

pub fn run_cli() -> Result<(), McpcError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "build" => {
            let mut remote = None;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--remote" && i + 1 < args.len() {
                    remote = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            build::execute(remote)?
        },
        "run" => run::execute()?,
        "validate" => validate::execute()?,
        "clean" => clean::execute()?,
        "worker" => worker::run_worker(50051)?,
        "--help" | "-h" => print_help(),
        _ => {
            tracing::error!("Unknown command: {}", args[1]);
            print_help();
        }
    }

    Ok(())
}

fn print_help() {
    println!(
        r#"
mcpc CLI

Usage:
  mcpc build [--remote <URL>]
  mcpc run
  mcpc validate
  mcpc clean
  mcpc worker
"#
    );
}