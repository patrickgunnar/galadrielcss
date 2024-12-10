use std::{path::PathBuf, sync::Arc};

use chrono::{DateTime, Local};
use events::{BaraddurEventProcessor, BaraddurEventProcessorKind, BaraddurRenameEventState};
use ignore::overrides;
use nenyr::NenyrParser;
use notify::{
    event::{CreateKind, ModifyKind, RenameMode},
    EventKind, RecommendedWatcher,
};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, RecommendedCache};
use rand::Rng;
use tokio::{
    runtime,
    sync::{self, mpsc, RwLock},
    task::JoinHandle,
};

use crate::{
    astroform::Astroform,
    configatron::{
        get_auto_naming, get_minified_styles, get_reset_styles, load_galadriel_configs,
        reconstruct_exclude_matcher,
    },
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::{GaladrielAlerts, GaladrielEvents},
    formera::Formera,
    gatekeeper::remove_path_from_gatekeeper,
    intaker::remove_context_from_intaker::remove_context_from_intaker,
    trailblazer::Trailblazer,
    utils::inject_names::inject_names,
    GaladrielResult,
};

pub mod events;

/// A struct to observe changes in a directory using an event-driven approach.
///
/// This struct facilitates monitoring a specified directory for events such as
/// file modifications, creations, or deletions. It employs unbounded channels
/// to handle event communication and uses a broadcast channel for alerting.
///
/// # Fields
/// - `baraddur_sender`: The unbounded sender for `GaladrielEvents`.
/// - `baraddur_receiver`: The unbounded receiver for `GaladrielEvents`.
/// - `palantir_sender`: Broadcast sender for sending `GaladrielAlerts`.
/// - `working_dir`: The directory where files or configurations are monitored for changes.
/// - `from_millis`: Duration in milliseconds used for debouncing events to minimize redundant notifications.
#[derive(Debug)]
pub struct Baraddur {
    /// Unbounded sender for transmitting system events (`GaladrielEvents`).
    baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
    /// Unbounded receiver for receiving system events (`GaladrielEvents`).
    baraddur_receiver: mpsc::UnboundedReceiver<GaladrielEvents>,
    /// Broadcast sender for sending alerts (`GaladrielAlerts`).
    palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,

    /// Path to the directory being monitored for file changes.
    ///
    /// The directory path is stored as a `PathBuf` to allow flexible manipulation
    /// and compatibility with the Rust file system APIs.
    working_dir: PathBuf,

    /// Duration in milliseconds used for debouncing file events.
    ///
    /// Debouncing prevents rapid consecutive notifications for the same event,
    /// ensuring more efficient and meaningful alerts.
    from_millis: u64,
}

impl Baraddur {
    /// Creates a new `Baraddur` instance with the given parameters.
    ///
    /// # Arguments
    /// - `from_millis`: Duration in milliseconds for debounce timing.
    /// - `working_dir`: Path to the working directory.
    /// - `palantir_sender`: A broadcast sender for alerts.
    ///
    /// # Returns
    /// A new instance of `Baraddur`.
    pub fn new(
        from_millis: u64,
        working_dir: PathBuf,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) -> Self {
        tracing::info!(
            "Creating a new Baraddur instance with debounce duration {}ms, working directory: {:?}",
            from_millis,
            working_dir
        );

        // Create an unbounded channel for `GaladrielEvents`.
        let (baraddur_sender, baraddur_receiver) = mpsc::unbounded_channel();

        tracing::info!("Baraddur instance created successfully.");

        Self {
            baraddur_sender,
            baraddur_receiver,
            palantir_sender,
            working_dir,
            from_millis,
        }
    }

    /// Asynchronously retrieves the next event from the receiver.
    ///
    /// # Errors
    /// Returns an error if no response is received from the `Baraddur` observer sender.
    ///
    /// # Returns
    /// A result containing the next `GaladrielEvent` or an error.
    pub async fn next(&mut self) -> GaladrielResult<GaladrielEvents> {
        // Wait for the next event from the receiver or return an error if none is received.
        self.baraddur_receiver.recv().await.ok_or_else(|| {
            tracing::error!(
                "Error while receiving response from Baraddur observer sender: No response received."
            );

            GaladrielError::raise_general_observer_error(
                ErrorKind::ObserverEventReceiveFailed,
                "Error while receiving response from Barad-dûr observer sender: No response received.",
                ErrorAction::Notify
            )
        })
    }

