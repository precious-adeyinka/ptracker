use notify::EventKind;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileEvent {
    Created(String),
    Modified(String),
    Renamed(String, String),
    Deleted(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PTrackerConfig {
    pub projects: Vec<ProjectEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectEntry {
    pub name: String,
    pub slug: String,
    pub watch_path: String,
    pub created_at: String,
    pub total_hours: f64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub name: String,
    pub slug: String,
    pub watch_path: String,
    pub created_at: String,
    pub total_hours: f64,
    pub total_sessions: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub id: String,
    pub date: String,
    pub start: String,
    pub end: Option<String>,
    pub duration_mins: Option<f64>,
    pub watch_path: String,
    pub events: SessionEvents,
    pub totals: SessionTotals,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SessionEvents {
    pub created: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub renamed: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SessionTotals {
    pub created: u32,
    pub modified: u32,
    pub deleted: u32,
    pub renamed: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DayLog {
    pub date: String,
    pub sessions: Vec<Session>,
    pub total_hours: f64,
}

pub fn map_file_event(event: &notify::Event) -> Option<FileEvent> {
    let path = event.paths.first()?.to_string_lossy().to_string();

    match &event.kind {
        EventKind::Create(_) => Some(FileEvent::Created(path)),
        EventKind::Modify(_) => Some(FileEvent::Modified(path)  ),
        EventKind::Remove(_) => Some(FileEvent::Deleted(path)),
        _ => None,
    }
}

#[allow(dead_code)]
pub trait StorageBackend {
    fn save(&mut self) -> anyhow::Result<()>;
    fn load(name: &str) -> anyhow::Result<Self> where Self: Sized;
    fn update(&mut self) -> anyhow::Result<()>;
    fn delete(&mut self, name: &str) -> anyhow::Result<()>;
}
