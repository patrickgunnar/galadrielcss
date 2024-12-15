use core::str;
use std::{path::PathBuf, sync::Arc, time::SystemTime};

use chrono::Local;
use ignore::{overrides, WalkBuilder};
use lazy_static::lazy_static;
use regex::Regex;
use tokio::sync::{broadcast, RwLock};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    utils::send_palantir_error_notification::send_palantir_error_notification,
};

const PATTERN: &str = r"@(?:class|layout|module):([a-zA-Z0-9_-]+)(?:::([a-zA-Z0-9_-]+))?";

lazy_static! {
    pub static ref MARKUP_RE: Regex = Regex::new(PATTERN).unwrap();
}

#[derive(Clone, Debug)]
pub struct FileTimestampUpdater {
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
}

impl FileTimestampUpdater {
    pub fn new(palantir_sender: broadcast::Sender<GaladrielAlerts>) -> Self {
        Self { palantir_sender }
    }

    pub async fn process_from_folder(
        &self,
        path: PathBuf,
        matcher: Arc<RwLock<overrides::Override>>,
    ) {
        // Clone the matcher reference to avoid locking it multiple times.
        // Acquire a read lock on the matcher.
        let cloned_matcher = Arc::clone(&matcher);
        let matcher = cloned_matcher.read().await;

        // Initialize a directory walker to recursively traverse the directory.
        let walker = WalkBuilder::new(path)
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

            if self.is_component(&current_path, &matcher) {
                self.set_file_times(&current_path);
            }
        }
    }

    fn is_component(&self, current_path: &PathBuf, matcher: &overrides::Override) -> bool {
        current_path.is_file()
            && !matcher.matched(current_path, false).is_ignore()
            && current_path
                .extension()
                .map(|ext| {
                    ext == "js"
                        || ext == "jsx"
                        || ext == "ts"
                        || ext == "tsx"
                        || ext == "html"
                        || ext == "css"
                })
                .unwrap_or(false)
            && self.has_component_markup(current_path)
    }

    fn has_component_markup(&self, current_path: &PathBuf) -> bool {
        match std::fs::read_to_string(current_path) {
            Ok(file_content) => {
                return MARKUP_RE.is_match(&file_content)
                    || file_content.contains("@galadrielcss styles;");
            }
            Err(err) => {
                let error = GaladrielError::raise_general_other_error(
                    ErrorKind::Other,
                    &format!(". Error: {}", err.to_string()),
                    ErrorAction::Notify,
                );

                send_palantir_error_notification(error, Local::now(), self.palantir_sender.clone());

                return false;
            }
        }
    }

    fn set_file_times(&self, current_path: &PathBuf) {
        let palantir_sender = self.palantir_sender.clone();

        if let Err(err) = filetime::set_file_times(
            current_path,
            SystemTime::now().into(),
            SystemTime::now().into(),
        ) {
            let error = GaladrielError::raise_general_other_error(
                ErrorKind::Other,
                &err.to_string(),
                ErrorAction::Notify,
            );

            let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

            if let Err(err) = palantir_sender.send(notification) {
                tracing::error!("{:?}", err);
            }
        }
    }
}