    /// Creates and configures an asynchronous debouncer.
    ///
    /// # Arguments
    /// - `matcher`: A shared `Override` matcher used for processing events.
    ///
    /// # Returns
    /// A tuple containing:
    /// - A configured `Debouncer` instance.
    /// - A broadcast sender for processed debouncer events.
    ///
    /// # Errors
    /// Returns an error if the debouncer cannot be created.
    pub fn async_debouncer(
        &self,
        matcher: Arc<RwLock<overrides::Override>>,
    ) -> GaladrielResult<(
        Debouncer<RecommendedWatcher, RecommendedCache>,
        sync::broadcast::Sender<Vec<BaraddurEventProcessor>>,
    )> {
        tracing::info!(
            "Creating an asynchronous debouncer with duration {}ms.",
            self.from_millis
        );

        // Create a broadcast channel for sending processed events.
        let (debouncer_sender, _) = sync::broadcast::channel(100);

        // Clone necessary variables for the asynchronous debouncer.
        let handle_view = runtime::Handle::current();
        let palantir_sender = self.palantir_sender.clone();
        let debouncer_tx = debouncer_sender.clone();
        let working_dir = self.working_dir.clone();

        // Create a new debouncer with the specified duration and configuration.
        let debouncer = new_debouncer(
            tokio::time::Duration::from_millis(self.from_millis),
            None,
            move |event_result| {
                tracing::debug!("Debouncer received an event result.");

                // Spawn an asynchronous task to process the debouncer events.
                let debouncer_sender = debouncer_tx.clone();
                let palantir_sender = palantir_sender.clone();
                let configuration_path = working_dir.join("galadriel.config.json");
                let matcher = Arc::clone(&matcher);

                handle_view.spawn(async move {
                    Self::match_async_debouncer_result(
                        &configuration_path,
                        Arc::clone(&matcher),
                        event_result,
                        debouncer_sender,
                        palantir_sender,
                    )
                    .await;
                });
            },
        )
        .map_err(|err| {
            tracing::error!("Failed to create the asynchronous debouncer: {}", err);

            GaladrielError::raise_critical_observer_error(
                ErrorKind::AsyncDebouncerCreationFailed,
                &err.to_string(),
                ErrorAction::Restart,
            )
        })?;

        tracing::info!("Asynchronous debouncer created successfully.");

        Ok((debouncer, debouncer_sender))
    }

    /// Matches and processes the results from the asynchronous debouncer.
    ///
    /// # Arguments
    /// - `configuration_path`: Path to the configuration file.
    /// - `matcher`: Shared override matcher for event matching.
    /// - `event_result`: Result containing debounced events or errors.
    /// - `debouncer_sender`: Broadcast sender for processed events.
    /// - `palantir_sender`: Broadcast sender for alerts.
    pub async fn match_async_debouncer_result(
        configuration_path: &PathBuf,
        matcher: Arc<RwLock<overrides::Override>>,
        event_result: Result<Vec<DebouncedEvent>, Vec<notify::Error>>,
        debouncer_sender: sync::broadcast::Sender<Vec<BaraddurEventProcessor>>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        match event_result {
            Ok(debounced_events) => {
                tracing::info!(
                    "Processing {} debounced events from the asynchronous debouncer.",
                    debounced_events.len()
                );

                // Initialize processing and rename state for events.
                let mut processing_events: Vec<BaraddurEventProcessor> = Vec::new();
                let mut rename_state = BaraddurRenameEventState::None;
                let matcher = matcher.read().await;

                // Process each debounced event.
                Self::process_debounced_events(
                    configuration_path,
                    &matcher,
                    debounced_events,
                    &mut rename_state,
                    &mut processing_events,
                );

                // Send processed events if any exist.
                if !processing_events.is_empty() {
                    if let Err(err) = debouncer_sender.send(processing_events) {
                        tracing::error!("Failed to send debounced events: {:?}", err);
                    }
                }
            }
            Err(errs) => {
                tracing::warn!(
                    "Received {} errors from the asynchronous debouncer.",
                    errs.len()
                );

                // Handle each error from the debounced events.
                for err in errs {
                    tracing::error!("Debouncer error: {}", err);

                    let error = GaladrielError::raise_general_observer_error(
                        ErrorKind::DebouncedEventError,
                        &err.to_string(),
                        ErrorAction::Notify,
                    );

                    Self::send_palantir_error_notification(
                        error,
                        Local::now(),
                        palantir_sender.clone(),
                    );
                }
            }
        }
    }

