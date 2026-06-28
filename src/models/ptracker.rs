use crate::models::types::{
    Project, ProjectEntry, PTrackerConfig, 
    Session, SessionEvents, SessionTotals, 
    DayLog, FileEvent, StorageBackend, map_file_event
};
use crate::models::storage::{
    ensure_ptracker_dir, create_project_dirs, 
    config_path, project_path, project_dir, 
    session_path, log_path
};
use chrono::Local;
use std::fs::{File, remove_dir_all};
use std::io::{BufReader, BufWriter, Write};
use std::fs::OpenOptions;
use comfy_table::Table;
use std::sync::mpsc;
use std::path::Path;
use notify::{RecursiveMode, Watcher, recommended_watcher, Event};
use std::collections::HashMap;
use std::time::Instant;
use anyhow::{Context, Result};
use uuid::Uuid;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ─── Config helpers ───────────────────────────────────────────────────────────

fn load_config() -> Result<PTrackerConfig> {
    let path = config_path()?;
    let file = File::open(&path).context("Could not open projects.json\n")?;
    let reader = BufReader::new(file);
    let config: PTrackerConfig = serde_json::from_reader(reader)
        .context("Could not parse projects.json\n")?;
    Ok(config)
}

fn save_config(config: &PTrackerConfig) -> Result<()> {
    let path = config_path()?;
    let file = File::create(&path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, config)?;
    Ok(())
}

fn to_slug(name: &str) -> String {
    name.trim().to_lowercase().replace(" ", "-")
}

// ─── Project impl ─────────────────────────────────────────────────────────────

impl Project {
     pub fn new(name: &str, watch_path: &str) -> Project {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Project {
            name: String::from(name),
            slug: to_slug(name),
            watch_path: String::from(watch_path),
            created_at: now,
            total_hours: 0.0,
            total_sessions: 0
        }
    }

    // ── Commands ──────────────────────────────────────────────────────────────

    pub fn init(name: &str, watch_path: Option<&str>) -> Result<()> {
        ensure_ptracker_dir()?;

        let slug = to_slug(name);
        let mut config = load_config()?;

        // check duplicates
        if config.projects.iter().any(|p| p.slug == slug) {
            anyhow::bail!("Project '{}' already exists.", name);
        }

        let path = watch_path.unwrap_or(".");
        let project = Project::new(name, path);

        // create ~/.ptracker/projects/{slug}/sessions/ and logs/
        create_project_dirs(&slug)?;

        // save project.json
        let proj_file = File::create(project_path(&slug)?)?;
        serde_json::to_writer_pretty(BufWriter::new(proj_file), &project)?;

        // register in projects.json
        config.projects.push(ProjectEntry {
            name: project.name.clone(),
            slug: project.slug.clone(),
            watch_path: project.watch_path.clone(),
            created_at: project.created_at.clone(),
            total_hours: 0.0
        });
        save_config(&config)?;

        println!("Project '{}' initialized.", name);
        println!("Watching: {}", path);
        println!("Location: ~/.ptracker/projects/{}/", slug);

        Ok(())
    }

    pub fn list() -> Result<()> {
        ensure_ptracker_dir()?;
        let config = load_config()?;

        if config.projects.is_empty() {
            println!("No projects yet. Run `ptracker init <name>` to create one.");
            return Ok(());
        }

        let mut table = Table::new();
        table.set_header(vec!["Name", "Slug", "Watch Path", "Total Hours", "Created At"]);
        for p in &config.projects {
            table.add_row(vec![
                &p.name,
                &p.slug,
                &p.watch_path,
                &format!("{:.1}h", p.total_hours),
                &p.created_at,
            ]);
        }
        println!("{table}");
        Ok(())
    }

