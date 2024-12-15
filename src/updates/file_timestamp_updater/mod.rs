use std::{path::PathBuf, sync::Arc, time::SystemTime};

use chrono::Local;
use ignore::{overrides, WalkBuilder};
use tokio::sync::{broadcast, RwLock};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
};

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
        !matcher.matched(current_path, false).is_ignore()
            && current_path
                .extension()
                .map(|ext| {
                    ext == "js" || ext == "jsx" || ext == "ts" || ext == "tsx" || ext == "html"
                })
                .unwrap_or(false)
    }

    pub fn process_from_file(&self, path: PathBuf) {
        for ext in ["js", "jsx", "ts", "tsx", "html"] {
            let mut complete_path = path.to_path_buf();

            complete_path.set_extension(ext);

            if complete_path.exists() {
                self.set_file_times(&complete_path);
                break;
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
