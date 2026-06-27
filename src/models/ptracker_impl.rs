use crate::models::types::{Project, FileEvent, StorageBackend};
use chrono::Local;
use std::fs::{rename, remove_file};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Error};
use std::fs::OpenOptions;
use comfy_table::Table;
use std::sync::mpsc;
use std::path::Path;
use notify::{RecursiveMode, Watcher, recommended_watcher, Event};
use std::collections::HashMap;
use std::time::Instant;
use anyhow::{Context};

impl Project {
    pub fn start_watching(project_name: &str, path: &str) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = recommended_watcher(tx).context("Failed to create watcher")?;
        
        watcher.watch(Path::new(path), RecursiveMode::Recursive).context("Failed to watch folder")?;

        println!("Tracking '{}' — watching '{}'", project_name, path);
        println!("Press Ctrl+C to stop.\n");

        let mut last_seen: HashMap<String, Instant> = HashMap::new();

        for result in rx {
            let event = result.context(format!("Watch Error"))?;

            if let Some(file_event) = Self::map_events(&event) {
                let path_key = match &file_event {
                    FileEvent::Created(p) => p.clone(),
                    FileEvent::Modified(p) => p.clone(),
                    FileEvent::Deleted(p) => p.clone(),
                    _ => continue
                };

                let now = Instant::now();
                if let Some(last) = last_seen.get(&path_key) {
                    if now.duration_since(*last).as_millis() < 500 {
                        continue
                    }
                }
                last_seen.insert(path_key, now);

                let mut project = Project::load(project_name).context("Error loading project")?;
                let msg = project.describe_event(&file_event);
                println!("{}", msg);

                project.events.push(file_event);
                project.logs.push(msg.clone());
                
                let _ = project.log_activity(&msg);
                let _ = project.update();
            }

        }