    pub fn summary(name: &str) -> Result<()> {
        ensure_ptracker_dir()?;
        let slug = to_slug(name);
        let project = Self::load_project(&slug)?;

        println!("\n── {} ──", project.name.to_uppercase());
        let mut table = Table::new();
        table.set_header(vec!["Field", "Value"]);
        table.add_row(vec!["Name", &project.name]);
        table.add_row(vec!["Slug", &project.slug]);
        table.add_row(vec!["Watch Path", &project.watch_path]);
        table.add_row(vec!["Total Hours", &format!("{:.1}h", project.total_hours)]);
        table.add_row(vec!["Total Sessions", &project.total_sessions.to_string()]);
        table.add_row(vec!["Created At", &project.created_at]);
        println!("{table}");

        // show today's session if exists
        let today = Local::now().format("%Y-%m-%d").to_string();
        let session_file = session_path(&slug, &today)?;
        if session_file.exists() {
            let file = File::open(&session_file)?;
            let day: DayLog = serde_json::from_reader(BufReader::new(file))?;
            println!("\n── Today's Sessions ──");
            let mut s_table = Table::new();
            s_table.set_header(vec!["#", "Start", "End", "Duration", "Created", "Modified", "Deleted", "Renamed"]);
            for (i, s) in day.sessions.iter().enumerate() {
                s_table.add_row(vec![
                    &(i + 1).to_string(),
                    &s.start,
                    s.end.as_deref().unwrap_or("active"),
                    &format!("{:.1}m", s.duration_mins.unwrap_or(0.0)),
                    &s.totals.created.to_string(),
                    &s.totals.modified.to_string(),
                    &s.totals.deleted.to_string(),
                    &s.totals.renamed.to_string(),
                ]);
            }
            println!("{s_table}");
        }

        Ok(())
    }

    pub fn rename(old_name: &str, new_name: &str) -> Result<()> {
        ensure_ptracker_dir()?;
        let old_slug = to_slug(old_name);
        let new_slug = to_slug(new_name);

        let mut config = load_config()?;
        let entry = config.projects.iter_mut()
            .find(|p| p.slug == old_slug)
            .context(format!("Project '{}' not found.\n", old_name))?;

        // rename the folder
        let old_dir = project_dir(&old_slug)?;
        let new_dir = project_dir(&new_slug)?;
        std::fs::rename(&old_dir, &new_dir)
            .context("Failed to rename project folder\n")?;

        // update project.json inside
        let mut project = Self::load_project(&new_slug)?;
        project.name = String::from(new_name);
        project.slug = new_slug.clone();
        Self::save_project(&project)?;

        // update registry
        entry.name = String::from(new_name);
        entry.slug = new_slug.clone();
        save_config(&config)?;

        println!("Renamed '{}' to '{}'.", old_name, new_name);
        Ok(())
    }

    pub fn delete(name: &str) -> Result<()> {
        ensure_ptracker_dir()?;
        let slug = to_slug(name);

        let mut config = load_config()?;
        let pos = config.projects.iter()
            .position(|p| p.slug == slug)
            .context(format!("Project '{}' not found.\n", name))?;

        // delete the whole project folder
        remove_dir_all(project_dir(&slug)?)
            .context("Failed to delete project folder\n")?;

        config.projects.remove(pos);
        save_config(&config)?;

        println!("Project '{}' deleted.\n", name);
        Ok(())
    }

    pub fn log_hours(name: &str, hours: f64) -> Result<()> {
        ensure_ptracker_dir()?;
        let slug = to_slug(name);
        let mut project = Self::load_project(&slug)?;

        project.total_hours += hours;
        Self::save_project(&project)?;
        Self::update_config_hours(&slug, project.total_hours)?;

        println!("Logged {:.1}h to '{}'. Total: {:.1}h\n", hours, name, project.total_hours);
        Ok(())
    }

