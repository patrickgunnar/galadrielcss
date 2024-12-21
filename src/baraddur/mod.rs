use std::{path::PathBuf, sync::Arc};

use chrono::Local;
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
    crealion::CrealionContextType,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::{GaladrielAlerts, GaladrielEvents},
    formera::formera,
    gatekeeper::remove_path_from_gatekeeper,
    intaker::remove_context_from_intaker::remove_context_from_intaker,
    synthesizer::Synthesizer,
    trailblazer::Trailblazer,
    utils::{
        file_timestamp_updater::FileTimestampUpdater, inject_names::inject_names,
        is_nenyr_event::is_nenyr_event,
        send_palantir_error_notification::send_palantir_error_notification,
        send_palantir_notification::send_palantir_notification,
        send_palantir_success_notification::send_palantir_success_notification,
    },
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

                    send_palantir_error_notification(error, Local::now(), palantir_sender.clone());
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
        // Iterate over each debounced event.
        debounced_events.iter().for_each(|debounced_event| {
            // Check paths associated with the event.
            debounced_event.paths.iter().for_each(|path| {
                if Self::is_configuration_event(path, configuration_path) {
                    tracing::debug!(
                        "Detected configuration event for Galadriel CSS configurations. Path: {:?}",
                        path
                    );

                    // Handle events related to the configuration file.
                    Self::process_configuration_event(debounced_event.kind, processing_events);
                } else if is_nenyr_event(path, matcher) {
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
            Self::send_event_to_main_runtime(
                GaladrielEvents::Error(error),
                baraddur_sender.clone(),
            );
        } else {
            tracing::info!(
                "File watcher successfully started for directory: {:?}",
                working_dir
            );

            // Create and send a success notification to indicate that the watcher is active.
            send_palantir_success_notification(
                &Self::random_watch_message(),
                Local::now(),
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
                                working_dir,
                                path,
                                nenyr_parser,
                                Arc::clone(&matcher),
                                kind,
                                baraddur_sender.clone(),
                                palantir_sender.clone(),
                            )
                            .await;
                        }
                    }
                }
            }
            // Handle errors that occur during event reception.
            Err(err) => {
                tracing::error!("Failed to receive debounced event: {:?}", err);

                let error = GaladrielError::raise_general_observer_error(
                    ErrorKind::Other,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                send_palantir_error_notification(error, Local::now(), palantir_sender.clone());
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
                        send_palantir_notification(notification, palantir_sender.clone());
                    }
                    // Notify Palantir of any errors that occur during reconstruction.
                    Err(error) => {
                        tracing::error!("Failed to reconstruct exclude matcher: {:?}", error);
                        send_palantir_error_notification(
                            error,
                            starting_time,
                            palantir_sender.clone(),
                        );
                    }
                }

                tracing::info!("Galadriel CSS configurations updated successfully.");

                // Send a success notification indicating the system has been updated.
                send_palantir_success_notification(
                    "Galadriel CSS configurations updated successfully. System is now operating with the latest configuration.",
                    starting_time,
                    palantir_sender.clone()
                );
            }
            // Handle errors that occur during the configuration loading process.
            Err(error) => {
                tracing::error!("Failed to load Galadriel configurations: {:?}", error);

                send_palantir_error_notification(error, starting_time, palantir_sender.clone());
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
        working_dir: &PathBuf,
        current_path: &PathBuf,
        nenyr_parser: &mut NenyrParser,
        matcher: Arc<RwLock<overrides::Override>>,
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
                            send_palantir_error_notification(
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
                    working_dir,
                    nenyr_parser,
                    matcher,
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
        working_dir: &PathBuf,
        nenyr_parser: &mut NenyrParser,
        matcher: Arc<RwLock<overrides::Override>>,
        baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
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

        send_palantir_notification(notification, palantir_sender.clone());

        // Parses and processes the current Nenyr context.
        // This function returns a tuple containing:
        // - `context_type`: The type of the context being processed.
        // - `layout_relation`: The layout relationship associated with the parsed context.
        let (context_type, layout_relation) = formera(
            current_path.to_owned(),
            nenyr_parser,
            starting_time,
            palantir_sender.clone(),
        )
        .await;

        tracing::info!(
            "Parsed Nenyr context: {:?}, Layout relation: {:?}",
            context_type,
            layout_relation
        );

        // If the current processed context is a layout context and a layout relation exists,
        // this method iterates through the related module paths and reprocesses the associated
        // Nenyr module contexts.
        if let Some(layout_relation) = layout_relation {
            tracing::debug!("Layout relation found, processing related module paths.");

            let notification = GaladrielAlerts::create_information(
                Local::now(),
                &format!("Galadriel CSS has just started reprocessing the Module contexts related to the `{stringified_path}` Layout context, as the Layout context has been completely reprocessed. This is necessary to maintain all styles up-to-date.")
            );

            send_palantir_notification(notification, palantir_sender.clone());

            for module_path in layout_relation {
                tracing::debug!("Processing related module path: {:?}", module_path);

                let _ = formera(
                    PathBuf::from(module_path),
                    nenyr_parser,
                    Local::now(),
                    palantir_sender.clone(),
                )
                .await;
            }
        }

        // This method processes the current context type and sends appropriate events to
        // the main runtime. These events are then forwarded to the connected integration
        // client for further handling.
        Self::match_current_context_type_event(
            working_dir,
            &current_path,
            context_type,
            matcher,
            baraddur_sender.clone(),
            palantir_sender.clone(),
        )
        .await;

        tracing::info!("Parsing process completed for: {:?}", current_path);

        // Create a notification indicating that the parsing process has been completed.
        let notification = GaladrielAlerts::create_information(
            Local::now(),
            "The parsing process has concluded. All stages, including the interpretation, validation, and transformation of data, have been finalized. The system is now ready for subsequent operations or tasks."
        );

        send_palantir_notification(notification, palantir_sender.clone());
    }

    /// This method matches the current context type and sends corresponding events to the main runtime,
    /// which will then propagate them to the integration client.
    ///
    /// Depending on the context type, the method will trigger different events such as refreshing components
    /// or refreshing the entire application starting from the root or a parent folder.
    ///
    /// # Arguments
    /// - `working_dir`: The working directory, used for the processing contexts from the root of the application.
    /// - `current_path`: The path to the current context file.
    /// - `context_type`: An optional value representing the current context type (e.g., Layout, Module, or Central).
    /// - `matcher`: A reference to a `RwLock` containing an `Override` object for the exclude matching.
    /// - `baraddur_sender`: A sender used to send events to the main runtime.
    /// - `palantir_sender`: A broadcast sender used to send alerts to Palantir.
    ///
    /// This function processes the current path and context type to send the appropriate refresh event
    /// to the main runtime, notifying the integration client.
    async fn match_current_context_type_event(
        working_dir: &PathBuf,
        current_path: &PathBuf,
        context_type: Option<CrealionContextType>,
        matcher: Arc<RwLock<overrides::Override>>,
        _baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        tracing::info!(
            "Matching context type event for context: {:?}, path: {:?}",
            context_type,
            current_path
        );

        // Match the provided context type to determine the correct event to trigger.
        match context_type {
            // If the context type is Layout or Module, trigger the corresponding layout/module events.
            Some(CrealionContextType::Layout) | Some(CrealionContextType::Module) => {
                // Applies inheritance for Nenyr classes and their corresponding utility class names.
                Trailblazer::default().blazer();

                // Updates the CSS cache by transforming the most up-to-date styles.
                Astroform::new(
                    get_minified_styles(),
                    get_reset_styles(),
                    palantir_sender.clone(),
                )
                .transform()
                .await;

                if let Some(parent_path) = current_path.parent() {
                    tracing::debug!("Sending refresh event for folder: {:?}", parent_path);

                    FileTimestampUpdater::new(palantir_sender.clone())
                        .process_from_folder(false, parent_path.to_owned(), Arc::clone(&matcher))
                        .await;
                }
            }
            // If the context type is Central.
            Some(CrealionContextType::Central) => {
                tracing::info!(
                    "Context type is Central, initiating reprocess of Nenyr contexts from root."
                );

                let notification = GaladrielAlerts::create_information(
                    Local::now(),
                    "Galadriel CSS has just started reprocessing the Layout and Module contexts of the application, as the Central context has been fully reprocessed. This is necessary to keep all styles up-to-date."
                );

                send_palantir_notification(notification, palantir_sender.clone());

                // Reprocess all layout and module contexts from the application, excluding the central context.
                Synthesizer::new(false, Arc::clone(&matcher), palantir_sender.clone())
                    .process(get_minified_styles(), working_dir)
                    .await;

                tracing::debug!("Synthesizer process completed. Sending refresh event from root.");

                FileTimestampUpdater::new(palantir_sender.clone())
                    .process_from_folder(false, working_dir.to_owned(), Arc::clone(&matcher))
                    .await;
            }
            None => {}
        }

        FileTimestampUpdater::new(palantir_sender.clone())
            .process_from_folder(true, working_dir.to_owned(), matcher)
            .await;
    }

    /// Sends an event to the main runtime through the Baraddur sender.
    ///
    /// # Arguments
    /// - `event`: The event to be sent to the main runtime.
    /// - `baraddur_sender`: The sender responsible for transmitting the event.
    ///
    /// This function tries to send the provided event to the main runtime. If the sending process fails,
    /// an error is logged.
    fn send_event_to_main_runtime(
        event: GaladrielEvents,
        baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
    ) {
        if let Err(err) = baraddur_sender.send(event) {
            tracing::error!("Failed to send notification to main runtime: {:?}", err);
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