        Ok(())

    }

    pub fn summarize(project_name: &str) -> anyhow::Result<()> {
        let project = Self::load(project_name).context(format!("\nProject '{}' not found.", &project_name))?;

        println!("\nPrinting details for project: {}", &project.name.to_uppercase());
        let mut table = Table::new();
        table.set_header(vec!["Project Name", "Hours Worked", "Log Path"]);
        table.add_row(vec![&project.name, &project.hours_worked.to_string(), &project.log_path]);
        println!("{table}");

        println!("\nPrinting logs for project: {}", &project.name.to_uppercase());
        let mut log_table = Table::new();
        log_table.set_header(vec!["S/N", "Logs"]);
        let mut count = 0;
        for log in &project.logs {
            count += 1;
            log_table.add_row(vec![&count.to_string(), &log]);
        }
        println!("{log_table}");

        Ok(())
    }

    pub fn log_hours(project_name: &str, hours: f64) ->anyhow::Result<()> {
        let mut project = Project::load(project_name).context(format!("\nProject '{}' not found.", project_name))?;
        project.add_hours(project_name, hours).context(format!("Error logging to: {}.", project_name))?;
        println!("Logged {:.1} hrs to '{}'", hours, project_name);
        Ok(())
    }

    pub fn rename_project(old_project_name: &str, new_project_name: &str) -> anyhow::Result<()> {
        let mut project = Project::load(old_project_name).context(format!("\nProject '{}' not found.", old_project_name))?;
        let names = project.rename_file(new_project_name).context(format!("Error renaming '{}' to: '{}'.", old_project_name, new_project_name))?;
        println!("Project renamed from '{}' to '{}'", names[0], names[1]);

        Ok(())
    }

    pub fn delete_project(project_name: &str) -> anyhow::Result<()> {
        let mut project = Project::load(project_name).context(format!("\nProject '{}' not found.", project_name))?;
        project.delete_file(project_name).context(format!("Error deleting project: {}.", project_name))?;
        println!("Project '{}' deleted successfully", project_name);
        Ok(())
    }

    fn  update(&mut self) -> anyhow::Result<()> {
        let file = File::create(self.filename())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    fn  load(project_name: &str) -> anyhow::Result<Project> {
        let path = Self::to_filename(&project_name);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: Project = serde_json::from_reader(reader).map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    fn filename(&self) -> String {
        format!("{}.json", self.name.to_lowercase())
    }

    fn to_logname(name: &str) -> String {
        format!("{}.log", name.to_lowercase())
    }

    fn to_filename(name: &str) -> String {
        format!("{}.json", name.to_lowercase())
    }

    fn add_hours(&mut self, project_name: &str, hours: f64) -> anyhow::Result<()> {
        let project = Self::load(&project_name.to_lowercase())?;
        println!("{:#?}", project);

        self.hours_worked += hours;
        self.update()?;

        Ok(())
    }

    fn rename_file(&mut self, new_project_name: &str) -> anyhow::Result<Vec<String>> {
        let old_name = &self.name.clone();
        let old_path = self.log_path.clone();

        self.name = new_project_name.to_string();
        self.log_path = new_project_name.to_lowercase();
        
        rename(Self::to_filename(old_name), Self::to_filename(new_project_name))?;
        rename(Self::to_logname(&old_path), Self::to_logname(new_project_name))?;

        self.update()?;

        Ok(vec![Self::to_filename(&old_name.to_string()), Self::to_filename(&new_project_name.to_string())])
    }

    fn delete_file(&mut self, project_name: &str) -> anyhow::Result<()> {
        remove_file(Self::to_filename(project_name))?;
        remove_file(format!("{}.log", self.log_path))?;

        Ok(())
    }

    fn log_activity(&self, msg: &str) -> anyhow::Result<()> {
        let filename = format!("{}.log", &self.log_path);
        let now = Local::now();

        let mut file = OpenOptions::new().create(true).append(true).open(filename)?;
        let log_msg = format!("[{}] | {} | {}", now.format("%H:%M:%S"), self.name.to_uppercase(), msg);

        writeln!(file, "{}", log_msg).context(format!("Error saving logs"))?;

        Ok(())
    }
}


impl StorageBackend for Project {
    fn save(&mut self) -> anyhow::Result<()> {
        let file = File::create(self.filename())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;

        let event = FileEvent::Created(Self::to_filename(&self.name));
        let msg = Self::describe_event(&self, &event);
        let _ = Self::log_activity(&self, &msg)?;

        Ok(())
    }

    fn load(name: &str) -> anyhow::Result<Self> where Self: Sized {
        let path = Self::to_filename(&name);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: Project = serde_json::from_reader(reader)?;
        Ok(data)
    }

    fn  update(&mut self) -> anyhow::Result<()> {
        let file = File::create(self.filename())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    fn delete(&mut self, name: &str) -> anyhow::Result<()> {
        let mut project = Project::load(name).context(format!("\nProject '{}' not found.", name))?;
        project.delete_file(name).context(format!("Error deleting project: {}.", name))?;
        println!("Project '{}' deleted successfully", name);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project() {
        let project = Project::new("TestProject");
        let msg = project.describe_event(&project.events[0]);
        assert_eq!(project.name, "TestProject");
        assert_eq!(project.hours_worked, 0.0);
        assert_eq!(project.log_path, "testproject");
        assert_eq!(&msg, "Project 'Project 'testproject' initialization' initialized");
        assert_eq!(&project.logs[0], "Project initialized");
        assert!(project.logs.len() > 0);
        assert!(project.events.len() > 0);
    }

    #[test]
    fn test_load_project() {
        let mut project = Project::new("TestProject");
        project.save().unwrap();

        let saved_project = Project::load("testproject").unwrap();
        assert_eq!(saved_project.name, "TestProject");
        assert_eq!(saved_project.hours_worked, 0.0);
        assert_eq!(saved_project.log_path, "testproject");
        assert!(saved_project.logs.len() > 0);
        assert!(saved_project.events.len() > 0);

        std::fs::remove_file("testproject.json").unwrap();
        std::fs::remove_file("testproject.log").ok();
    }

    #[test]
    #[should_panic]
    fn test_project_not_found() {
        Project::load("xyzdfdf").unwrap();
    }

    #[test]
    fn test_describe_event_initialized() {
        let project = Project::new("TestProject");
        let event = FileEvent::Initialized(String::from("client"));
        let msg = project.describe_event(&event);
        assert_eq!(&msg, "Project 'client' initialized");
    }

    #[test]
    fn test_describe_event_created() {
        let project = Project::new("TestProject");
        let event = FileEvent::Created(String::from("main.rs"));
        let msg = project.describe_event(&event);
        assert_eq!(&msg, "Created main.rs");
    }

    #[test]
    fn test_describe_event_modified() {
        let project = Project::new("TestProject");
        let event = FileEvent::Modified(String::from("main.rs"));
        let msg = project.describe_event(&event);
        assert_eq!(&msg, "Modified main.rs");
    }

    #[test]
    fn test_describe_event_renamed() {
        let project = Project::new("TestProject");
        let event = FileEvent::Renamed(String::from("client.rs"), String::from("cli.rs"));
        let msg = project.describe_event(&event);
        assert_eq!(&msg, "Renamed client.rs -> cli.rs");
    }

    #[test]
    fn test_describe_event_deleted() {
        let project = Project::new("TestProject");
        let event = FileEvent::Deleted(String::from("main.rs"));
        let msg = project.describe_event(&event);
        assert_eq!(&msg, "Deleted main.rs");
    }

    #[test]
    fn test_hours_accumulate() {
        let mut project = Project::new("HoursTest");
        project.save().unwrap();
        
        Project::log_hours("HoursTest", 2.5).unwrap();
        Project::log_hours("HoursTest", 3.0).unwrap();
        
        let loaded = Project::load("HoursTest").unwrap();
        assert_eq!(loaded.hours_worked, 5.5);
        
        std::fs::remove_file("hourstest.json").unwrap();
        std::fs::remove_file("hourstest.log").ok();
    }

    #[test]
    fn test_rename_project() {
        let mut project = Project::new("OldName");
        project.save().unwrap();
        
        Project::rename_project("OldName", "NewName").unwrap();
        
        assert!(!std::path::Path::new("oldname.json").exists());
        assert!(std::path::Path::new("newname.json").exists());
        
        std::fs::remove_file("newname.json").unwrap();
        std::fs::remove_file("newname.log").ok();
    }

    #[test]
    fn test_log_path_matches_name() {
        let project = Project::new("MyProject");
        assert_eq!(project.log_path, "myproject");
    }

    #[test]
    fn test_working_dir() {
        let dir = std::env::current_dir().unwrap();
        println!("Running in: {:?}", dir);
    }
}