use core::str;
use std::{path::PathBuf, sync::Arc};

use chrono::Local;
use ignore::{overrides, WalkBuilder};
use lazy_static::lazy_static;
use regex::Regex;
use tokio::sync::{broadcast, RwLock};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    utils::{
        send_palantir_error_notification::send_palantir_error_notification,
        set_file_times::set_file_times,
    },
};

// Define a regex pattern to match specific Galadriel CSS-related markup in files.
const PATTERN: &str = r"@(?:class|layout|module):([a-zA-Z0-9_-]+)(?:::([a-zA-Z0-9_-]+))?";

lazy_static! {
    // Compile the regex pattern at runtime and store it in a static variable for reuse.
    pub static ref MARKUP_RE: Regex = Regex::new(PATTERN).unwrap();
}

// Structure to handle file timestamp updates, using a broadcast channel for notifications.
#[derive(Clone, Debug)]
pub struct FileTimestampUpdater {
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
}

impl FileTimestampUpdater {
    // Constructor to initialize the FileTimestampUpdater with a broadcast sender.
    pub fn new(palantir_sender: broadcast::Sender<GaladrielAlerts>) -> Self {
        Self { palantir_sender }
    }

    // Asynchronous method to process files in a folder and update their timestamps.
    pub async fn process_from_folder(
        &self,
        is_css_processing: bool,
        path: PathBuf, // Path to the folder to be processed.
        matcher: Arc<RwLock<overrides::Override>>, // Clone the sender for thread safety.
    ) {
        tracing::info!("Started processing files from folder: {}", path.display());

        let palantir_sender = self.palantir_sender.clone();
        // Clone the matcher reference to avoid locking it multiple times.
        // Acquire a read lock on the matcher.
        let cloned_matcher = Arc::clone(&matcher);
        let matcher = cloned_matcher.read().await;

        // Initialize a directory walker to recursively traverse the directory.
        let walker = WalkBuilder::new(&path)
            .hidden(true) // Exclude hidden files
            .ignore(false) // Do ignore excluded paths at this stage
            .parents(false) // Ignores .gitignore files in parent dir
            .git_global(false) // Do not consider global git ignore rules
            .git_ignore(false) // Do not consider project-specific git ignore rules
            .git_exclude(false) // Do not consider global git exclude rules
            .build();

        // Traverse all the directory entries found by the walker.
        for dir_entry in walker.into_iter().filter_map(|entry| entry.ok()) {
            // Get the path of the current directory entry.
            let current_path = dir_entry.path().to_path_buf();

            tracing::debug!("Checking file: {}", current_path.display());

            // Check if the file matches the component criteria.
            if self.is_component(is_css_processing, &current_path, &matcher)
                || self.is_css_processing(is_css_processing, &current_path, &matcher)
            {
                tracing::info!(
                    "File {} is a component or CSS file. Attempting to update timestamp.",
                    current_path.display()
                );

                // Try to update the file's timestamps.
                if let Err(error) = set_file_times(&current_path) {
                    tracing::error!(
                        "Failed to set timestamp for file {}. Error: {:?}",
                        current_path.display(),
                        error
                    );

                    let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

                    if let Err(err) = palantir_sender.send(notification) {
                        tracing::error!(
                            "Error sending notification about failed timestamp update for file {}. Error: {:?}",
                            current_path.display(),
                            err
                        );
                    }
                }
            }
        }

        tracing::info!("Finished processing files from folder: {}", path.display());
    }

    fn is_css_processing(
        &self,
        is_css_processing: bool,
        current_path: &PathBuf,
        matcher: &overrides::Override,
    ) -> bool {
        tracing::debug!("Checking if file {} is a CSS file.", current_path.display());

        is_css_processing
            && current_path
                .extension()
                .map(|ext| ext == "css")
                .unwrap_or(false)
            && self.is_valid(true, current_path, matcher)
    }

    // Helper function to determine if a file is a component based on path and content.
    fn is_component(
        &self,
        is_css_processing: bool,
        current_path: &PathBuf,
        matcher: &overrides::Override,
    ) -> bool {
        tracing::debug!(
            "Checking if file {} is a component.",
            current_path.display()
        );

        !is_css_processing
            && current_path
                .extension()
                .map(|ext| {
                    // Check if the file has a supported extension.
                    ext == "js" || ext == "jsx" || ext == "ts" || ext == "tsx" || ext == "html"
                })
                .unwrap_or(false)
            && self.is_valid(false, current_path, matcher)
    }

    fn is_valid(
        &self,
        is_css_processing: bool,
        current_path: &PathBuf,
        matcher: &overrides::Override,
    ) -> bool {
        !matcher.matched(current_path, false).is_ignore()
            && self.has_component_markup(is_css_processing, current_path)
        // Verify the file contains component markup.
    }

    // Helper function to check if a file contains specific markup.
    fn has_component_markup(&self, is_css_processing: bool, current_path: &PathBuf) -> bool {
        tracing::debug!(
            "Checking if file {} contains component markup.",
            current_path.display()
        );

        // Check for regex matches or specific Galadriel CSS markers in the CSS file content.
        match std::fs::read_to_string(current_path) {
            Ok(file_content) if is_css_processing => {
                return file_content.contains("@galadrielcss styles;");
            }
            Ok(file_content) => {
                return MARKUP_RE.is_match(&file_content);
            }
            Err(err) => {
                let error = GaladrielError::raise_general_other_error(
                    ErrorKind::Other,
                    &format!(
                        "Something went wrong while reading file at path: `{}`. Error: {}",
                        current_path.to_string_lossy().to_string(),
                        err.to_string()
                    ),
                    ErrorAction::Notify,
                );

                send_palantir_error_notification(error, Local::now(), self.palantir_sender.clone());

                return false;
            }
        }
    }
}
