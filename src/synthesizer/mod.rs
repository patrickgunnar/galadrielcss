use std::{path::PathBuf, sync::Arc};

use chrono::Local;
use ignore::{overrides, WalkBuilder};
use nenyr::NenyrParser;
use tokio::sync::{broadcast, RwLock};

use crate::{
    astroform::Astroform,
    configatron::{get_minified_styles, get_reset_styles},
    events::GaladrielAlerts,
    formera::formera,
    trailblazer::Trailblazer,
    utils::is_nenyr_event::is_nenyr_event,
};

/// `Synthesizer` is responsible for reprocessing all Nenyr contexts in the application.
/// It traverses directories, identifies Nenyr context files (central, layout, or module),
/// and processes them in the correct order.
///
/// # Fields
/// - `include_central`: A flag indicating whether to include the central context in processing.
/// - `central_context`: Path to the central context file.
/// - `layout_contexts`: A vector holding paths to layout context files.
/// - `module_contexts`: A vector holding paths to module context files.
/// - `matcher`: A reference to the matcher used for context filtering.,
/// - `palantir_sender`: A broadcast sender used for sending alerts.
#[derive(Clone, Debug)]
pub struct Synthesizer {
    include_central: bool,
    central_context: PathBuf,
    layout_contexts: Vec<PathBuf>,
    module_contexts: Vec<PathBuf>,
    matcher: Arc<RwLock<overrides::Override>>,
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
}

impl Synthesizer {
    /// Constructs a new `Synthesizer` instance with the specified configuration.
    ///
    /// # Arguments
    /// - `include_central`: A flag to specify whether the central context should be included in the processing.
    /// - `matcher`: A reference to the matcher used for context filtering.
    /// - `palantir_sender`: A sender used to broadcast alerts.
    ///
    /// # Returns
    /// Returns an instance of `Synthesizer`.
    pub fn new(
        include_central: bool,
        matcher: Arc<RwLock<overrides::Override>>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) -> Self {
        Self {
            central_context: PathBuf::new(),
            layout_contexts: vec![],
            module_contexts: vec![],
            palantir_sender,
            include_central,
            matcher,
        }
    }

    /// Processes the Nenyr contexts within the provided working directory.
    ///
    /// This function traverses the directory to find Nenyr context files (i.e., `central.nyr`, `layout.nyr`, and others),
    /// and categorizes them into central, layout, and module contexts for further processing.
    ///
    /// # Arguments
    /// - `working_dir`: The directory to traverse for Nenyr context files.
    ///
    /// # Returns
    /// This function is asynchronous and does not return a value.
    pub async fn process(&mut self, working_dir: &PathBuf) {
        tracing::info!(
            "Starting to process Nenyr contexts in directory: {:?}",
            working_dir
        );

        // Clone the matcher reference to avoid locking it multiple times.
        // Acquire a read lock on the matcher.
        let cloned_matcher = Arc::clone(&self.matcher);
        let matcher = cloned_matcher.read().await;

        // Initialize a directory walker to recursively traverse the directory.
        let walker = WalkBuilder::new(working_dir)
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

            // Check if the current path corresponds to a Nenyr event based on the matcher logic.
            if is_nenyr_event(&current_path, &matcher) {
                if let Some(file_name) = current_path.file_name() {
                    // Get the file name as a string.
                    let file_name = file_name.to_string_lossy().to_string();

                    // If the file is a central context file and the flag is set, store its path.
                    if file_name.ends_with("central.nyr") {
                        if self.include_central {
                            tracing::info!("Identified central context: {:?}", current_path);

                            self.central_context = current_path;
                        }
                    // If the file is a layout context file, store its path.
                    } else if file_name.ends_with("layout.nyr") {
                        tracing::info!("Identified layout context: {:?}", current_path);

                        self.layout_contexts.push(current_path);
                    // Otherwise, treat the file as a module context and store its path.
                    } else {
                        tracing::info!("Identified module context: {:?}", current_path);

                        self.module_contexts.push(current_path);
                    }
                }
            }
        }

        // After identifying all the relevant context files, start the parsing process.
        self.run_parsing().await;

        tracing::info!("Finished parsing and transforming all contexts.");
    }

    /// Runs the actual parsing for all the identified contexts (central, layout, and modules).
    ///
    /// This function processes the contexts in the order: central, then layout, and then modules.
    /// After parsing, it triggers a final transformation using `Astroform`.
    ///
    /// # Returns
    /// This function is asynchronous and does not return a value.
    async fn run_parsing(&mut self) {
        tracing::info!("Running parsing for contexts: central, layout, and modules.");

        // Create a new instance of the Nenyr parser.
        let mut nenyr_parser = NenyrParser::new();
        let palantir_sender = self.palantir_sender.clone();

        // A vector to hold all context paths in the correct order for processing.
        let mut ordered_contexts: Vec<PathBuf> = Vec::new();

        // If the central context is included, add its path to the order.
        if self.include_central {
            ordered_contexts.push(self.central_context.to_owned());
        }

        // Add layout and module contexts to the order of processing.
        ordered_contexts.append(&mut self.layout_contexts);
        ordered_contexts.append(&mut self.module_contexts);

        // For each context path in the ordered list, parse the corresponding Nenyr file.
        for context_path in ordered_contexts {
            tracing::info!("Parsing context file: {:?}", context_path);

            let _ = formera(
                context_path,
                &mut nenyr_parser,
                Local::now(),
                palantir_sender.clone(),
            )
            .await;
        }

        tracing::info!("Applying inheritance for Nenyr classes.");

        // Applies inheritance for Nenyr classes and their corresponding utility class names.
        Trailblazer::default().blazer();

        if self.include_central {
            tracing::info!("Transforming styles in CSS utility rules.");

            // Updates the CSS cache by transforming the most up-to-date styles.
            Astroform::new(
                get_minified_styles(),
                get_reset_styles(),
                palantir_sender.clone(),
            )
            .transform()
            .await;
        }
    }
}
