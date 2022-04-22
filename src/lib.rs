pub mod event;
pub mod utils;

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Stores the configuration of the program
pub struct App {
    /// The directory in which the programs data is stored
    pub data_dir: PathBuf,

    /// The directory in which the events are stored, should be the "events" subdirectory of [App::data_dir]
    pub events_dir: PathBuf,
}

impl App {
    pub fn new<P: AsRef<Path> + ToOwned>(data_dir: P) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            events_dir: data_dir.as_ref().to_path_buf().join("events/"),
        }
    }
}

/// Run the program
pub fn run(app: App) -> Result<()> {
    utils::create_dirs(&app)?;

    Ok(())
}
