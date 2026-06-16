//! ACP CLI - Inspector tool for ACP-Swarm
//!
//! Command-line tool for inspecting and managing Goals.
//!
//! ## Commands
//! - `goal list` — List all goals
//! - `goal status <id>` — Show goal status
//! - `goal submit <file>` — Submit a goal from file
//! - `worker list` — List all workers
//! - `graph summary` — Show goal graph summary

use clap::{Parser, Subcommand};
use acp_core::GoalGraphSummary;
use anyhow::Result;

/// ACP-Swarm CLI Inspector
#[derive(Parser)]
#[command(name = "acp-cli")]
#[command(about = "ACP-Swarm Goal Inspector", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Goal management commands
    Goal {
        #[command(subcommand)]
        command: GoalCommands,
    },
    /// Worker management commands
    Worker {
        #[command(subcommand)]
        command: WorkerCommands,
    },
    /// Goal graph commands
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
}

#[derive(Subcommand)]
enum GoalCommands {
    /// List all goals
    List,
    /// Show goal status
    Status {
        /// Goal ID
        id: String,
    },
    /// Submit a goal from file
    Submit {
        /// Path to .goal.yaml file
        file: String,
    },
}

#[derive(Subcommand)]
enum WorkerCommands {
    /// List all workers
    List,
    /// Show worker status
    Status {
        /// Worker ID
        id: String,
    },
}

#[derive(Subcommand)]
enum GraphCommands {
    /// Show goal graph summary
    Summary,
}

/// Main entry point
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Goal { command } => handle_goal_command(command),
        Commands::Worker { command } => handle_worker_command(command),
        Commands::Graph { command } => handle_graph_command(command),
    }
}

fn handle_goal_command(command: GoalCommands) -> Result<()> {
    match command {
        GoalCommands::List => {
            println!("Goal List:");
            println!("  (No goals registered — connect to ACP-Swarm backend)");
            println!("\n  Use 'acp-cli goal submit <file>' to submit a goal");
        }
        GoalCommands::Status { id } => {
            println!("Goal Status: {}", id);
            println!("  Status: Pending (no backend connection)");
            println!("  Use 'acp-cli goal submit' first");
        }
        GoalCommands::Submit { file } => {
            println!("Submitting goal from: {}", file);
            println!("  (Stub — connect to ACP-Swarm backend for real submission)");
            // TODO: Read .goal.yaml and parse
        }
    }
    Ok(())
}

fn handle_worker_command(command: WorkerCommands) -> Result<()> {
    match command {
        WorkerCommands::List => {
            println!("Worker List:");
            println!("  (No workers registered — connect to ACP-Swarm backend)");
        }
        WorkerCommands::Status { id } => {
            println!("Worker Status: {}", id);
            println!("  Status: Offline (no backend connection)");
        }
    }
    Ok(())
}

fn handle_graph_command(command: GraphCommands) -> Result<()> {
    match command {
        GraphCommands::Summary => {
            println!("Goal Graph Summary:");
            let summary = GoalGraphSummary::default();
            println!("  Total: {}", summary.total);
            println!("  Pending: {}", summary.pending);
            println!("  Active: {}", summary.active);
            println!("  Converged: {}", summary.converged);
            println!("  Failed: {}", summary.failed);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parse_goal_list() {
        let cli = Cli::try_parse_from(["acp-cli", "goal", "list"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn cli_parse_goal_status() {
        let cli = Cli::try_parse_from(["acp-cli", "goal", "status", "goal-001"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn cli_parse_worker_list() {
        let cli = Cli::try_parse_from(["acp-cli", "worker", "list"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn cli_parse_graph_summary() {
        let cli = Cli::try_parse_from(["acp-cli", "graph", "summary"]);
        assert!(cli.is_ok());
    }
}