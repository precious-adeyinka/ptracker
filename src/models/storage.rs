use std::path::PathBuf;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufWriter;
use crate::models::types::PTrackerConfig;

/// Returns ~/.ptracker
pub fn ptracker_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".ptracker"))
}

/// Returns ~/.ptracker/projects.json
pub fn config_path() -> Result<PathBuf> {
    Ok(ptracker_dir()?.join("projects.json"))
}

/// Returns ~/.ptracker/projects/{slug}
pub fn project_dir(slug: &str) -> Result<PathBuf> {
    Ok(ptracker_dir()?.join("projects").join(slug))
}

/// Returns ~/.ptracker/projects/{slug}/project.json
pub fn project_path(slug: &str) -> Result<PathBuf> {
    Ok(project_dir(slug)?.join("project.json"))
}

/// Returns ~/.ptracker/projects/{slug}/sessions
pub fn sessions_dir(slug: &str) -> Result<PathBuf> {
    Ok(project_dir(slug)?.join("sessions"))
}

/// Returns ~/.ptracker/projects/{slug}/sessions/{date}.json
pub fn session_path(slug: &str, date: &str) -> Result<PathBuf> {
    Ok(sessions_dir(slug)?.join(format!("{}.json", date)))
}

/// Returns ~/.ptracker/projects/{slug}/logs
pub fn logs_dir(slug: &str) -> Result<PathBuf> {
    Ok(project_dir(slug)?.join("logs"))
}

/// Returns ~/.ptracker/projects/{slug}/logs/{date}.log
pub fn log_path(slug: &str, date: &str) -> Result<PathBuf> {
    Ok(logs_dir(slug)?.join(format!("{}.log", date)))
}

/// Creates all required directories for a new project
pub fn create_project_dirs(slug: &str) -> Result<()> {
    std::fs::create_dir_all(sessions_dir(slug)?)?;
    std::fs::create_dir_all(logs_dir(slug)?)?;
    Ok(())
}

/// Creates ~/.ptracker, projects/ and projects.json if they don't exist
pub fn ensure_ptracker_dir() -> Result<()> {
    std::fs::create_dir_all(ptracker_dir()?.join("projects"))?;

    let config = config_path()?;
    if !config.exists() {
        let file = File::create(&config)?;
        let writer = BufWriter::new(file); 
        let empty = PTrackerConfig { projects: vec![] };
        serde_json::to_writer_pretty(writer, &empty)?;
    }

    Ok(())
}