    /// Processes debounced file system events and classifies them into appropriate actions.
    ///
    /// # Arguments
    /// - `configuration_path`: Path to the configuration file to detect related events.
    /// - `matcher`: Matcher for identifying Nenyr-specific events.
    /// - `debounced_events`: A vector of debounced events captured from the file system watcher.
    /// - `rename_state`: Tracks the state of rename operations to handle multi-step rename events.
    /// - `processing_events`: A mutable vector to collect processed events.
    fn process_debounced_events(
        configuration_path: &PathBuf,
        matcher: &overrides::Override,
        debounced_events: Vec<DebouncedEvent>,
        rename_state: &mut BaraddurRenameEventState,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        tracing::info!(
            "Processing {} debounced events with configuration path: {:?}",
            debounced_events.len(),
            configuration_path
        );

        // Iterate over each debounced event.
        debounced_events.iter().for_each(|debounced_event| {
            tracing::debug!(
                "Processing debounced event: {:?} with paths: {:?}",
                debounced_event.kind,
                debounced_event.paths
            );

            // Check paths associated with the event.
            debounced_event.paths.iter().for_each(|path| {
                if Self::is_configuration_event(path, configuration_path) {
                    tracing::debug!(
                        "Detected configuration event for Galadriel CSS configurations. Path: {:?}",
                        path
                    );

                    // Handle events related to the configuration file.
                    Self::process_configuration_event(debounced_event.kind, processing_events);
                } else if Self::is_nenyr_event(path, matcher) {
                    tracing::debug!("Detected Nenyr-specific event for path: {:?}", path);

                    // Handle events related to Nenyr files.
                    Self::process_nenyr_event(
                        path,
                        debounced_event.kind,
                        rename_state,
                        processing_events,
                    );
                }
            });
        });

        tracing::info!("Completed processing debounced events.");
    }

    /// Processes events related to the configuration file.
    ///
    /// # Arguments
    /// - `debounced_event_kind`: The kind of the event (e.g., create, modify, remove).
    /// - `processing_events`: A mutable vector to collect processed events.
    fn process_configuration_event(
        debounced_event_kind: EventKind,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        // Determine the type of configuration event.
        // Only Modify and None are possible.
        // If None, return without doing any operation.
        // If Modify, adds the reload configs token to the processing events vector.
        if let BaraddurEventProcessorKind::None =
            Self::process_debounced_events_for_configs(debounced_event_kind)
        {
            tracing::trace!("No relevant action for configuration event.");
            return;
        }

        // Add a reload configuration event if it doesn't already exist.
        let reload_configs = BaraddurEventProcessor::ReloadGaladrielConfigs;
        Self::add_event_if_not_exists(reload_configs, processing_events);
        tracing::info!("Added reload configuration event to processing queue.");
    }

    /// Processes events related to Nenyr files.
    ///
    /// # Arguments
    /// - `path`: Path to the file triggering the event.
    /// - `debounced_event_kind`: The kind of the event (e.g., create, modify, remove).
    /// - `rename_state`: Tracks the state of rename operations to handle multi-step rename events.
    /// - `processing_events`: A mutable vector to collect processed events.
    fn process_nenyr_event(
        path: &PathBuf,
        debounced_event_kind: EventKind,
        rename_state: &mut BaraddurRenameEventState,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        tracing::debug!(
            "Processing Nenyr event for path: {:?} with kind: {:?}",
            path,
            debounced_event_kind
        );

        // Determine the type of Nenyr event.
        let event_kind =
            Self::process_debounced_events_for_nenyr(debounced_event_kind, rename_state);

        if let BaraddurEventProcessorKind::None = event_kind {
            tracing::trace!("No relevant action for Nenyr event on path: {:?}", path);

            return;
        }

        // Create a processing event for the Nenyr file and add it if not already present.
        let processing_event = BaraddurEventProcessor::ProcessEvent {
            kind: event_kind,
            path: path.to_owned(),
        };

        Self::add_event_if_not_exists(processing_event, processing_events);
        tracing::info!(
            "Added Nenyr processing event for path: {:?} to processing queue.",
            path
        );
    }

