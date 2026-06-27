use clap::{Parser, Subcommand};

mod models;
use models::Project;
use models::StorageBackend;

#[derive(Parser)]
#[command(name = "project-tracker")]
#[command(about = "A CLI tool for tracking projects effortlessly for freelancers")]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        name: String,
    },
    /// Log hours to a project
    Log {
        project_name: String,
        hours: f64,
    },
    /// Rename a project
    Rename {
        old_project_name: String,
        new_project_name: String,
    },
    /// Delete a project
    Delete {
        project_name: String,
    },
    /// Show a project summary
    Summary {
        project_name: String
    },
    /// Watch a folder and track file activity
    Watch {
        project_name: String,
        path: String,
    },
}

fn handle_init(name: &str) -> anyhow::Result<()> {
    let mut project = Project::new(name);
    StorageBackend::save(&mut project)?;
    println!("Project '{}' initialized", name);

    Ok(())
}

fn handle_log(project_name: &str, hours: f64) -> anyhow::Result<()> {
    Project::log_hours(project_name, hours)?;

    Ok(())
}

fn handle_rename(old_project_name: &str, new_project_name: &str) -> anyhow::Result<()> {
    Project::rename_project(old_project_name, new_project_name)?;

    Ok(())
}

fn handle_delete(project_name: &str) -> anyhow::Result<()> {
    Project::delete_project(project_name)?;

    Ok(())
}

fn handle_summary(project_name: &str) -> anyhow::Result<()> {
    Project::summarize(project_name)?;

    Ok(())
}

fn handle_watch(project_name: &str, path: &str) -> anyhow::Result<()> {
    Project::start_watching(project_name, path)?;
    
    Ok(())
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => handle_init(&name)?,
        Commands::Log { project_name, hours } => handle_log(&project_name, hours)?,
        Commands::Rename { old_project_name, new_project_name } => handle_rename(&old_project_name, &new_project_name)?,
        Commands::Delete { project_name } => handle_delete(&project_name)?,
        Commands::Summary { project_name } => handle_summary(&project_name)?,
        Commands::Watch { project_name, path } => handle_watch(&project_name, &path)?,
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprint!("Error: {}", e);
        std::process::exit(1);
    }
}