use notify::EventKind;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileEvent {
    Created(String),
    Modified(String),
    Renamed(String, String),
    Deleted(String),
    Initialized(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub name: String,
    pub hours_worked: f64,
    pub events: Vec<FileEvent>,
    pub logs: Vec<String>,
    pub log_path: String,
}

#[allow(dead_code)]
pub trait StorageBackend {
    fn save(&mut self) -> anyhow::Result<()>;
    fn load(name: &str) -> anyhow::Result<Self> where Self: Sized;
    fn update(&mut self) -> anyhow::Result<()>;
    fn delete(&mut self, name: &str) -> anyhow::Result<()>;
}

impl Project {
    pub fn new(name: &str) -> Project {
        Project {
            name: String::from(name),
            hours_worked: 0.0,
            events: vec![FileEvent::Initialized(format!("Project '{}' initialization", name.to_lowercase()))],
            logs: vec![String::from("Project initialized")],
            log_path: format!("{}", name.to_lowercase()),
        }
    }

    pub fn describe_event(&self, event: &FileEvent) -> String {
        match &event {
            FileEvent::Created(filename) => format!("Created {}", filename),
            FileEvent::Modified(filename) => format!("Modified {}", filename),
            FileEvent::Renamed(old_filename, new_filename) => format!("Renamed {} -> {}", old_filename, new_filename),
            FileEvent::Deleted(filename) => format!("Deleted {}", filename),
            FileEvent::Initialized(filename) => format!("Project '{}' initialized", filename),
        }
    }

    pub fn map_events(event: &notify::Event) -> Option<FileEvent> {
        let path = event.paths.first()?.to_string_lossy().to_string();

        match &event.kind {
            EventKind::Create(_) => Some(FileEvent::Created(path)),
            EventKind::Modify(_) => Some(FileEvent::Modified(path)  ),
            EventKind::Remove(_) => Some(FileEvent::Deleted(path)),
            _ => None, // ignore other events
        }
    }
}