    /// Adds an event to the collection if it doesn't already exist.
    ///
    /// # Arguments
    /// - `processing_event`: The event to add.
    /// - `processing_events`: A mutable vector to collect processed events.
    fn add_event_if_not_exists(
        processing_event: BaraddurEventProcessor,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        // Avoid adding duplicate events to the collection.
        if !processing_events.contains(&processing_event) {
            tracing::debug!(
                "Adding new event to processing queue: {:?}",
                processing_event
            );

            processing_events.push(processing_event);
        }
    }

    /// Checks if a given path matches the configuration file path.
    ///
    /// # Arguments
    /// - `path`: Path to the file triggering the event.
    /// - `configuration_path`: Path to the configuration file.
    ///
    /// # Returns
    /// - `true` if the paths match, otherwise `false`.
    fn is_configuration_event(path: &PathBuf, configuration_path: &PathBuf) -> bool {
        path == configuration_path
    }

    /// Checks if a given path corresponds to a Nenyr file.
    ///
    /// # Arguments
    /// - `path`: Path to the file triggering the event.
    /// - `matcher`: Matcher for identifying Nenyr-specific events.
    ///
    /// # Returns
    /// - `true` if the path corresponds to a Nenyr file, otherwise `false`.
    fn is_nenyr_event(path: &PathBuf, matcher: &overrides::Override) -> bool {
        !matcher.matched(path, false).is_ignore()
            && path.extension().map(|ext| ext == "nyr").unwrap_or(false)
    }

    /// Determines the type of event for configuration files.
    ///
    /// # Arguments
    /// - `debounced_event_kind`: The kind of the event.
    ///
    /// # Returns
    /// - The corresponding event processor kind.
    fn process_debounced_events_for_configs(
        debounced_event_kind: EventKind,
    ) -> BaraddurEventProcessorKind {
        tracing::trace!(
            "Processing debounced event for configurations: {:?}",
            debounced_event_kind
        );

        match debounced_event_kind {
            EventKind::Modify(modified_kind) => match modified_kind {
                ModifyKind::Any | ModifyKind::Data(_) => BaraddurEventProcessorKind::Modify,
                _ => BaraddurEventProcessorKind::None,
            },
            EventKind::Create(CreateKind::File) => BaraddurEventProcessorKind::Modify,
            _ => BaraddurEventProcessorKind::None,
        }
    }

    /// Determines the type of event for Nenyr files.
    ///
    /// # Arguments
    /// - `debounced_event_kind`: The kind of the event.
    /// - `rename_state`: Tracks the state of rename operations to handle multi-step rename events.
    ///
    /// # Returns
    /// - The corresponding event processor kind.
    fn process_debounced_events_for_nenyr(
        debounced_event_kind: EventKind,
        rename_state: &mut BaraddurRenameEventState,
    ) -> BaraddurEventProcessorKind {
        tracing::trace!(
            "Processing debounced event for Nenyr files: {:?}, rename state: {:?}",
            debounced_event_kind,
            rename_state
        );

        match debounced_event_kind {
            EventKind::Modify(modified_kind) => match modified_kind {
                ModifyKind::Any | ModifyKind::Data(_) => BaraddurEventProcessorKind::Modify,
                ModifyKind::Name(rename_kind) => match rename_kind {
                    RenameMode::From => BaraddurEventProcessorKind::Remove,
                    RenameMode::To => BaraddurEventProcessorKind::Modify,
                    RenameMode::Both => {
                        if *rename_state == BaraddurRenameEventState::Rename {
                            *rename_state = BaraddurRenameEventState::None;

                            return BaraddurEventProcessorKind::Modify;
                        } else {
                            *rename_state = BaraddurRenameEventState::Rename;

                            return BaraddurEventProcessorKind::Remove;
                        }
                    }
                    _ => BaraddurEventProcessorKind::None,
                },
                _ => BaraddurEventProcessorKind::None,
            },
            EventKind::Create(CreateKind::File) => BaraddurEventProcessorKind::Modify,
            EventKind::Remove(_) => BaraddurEventProcessorKind::Remove,
            _ => BaraddurEventProcessorKind::None,
        }
    }