    pub fn watch(name: &str, path: Option<&str>) -> Result<()> {
        ensure_ptracker_dir()?;
        let slug = to_slug(name);
        let project = Self::load_project(&slug)?;

        let watch_path = path.unwrap_or(&project.watch_path).to_string();
        let today = Local::now().format("%Y-%m-%d").to_string();
        let session_id = Uuid::new_v4().to_string();
        let start_time = Local::now().format("%H:%M:%S").to_string();
        let start_instant = Instant::now();

        println!("── ptracker ──────────────────────────────");
        println!("  Project : {}", project.name);
        println!("  Watching: {}", watch_path);
        println!("  Session : {}", session_id);
        println!("  Started : {}", start_time);
        println!("  Press Ctrl+C to stop.");
        println!("──────────────────────────────────────────\n");

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = recommended_watcher(tx).context("Failed to create watcher\n")?;
        watcher.watch(Path::new(&watch_path), RecursiveMode::Recursive)
            .context("Failed to watch folder\n")?;

        let mut last_seen: HashMap<String, Instant> = HashMap::new();
        let mut session_events = SessionEvents::default();
        let mut session_totals = SessionTotals::default();

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        }).context("Failed to set Ctrl+C handler")?;

        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(Ok(event)) => {
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }

                    if let Some(file_event) = map_file_event(&event) {
                        let path_key = match &file_event {
                            FileEvent::Created(p)    => p.clone(),
                            FileEvent::Modified(p)   => p.clone(),
                            FileEvent::Deleted(p)    => p.clone(),
                            FileEvent::Renamed(p, _) => p.clone(),
                        };

                        let now = Instant::now();
                        if let Some(last) = last_seen.get(&path_key) {
                            if now.duration_since(*last).as_millis() < 500 {
                                continue;
                            }
                        }
                        last_seen.insert(path_key, now);

                        Self::append_log(&slug, &today, &file_event)?;

                        match &file_event {
                            FileEvent::Created(p) => {
                                println!("  + Created  {}", p);
                                session_events.created.push(p.clone());
                                session_totals.created += 1;
                            }
                            FileEvent::Modified(p) => {
                                println!("  ~ Modified {}", p);
                                session_events.modified.push(p.clone());
                                session_totals.modified += 1;
                            }
                            FileEvent::Deleted(p) => {
                                println!("  - Deleted  {}", p);
                                session_events.deleted.push(p.clone());
                                session_totals.deleted += 1;
                            }
                            FileEvent::Renamed(old, new) => {
                                println!("  > Renamed  {} → {}", old, new);
                                session_events.renamed.push((old.clone(), new.clone()));
                                session_totals.renamed += 1;
                            }
                        }
                    }
                }
                Ok(Err(e)) => println!("Watch error: {:?}", e),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // no event in 250ms — just loop and check flag again
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // watcher dropped, exit cleanly
                    break;
                }
            }
        }

        // session ended — calculate duration
        let duration_mins = start_instant.elapsed().as_secs_f64() / 60.0;
        let end_time = Local::now().format("%H:%M:%S").to_string();
        let hours = duration_mins / 60.0;

        let session = Session {
            id: session_id,
            date: today.clone(),
            start: start_time,
            end: Some(end_time.clone()),
            duration_mins: Some(duration_mins),
            watch_path: watch_path.clone(),
            events: session_events,
            totals: session_totals.clone(),
        };

        // save session to day file
        Self::save_session(&slug, &today, session)?;

        // update project totals
        let mut project = Self::load_project(&slug)?;
        project.total_hours += hours;
        project.total_sessions += 1;
        Self::save_project(&project)?;
        Self::update_config_hours(&slug, project.total_hours)?;

        println!("\n── Session ended ─────────────────────────");
        println!("  Ended   : {}", end_time);
        println!("  Duration: {:.1} mins ({:.2}h)", duration_mins, hours);
        println!("  Created : {}", session_totals.created);
        println!("  Modified: {}", session_totals.modified);
        println!("  Deleted : {}", session_totals.deleted);
        println!("  Renamed : {}", session_totals.renamed);
        println!("──────────────────────────────────────────");

        Ok(())
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn load_project(slug: &str) -> Result<Project> {
        let path = project_path(slug)?;
        let file = File::open(&path)
            .context(format!("Project '{}' not found in ~/.ptracker\n", slug))?;
        let project: Project = serde_json::from_reader(BufReader::new(file))?;
        Ok(project)
    }

    fn save_project(project: &Project) -> Result<()> {
        let path = project_path(&project.slug)?;
        let file = File::create(&path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), project)?;
        Ok(())
    }

    fn update_config_hours(slug: &str, total_hours: f64) -> Result<()> {
        let mut config = load_config()?;
        if let Some(entry) = config.projects.iter_mut().find(|p| p.slug == *slug) {
            entry.total_hours = total_hours;
        }
        save_config(&config)?;
        Ok(())
    }

    fn save_session(slug: &str, date: &str, session: Session) -> Result<()> {
        let path = session_path(slug, date)?;

        let mut day: DayLog = if path.exists() {
            let file = File::open(&path)?;
            serde_json::from_reader(BufReader::new(file))?
        } else {
            DayLog {
                date: date.to_string(),
                sessions: vec![],
                total_hours: 0.0,
            }
        };

        day.total_hours += session.duration_mins.unwrap_or(0.0) / 60.0;
        day.sessions.push(session);

        let file = File::create(&path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), &day)?;
        Ok(())
    }

    fn append_log(slug: &str, date: &str, event: &FileEvent) -> Result<()> {
        let path = log_path(slug, date)?;
        let now = Local::now().format("%H:%M:%S").to_string();

        let msg = match event {
            FileEvent::Created(f)       => format!("[{}] + Created  {}", now, f),
            FileEvent::Modified(f)      => format!("[{}] ~ Modified {}", now, f),
            FileEvent::Deleted(f)       => format!("[{}] - Deleted  {}", now, f),
            FileEvent::Renamed(old, new) => format!("[{}] > Renamed  {} → {}", now, old, new),
        };

        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        writeln!(file, "{}", msg)?;
        Ok(())
    }
}

