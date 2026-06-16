//! ACP CLI Binary Entry Point

use acp_cli::run;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}