    /// Starts watching the filesystem for changes and processes events accordingly.
    ///
    /// # Parameters
    /// - `matcher`: Shared reference to the matcher used to filter relevant events.
    /// - `debouncer`: Debouncer for managing filesystem events efficiently.
    /// - `debouncer_sender`: Sender used to broadcast debounced events.
    ///
    /// # Returns
    /// - A `JoinHandle` representing the spawned asynchronous task.
    pub fn watch(
        &self,
        matcher: Arc<RwLock<overrides::Override>>,
        debouncer: &mut Debouncer<RecommendedWatcher, RecommendedCache>,
        debouncer_sender: sync::broadcast::Sender<Vec<BaraddurEventProcessor>>,
    ) -> JoinHandle<()> {
        tracing::info!(
            "Starting watcher for working directory: {:?}",
            self.working_dir
        );

        let baraddur_sender = self.baraddur_sender.clone(); // Clone the sender for internal use.
        let palantir_sender = self.palantir_sender.clone(); // Clone the Palantir notification sender.
        let working_dir = self.working_dir.clone(); // Clone the working directory path.

        let mut palantir_receiver = palantir_sender.subscribe(); // Subscribe to Palantir notifications.
        let mut debouncer_receiver = debouncer_sender.subscribe(); // Subscribe to debounced events.
        let mut nenyr_parser = NenyrParser::new(); // Initialize the Nenyr parser.

        // Attempt to start watching the specified working directory for filesystem changes.
        if let Err(err) = debouncer.watch(working_dir.clone(), notify::RecursiveMode::Recursive) {
            tracing::error!(
                "Failed to watch directory {:?}: {}",
                working_dir,
                err.to_string()
            );

            // Raise a critical error if the watch setup fails.
            let error = GaladrielError::raise_critical_observer_error(
                ErrorKind::DebouncerWatchFailed,
                &err.to_string(),
                ErrorAction::Restart,
            );

            // Attempt to send the error to the main runtime.
            if let Err(err) = baraddur_sender.send(GaladrielEvents::Error(error)) {
                tracing::error!(
                    "Failed to send error notification to main runtime: {:?}",
                    err
                );
            }
        } else {
            tracing::info!(
                "File watcher successfully started for directory: {:?}",
                working_dir
            );

            // Create and send a success notification to indicate that the watcher is active.
            let starting_time = Local::now();
            Self::send_palantir_success_notification(
                &Self::random_watch_message(),
                starting_time,
                palantir_sender.clone(),
            );
        }

        // Spawn an asynchronous task to process incoming events.
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Exit the loop if the Baraddur sender is closed.
                    _ = baraddur_sender.closed() => {
                        tracing::info!("Baraddur sender closed. Exiting watcher loop.");
                        break;
                    }
                    // Exit the loop if the Palantir receiver is closed.
                    Err(sync::broadcast::error::RecvError::Closed) = palantir_receiver.recv() => {
                        tracing::info!("Palantir receiver closed. Exiting watcher loop.");
                        break;
                    }
                    // Process debounced events when received.
                    debounced_event_result = debouncer_receiver.recv() => {
                        tracing::trace!("Received debounced event result: {:?}", debounced_event_result);

                        Self::match_debounced_result(
                            &working_dir,
                            &mut nenyr_parser,
                            Arc::clone(&matcher),
                            baraddur_sender.clone(),
                            palantir_sender.clone(),
                            &debounced_event_result
                        )
                        .await;
                    }
                }
            }
        })
    }

    /// Matches and processes the results of debounced filesystem events.
    ///
    /// # Parameters
    /// - `working_dir`: Path to the working directory.
    /// - `nenyr_parser`: Reference to the Nenyr parser.
    /// - `matcher`: Shared reference to the matcher for filtering events.
    /// - `palantir_sender`: Sender for Palantir notifications.
    /// - `debounced_event_result`: Result containing debounced events or an error.
    async fn match_debounced_result(
        working_dir: &PathBuf,
        nenyr_parser: &mut NenyrParser,
        matcher: Arc<RwLock<overrides::Override>>,
        baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
        debounced_event_result: &Result<
            Vec<BaraddurEventProcessor>,
            sync::broadcast::error::RecvError,
        >,
    ) {
        tracing::debug!(
            "Matching debounced result for events in working directory: {:?}",
            working_dir
        );

        match debounced_event_result {
            // Process each debounced event if received successfully.
            Ok(debounced_events) => {
                tracing::info!(
                    "Received {} debounced events to process.",
                    debounced_events.len()
                );

                for debounced_event in debounced_events {
                    match debounced_event {
                        // Handle configuration reload events.
                        BaraddurEventProcessor::ReloadGaladrielConfigs => {
                            tracing::info!("Processing ReloadGaladrielConfigs event.");

                            Self::reload_galadriel_configs(
                                working_dir,
                                Arc::clone(&matcher),
                                palantir_sender.clone(),
                            )
                            .await;
                        }
                        // Handle Nenyr processing events based on their kind and path.
                        BaraddurEventProcessor::ProcessEvent { kind, path } => {
                            tracing::info!("Processing event: {:?} for path: {:?}", kind, path);

                            Self::match_processing_event_kind(
                                path,
                                nenyr_parser,
                                kind,
                                baraddur_sender.clone(),
                                palantir_sender.clone(),
                            )
                            .await;
                        }
                    }
                }

                Astroform::new(
                    get_minified_styles(),
                    get_reset_styles(),
                    palantir_sender.clone(),
                )
                .transform()
                .await;

                // TODO: Make the observer notify the integration client to reload the CSS after processing a context or the configuration.
            }
            // Handle errors that occur during event reception.
            Err(err) => {
                tracing::error!("Failed to receive debounced event: {:?}", err);

                let error = GaladrielError::raise_general_observer_error(
                    ErrorKind::Other,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );
            }
        }
    }

    /// Reloads the Galadriel CSS configurations and updates the matcher.
    ///
    /// # Parameters
    /// - `working_dir`: Path to the working directory containing the configuration files.
    /// - `matcher`: Shared reference to the matcher for excluding or including paths.
    /// - `palantir_sender`: Sender used to broadcast alerts or notifications.
    async fn reload_galadriel_configs(
        working_dir: &PathBuf,
        matcher: Arc<RwLock<overrides::Override>>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        tracing::info!(
            "Starting Galadriel configurations reload for directory: {:?}",
            working_dir
        );

        let starting_time = Local::now(); // Record the start time for tracking performance.

        // Attempt to load the Galadriel configurations.
        match load_galadriel_configs(working_dir).await {
            Ok(()) => {
                tracing::info!("Galadriel configurations loaded successfully.");

                // If configurations are loaded successfully, reconstruct the exclude matcher.
                match reconstruct_exclude_matcher(working_dir, matcher).await {
                    // Notify Palantir of the successful matcher reconstruction.
                    Ok(notification) => {
                        tracing::info!("Exclude matcher reconstructed successfully.");
                        Self::send_palantir_notification(notification, palantir_sender.clone());
                    }
                    // Notify Palantir of any errors that occur during reconstruction.
                    Err(error) => {
                        tracing::error!("Failed to reconstruct exclude matcher: {:?}", error);
                        Self::send_palantir_error_notification(
                            error,
                            starting_time,
                            palantir_sender.clone(),
                        );
                    }
                }

                tracing::info!("Galadriel CSS configurations updated successfully.");

                // Send a success notification indicating the system has been updated.
                Self::send_palantir_success_notification(
                    "Galadriel CSS configurations updated successfully. System is now operating with the latest configuration.",
                    starting_time,
                    palantir_sender.clone()
                );
            }
            // Handle errors that occur during the configuration loading process.
            Err(error) => {
                tracing::error!("Failed to load Galadriel configurations: {:?}", error);

                Self::send_palantir_error_notification(
                    error,
                    starting_time,
                    palantir_sender.clone(),
                );
            }
        }
    }

    /// Matches the kind of processing event and performs the appropriate action.
    ///
    /// # Parameters
    /// - `current_path`: Path to the file or resource being processed.
    /// - `processing_event_kind`: Type of the event (e.g., `Modify`, `Remove`).
    /// - `nenyr_parser`: Reference to the Nenyr parser for handling Nenyr files.
    /// - `palantir_sender`: Sender used to broadcast alerts or notifications.
    async fn match_processing_event_kind(
        current_path: &PathBuf,
        nenyr_parser: &mut NenyrParser,
        processing_event_kind: &BaraddurEventProcessorKind,
        baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        tracing::debug!(
            "Matching processing event kind: {:?} for path: {:?}",
            processing_event_kind,
            current_path
        );

        match processing_event_kind {
            // If the event is a modification, process the associated Nenyr file.
            BaraddurEventProcessorKind::Modify => {
                // Checks if the auto-naming configuration is enabled.
                // If enabled, it processes the injection of context, class, and animation names
                // into the Nenyr context for the current path.
                if get_auto_naming() {
                    // Attempts to inject names into the Nenyr context for the provided path.
                    match inject_names(current_path.to_owned()).await {
                        // If names were successfully injected, the operation is complete, and the function returns early.
                        Ok(was_injected) if was_injected => {
                            return;
                        }
                        // If injection was successful but no names were injected, continue execution.
                        Ok(_) => {}
                        // If an error occurs during injection, send a Palantir error notification.
                        Err(error) => {
                            Self::send_palantir_error_notification(
                                error,
                                Local::now(),
                                palantir_sender.clone(),
                            );
                        }
                    }
                }

                tracing::info!(
                    "Processing Nenyr file for modification at path: {:?}",
                    current_path
                );

                Self::process_nenyr_file(
                    current_path.to_owned(),
                    nenyr_parser,
                    baraddur_sender,
                    palantir_sender.clone(),
                )
                .await;
            }
            // If the event is a removal, remove the path from gatekeeper and intaker.
            BaraddurEventProcessorKind::Remove => {
                let file_path = current_path.to_string_lossy().to_string();

                tracing::info!("Removing path from gatekeeper and intaker: {}", file_path);

                remove_path_from_gatekeeper(&file_path);
                remove_context_from_intaker(&file_path);
            }
            _ => {}
        }
    }

    /// Processes a Nenyr file and sends relevant notifications.
    ///
    /// # Parameters
    /// - `current_path`: Path to the Nenyr file being processed.
    /// - `nenyr_parser`: Reference to the Nenyr parser for parsing.
    /// - `palantir_sender`: Sender used to broadcast alerts or notifications.
    async fn process_nenyr_file(
        current_path: PathBuf,
        nenyr_parser: &mut NenyrParser,
        _baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        let stringified_path = current_path.to_string_lossy().to_string(); // Convert path to a string.
        let starting_time = Local::now(); // Record the start time for performance tracking.

        tracing::info!("Initiating parsing of Nenyr file: {:?}", stringified_path);

        // Notify Palantir that parsing is starting.
        let notification = GaladrielAlerts::create_information(
            starting_time,
            &format!(
                "Starting the parsing process for the specified path: {:?}",
                stringified_path
            ),
        );

        Self::send_palantir_notification(notification, palantir_sender.clone());

        let mut formera = Formera::new(current_path, palantir_sender.clone());

        // Attempt to start parsing the Nenyr file.
        match formera.start(nenyr_parser).await {
            // Notify Palantir on successful parsing.
            Ok(layout_relation) => {
                tracing::info!("Successfully parsed Nenyr file: {:?}", stringified_path);

                Self::send_palantir_success_notification(
                    &format!("The Nenyr file located at {:?} has been successfully parsed and processed without any errors. All relevant data has been extracted and is ready for further operations.", stringified_path),
                    starting_time,
                    palantir_sender.clone(),
                );

                if let Some(layout_relation) = layout_relation {
                    let notification = GaladrielAlerts::create_information(
                        Local::now(),
                        &format!(
                            "The current layout context contains these relations: {:?}",
                            layout_relation
                        ),
                    );

                    Self::send_palantir_notification(notification, palantir_sender.clone());
                }

                Trailblazer::default().blazer();

                // TODO: 1. Reprocessing Layout Contexts
                // - Develop a script to reprocess the module contexts derived from the layout's processing.
                // - Send a command to the integration client instructing it to reprocess the application starting from the folder
                //   where the layout context resides, including all components within that folder.
                // - Provide the path of the layout to the integration client, ensuring it knows the starting point for reprocessing
                //   the application's components.

                // TODO: 2. Reprocessing the Entire Application
                // - Create a script capable of reprocessing all the layout and module contexts in the application after reprocessing
                //   a central context.
                // - Send a command to the integration client to reprocess the entire application starting from the root directory.

                // TODO: 3. Reprocessing Module Contexts
                // - Trigger a notification for the integration client to reprocess the corresponding JS component.
                // - Provide the module context's path to the integration client, enabling it to identify and reprocess the specific
                //   application component.
            }
            // Handle Nenyr-specific errors.
            Err(GaladrielError::NenyrError { start_time, error }) => {
                tracing::error!(
                    "Nenyr error occurred while parsing file: {:?}",
                    stringified_path
                );
                tracing::error!("Error details: {:?}", error);

                let notification =
                    GaladrielAlerts::create_nenyr_error(start_time.to_owned(), error.to_owned());

                Self::send_palantir_notification(notification, palantir_sender.clone());
            }
            // Handle other errors and notify Palantir.
            Err(error) => {
                tracing::error!(
                    "Unexpected error occurred while processing Nenyr file: {:?}",
                    stringified_path
                );
                tracing::error!("Error details: {:?}", error);

                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );
            }
        }
    }

    /// Sends a success notification to Palantir with details about the operation's duration.
    ///
    /// # Parameters
    /// - `message`: A success message describing the operation.
    /// - `starting_time`: The timestamp when the operation started.
    /// - `palantir_sender`: Sender used to broadcast the success notification.
    fn send_palantir_success_notification(
        message: &str,
        starting_time: DateTime<Local>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        let ending_time = Local::now(); // Record the end time of the operation.
        let duration = ending_time - starting_time; // Calculate the operation's duration.

        tracing::info!(
            "Operation completed successfully in {:?}, duration: {:?}",
            ending_time,
            duration
        );

        // Create a success notification with timestamps and duration.
        let notification =
            GaladrielAlerts::create_success(starting_time, ending_time, duration, message);

        // Send the success notification.
        Self::send_palantir_notification(notification, palantir_sender.clone());
    }

    /// Sends an error notification to Palantir with details about the encountered error.
    ///
    /// # Parameters
    /// - `error`: The error encountered during the operation.
    /// - `starting_time`: The timestamp when the operation started.
    /// - `palantir_sender`: Sender used to broadcast the error notification.
    fn send_palantir_error_notification(
        error: GaladrielError,
        starting_time: DateTime<Local>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        tracing::error!("Error occurred during operation. Details: {:?}", error);

        // Create an error notification with the starting timestamp and error details.
        let notification = GaladrielAlerts::create_galadriel_error(starting_time, error);

        // Send the error notification.
        Self::send_palantir_notification(notification, palantir_sender.clone());
    }

    /// Sends a notification to Palantir. Handles errors if the notification fails to send.
    ///
    /// # Parameters
    /// - `notification`: The notification to be sent.
    /// - `palantir_sender`: Sender used to broadcast the notification.
    fn send_palantir_notification(
        notification: GaladrielAlerts,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        // Attempt to send the notification. Log an error if it fails.
        if let Err(err) = palantir_sender.send(notification) {
            tracing::error!("Failed to send alert: {:?}", err);
        }
    }

    /// Generates a random subtitle message indicating that the system is observing changes.
    ///
    /// # Returns
    /// A randomly selected message as a `String`.
    fn random_watch_message() -> String {
        let messages = [
            "Barad-dûr keeps watch over the realm. All changes are being observed.",
            "The Eye of Sauron turns its gaze upon your files. Observing all...",
            "The Dark Tower stands vigilant. All modifications will be noted.",
            "A shadow moves in the East... your files are under careful surveillance.",
            "Barad-dûr has awakened. All changes in the application are being observed.",
        ];

        let idx = rand::thread_rng().gen_range(0..messages.len());
        let selected_message = String::from(messages[idx]);

        tracing::debug!(
            "Selected random watch subtitle message: {}",
            selected_message
        );

        selected_message
    }
}
