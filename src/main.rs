use anyhow::Result;
use clap::{Parser, Subcommand};

mod models;
use models::Project;

#[derive(Parser)]
#[command(
    name = "ptracker",
    about = "A CLI tool for tracking projects, file activity and hours for freelancers",
    long_about = "ptracker tracks your freelance projects in real time.\n\nIt watches folders for file activity, logs sessions with start/end times,\nand keeps everything in ~/.ptracker so your data is available globally.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        name: String,
        /// Specify project watch path
        #[arg(short, long, help = "Folder to watch (defaults to current directory)")]
        path: Option<String>,
    },
    /// List all projects
    List,
    /// Log hours manually to a project
    Log {
        name: String,
        hours: f64,
    },
    /// Rename a project
    Rename {
        old_name: String,
        new_name: String,
    },
    /// Delete a project
    Delete {
        name: String,
    },
    /// Show project summary
    Summary {
        name: String,
    },
   /// Watch a folder and track file activity
    Watch {
        name: String,
        /// Override watch path for this session
        #[arg(short, long, help = "Override the watch path for this session")]
        path: Option<String>,
    },
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, path } =>
            Project::init(&name, path.as_deref())?,
        Commands::List =>
            Project::list()?,
        Commands::Log { name, hours } =>
            Project::log_hours(&name, hours)?,
        Commands::Rename { old_name, new_name } =>
            Project::rename(&old_name, &new_name)?,
        Commands::Delete { name } =>
            Project::delete(&name)?,
        Commands::Summary { name } =>
            Project::summary(&name)?,
        Commands::Watch { name, path } =>
            Project::watch(&name, path.as_deref())?,
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprint!("Error: {}", e);
        std::process::exit(1);
    }
}