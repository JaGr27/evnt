//! Functions and structs for managing calendar events

use std::fs;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::App;

/// An event that can be added to the calendar
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Name of the event
    pub name: String,
    /// Optional description for the event
    pub description: Option<String>,

    /// The time at which the event occurs (stored in UTC, timezone offset is added when needed)
    pub date_time: DateTime<Utc>,

    /// Unique id for the event. This is necessary because different events can have the same name. Also acts as the filename for the serialized event
    id: u128,
}

impl Event {
    pub fn new(
        name: &str,
        description: Option<&str>,
        date_time: DateTime<Utc>,
        app: &App,
    ) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            description: description.map(String::from),
            date_time,

            id: generate_id(app)
                .with_context(|| format!("failed to generate event id for `{}`", name))?,
        })
    }

    /// Serializes and writes the event to the filesystem (using [bincode]). The event gets written to [App::events_dir].
    /// The filename is equal to the unique id of the event
    pub fn store(&self, app: &App) -> Result<()> {
        let bytes = bincode::serialize(self).with_context(|| {
            format!(
                "failed to serialize event `{}` (id: {})",
                self.name, self.id
            )
        })?;

        let path = app.events_dir.join(self.id.to_string());

        fs::write(&path, bytes).with_context(|| {
            format!(
                "failed to write event `{}` (id: {}) to `{}`",
                self.name,
                self.id,
                path.to_string_lossy()
            )
        })?;

        Ok(())
    }

    /// Deletes the file associated with the event
    pub fn delete_file(&self, app: &App) -> Result<()> {
        fs::remove_file(app.events_dir.join(self.id.to_string()))
            .with_context(|| format!("failed to delete task `{}` (id: {})", self.name, self.id))?;

        Ok(())
    }
}

/// Reads all the events from [App::events_dir]
pub fn read_events(app: &App) -> Result<Vec<Event>> {
    let mut events = Vec::new();

    for entry in fs::read_dir(&app.events_dir).with_context(|| {
        format!(
            "failed to read directory `{}`",
            app.events_dir.to_string_lossy()
        )
    })? {
        let entry = entry.with_context(|| {
            format!(
                "failed to get directory entry from `{}`",
                app.events_dir.to_string_lossy()
            )
        })?;

        if entry
            .file_type()
            .with_context(|| {
                format!(
                    "failed to get file type from file `{}`",
                    entry.file_name().to_string_lossy()
                )
            })?
            .is_file()
        {
            let bytes = fs::read(entry.path()).with_context(|| {
                format!(
                    "failed to read from file `{}`",
                    entry.file_name().to_string_lossy()
                )
            })?;

            events.push(bincode::deserialize(&bytes).with_context(|| {
                format!(
                    "failed to deserialize event from file `{}`",
                    entry.file_name().to_string_lossy()
                )
            })?);
        }
    }

    Ok(events)
}

/// Generates a unique id for an event
fn generate_id(app: &App) -> Result<u128> {
    let ids = get_ids(app).with_context(|| "failed to get event id's".to_string())?;

    loop {
        let id = rand::random();

        if !ids.contains(&id) {
            return Ok(id);
        }
    }
}

/// Gets all event ids by reading filenames from [App::events_dir]
fn get_ids(app: &App) -> Result<Vec<u128>> {
    let mut ids = Vec::new();

    for entry in fs::read_dir(&app.events_dir).with_context(|| {
        format!(
            "failed to read directory `{}`",
            app.events_dir.to_string_lossy()
        )
    })? {
        let entry = entry.with_context(|| {
            format!(
                "failed to get directory entry from `{}`",
                app.events_dir.to_string_lossy()
            )
        })?;

        let name = entry.file_name();
        let name = name.to_string_lossy();

        if let Ok(id) = name.parse::<u128>() {
            ids.push(id);
        }
    }

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_unique_ids() {
        let temp_data_dir = assert_fs::TempDir::new().unwrap();
        let app = App::new(temp_data_dir.path());
        crate::utils::create_dirs(&app).unwrap();

        let mut ids = Vec::new();

        for _ in 0..1000 {
            let id = generate_id(&app).unwrap();
            assert!(!ids.contains(&id));

            ids.push(id);

            fs::File::create(app.data_dir.join(id.to_string())).unwrap();
        }
    }

    #[test]
    fn event_serializes_and_deserializes() {
        use chrono::TimeZone;

        let temp_data_dir = assert_fs::TempDir::new().unwrap();
        let app = App::new(temp_data_dir.path());
        crate::utils::create_dirs(&app).unwrap();

        let event = Event::new(
            "Test Event",
            Some("Event description"),
            Utc.ymd(1000, 10, 10).and_hms(14, 30, 0),
            &app,
        )
        .unwrap();
        event.store(&app).unwrap();

        let events = read_events(&app).unwrap();
        let read_event = events.get(0).unwrap();

        assert!(event == *read_event);
    }

    #[test]
    fn serializes_and_deserializes_lots_of_events() {
        use chrono::TimeZone;

        let temp_data_dir = assert_fs::TempDir::new().unwrap();
        let app = App::new(temp_data_dir.path());
        crate::utils::create_dirs(&app).unwrap();

        let mut original_events = Vec::new();

        for n in 0..100 {
            let event = Event::new(
                &n.to_string(),
                None,
                Utc.ymd(2000, 10, 10).and_hms(15, 15, 0),
                &app,
            )
            .unwrap();

            event.store(&app).unwrap();

            original_events.push(event);
        }

        let read_events = read_events(&app).unwrap();

        for event in original_events {
            assert!(read_events.contains(&event));
        }
    }

    #[test]
    fn deletes_events() {
        use chrono::TimeZone;

        let temp_data_dir = assert_fs::TempDir::new().unwrap();
        let app = App::new(temp_data_dir.path());
        crate::utils::create_dirs(&app).unwrap();

        let event = Event::new(
            "Event Name",
            Some("Event description"),
            Utc.ymd(2000, 2, 4).and_hms(20, 10, 0),
            &app,
        )
        .unwrap();

        event.store(&app).unwrap();
        assert!(app.events_dir.join(event.id.to_string()).exists());

        event.delete_file(&app).unwrap();
        assert!(!app.events_dir.join(event.id.to_string()).exists());
    }
}
