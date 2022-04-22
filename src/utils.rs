//! Utility functions for common tasks

use std::{fs, io};

use anyhow::{Context, Result};

use crate::App;

/// Creates all directories necessary for the program to run (if they don't exist)
pub fn create_dirs(app: &App) -> Result<()> {
    // We don't need to create App::data_dir because fs::create_dir_all() will create it for us, as App::events_dir is a subdirectory of App::data_dir
    if let Err(e) = fs::create_dir_all(&app.events_dir) {
        // Don't return Err if the directory already exists (that is expected)
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Err(e).with_context(|| {
                format!(
                    "failed to create directory `{}`",
                    app.events_dir.to_string_lossy()
                )
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_required_dirs() {
        let temp_data_dir = assert_fs::TempDir::new().unwrap();
        let app = App::new(temp_data_dir.path());

        crate::utils::create_dirs(&app).unwrap();

        assert!(app.data_dir.exists());
        assert!(app.events_dir.exists());
    }
}