impl StorageBackend for Project {
    fn save(&mut self) -> Result<()> {
        Self::save_project(self)
    }

    fn load(name: &str) -> Result<Self> where Self: Sized {
        Self::load_project(&to_slug(name))
    }

    fn update(&mut self) -> Result<()> {
        Self::save_project(self)
    }

    fn delete(&mut self, name: &str) -> Result<()> {
        Project::delete(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // force sequential test execution
    use std::sync::Mutex;
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn cleanup(slug: &str) {
        let dir = crate::models::storage::project_dir(slug).unwrap();
        if dir.exists() {
            std::fs::remove_dir_all(dir).ok();
        }
        // reset projects.json to empty array cleanly
        if let Ok(path) = crate::models::storage::config_path() {
            let file = std::fs::File::create(&path).unwrap();
            let writer = std::io::BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &PTrackerConfig { projects: vec![] }).unwrap();
        }
    }

    #[test]
    fn test_to_slug() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        assert_eq!(to_slug("My Project"), "my-project");
        assert_eq!(to_slug("VNACSA"), "vnacsa");
        assert_eq!(to_slug("Client API"), "client-api");
    }

    #[test]
    fn test_project_new() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let project = Project::new("TestProject", "./src");
        assert_eq!(project.name, "TestProject");
        assert_eq!(project.slug, "testproject");
        assert_eq!(project.watch_path, "./src");
        assert_eq!(project.total_hours, 0.0);
        assert_eq!(project.total_sessions, 0);
    }

    #[test]
    fn test_init_project() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("TestInit", Some("./src")).unwrap();

        let config = load_config().unwrap();
        assert!(config.projects.iter().any(|p| p.slug == "testinit"));

        cleanup("testinit");
    }

    #[test]
    fn test_init_duplicate_fails() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("DupTest", Some("./src")).unwrap();
        let result = Project::init("DupTest", Some("./src"));
        assert!(result.is_err());

        cleanup("duptest");
    }

    #[test]
    fn test_load_project() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("LoadTest", Some("./src")).unwrap();

        let project = Project::load_project("loadtest").unwrap();
        assert_eq!(project.name, "LoadTest");
        assert_eq!(project.slug, "loadtest");
        assert_eq!(project.total_hours, 0.0);

        cleanup("loadtest");
    }

    #[test]
    fn test_load_nonexistent_fails() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        let result = Project::load_project("nonexistent-xyz-abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_log_hours() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("HoursTest", Some("./src")).unwrap();

        Project::log_hours("HoursTest", 2.5).unwrap();
        Project::log_hours("HoursTest", 1.5).unwrap();

        let project = Project::load_project("hourstest").unwrap();
        assert_eq!(project.total_hours, 4.0);

        cleanup("hourstest");
    }

    #[test]
    fn test_rename_project() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("OldNameTest", Some("./src")).unwrap();
        Project::rename("OldNameTest", "NewNameTest").unwrap();

        // old slug gone
        let old = Project::load_project("oldnametest");
        assert!(old.is_err());

        // new slug exists
        let new = Project::load_project("newnametest").unwrap();
        assert_eq!(new.name, "NewNameTest");
        assert_eq!(new.slug, "newnametest");

        cleanup("newnametest");
    }

    #[test]
    fn test_delete_project() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("DeleteTest", Some("./src")).unwrap();
        Project::delete("DeleteTest").unwrap();

        let config = load_config().unwrap();
        assert!(!config.projects.iter().any(|p| p.slug == "deletetest"));

        let dir = crate::models::storage::project_dir("deletetest").unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn test_append_log() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("LogTest", Some("./src")).unwrap();

        let today = Local::now().format("%Y-%m-%d").to_string();
        let event = FileEvent::Created(String::from("main.rs"));
        Project::append_log("logtest", &today, &event).unwrap();

        let log_file = crate::models::storage::log_path("logtest", &today).unwrap();
        assert!(log_file.exists());

        cleanup("logtest");
    }

    #[test]
    fn test_config_updates_after_log_hours() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        ensure_ptracker_dir().unwrap();
        Project::init("ConfigTest", Some("./src")).unwrap();
        Project::log_hours("ConfigTest", 3.0).unwrap();

        let config = load_config().unwrap();
        let entry = config.projects.iter().find(|p| p.slug == "configtest").unwrap();
        assert_eq!(entry.total_hours, 3.0);

        cleanup("configtest");
    }
}