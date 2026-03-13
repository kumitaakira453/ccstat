mod display;
mod parser;
mod stats;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ccstat", about = "Claude Code session statistics")]
struct Cli {
    /// Show per-session breakdown
    #[arg(long)]
    session: bool,

    /// Filter by project name
    #[arg(long)]
    project: Option<String>,

    /// Custom log directory path
    #[arg(long)]
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let projects_dir = match cli.path {
        Some(p) => p,
        None => {
            let home = dirs::home_dir().expect("Cannot find home directory");
            home.join(".claude").join("projects")
        }
    };

    if !projects_dir.exists() {
        eprintln!("Directory not found: {}", projects_dir.display());
        std::process::exit(1);
    }

    let projects = stats::collect_stats(&projects_dir, cli.project.as_deref());

    if cli.session {
        display::display_sessions(&projects);
    } else {
        display::display_summary(&projects);
    }
}
