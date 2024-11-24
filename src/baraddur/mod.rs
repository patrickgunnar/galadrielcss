use std::{path::PathBuf, sync::Arc, time::Duration};

use chrono::Local;
use events::{DebouncedWatch, ProcessingState};
use ignore::overrides;
use notify::{EventKind, RecommendedWatcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, RecommendedCache};
use rand::Rng;
use tokio::{
    runtime,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielEvents,
    shellscape::alerts::ShellscapeAlerts,
    GaladrielResult,
};

pub mod events;

/// A struct to observe changes in a directory using an event-driven approach.
///
/// `BaraddurObserver` provides functionality to monitor a directory for file system changes,
/// utilizing an event-driven mechanism. Events are sent over an unbounded channel and handled
/// asynchronously to prevent blocking the main runtime.
#[derive(Debug)]
pub struct BaraddurObserver {
    /// Sender for observer events, allowing for asynchronous event notification.
    observer_sender: UnboundedSender<GaladrielEvents>,
    /// Receiver for observer events, used to await incoming events.
    observer_receiver: UnboundedReceiver<GaladrielEvents>,

    /// The directory path to monitor.
    working_dir: PathBuf,
    /// The debounce interval in milliseconds, controlling the delay between events.
    from_millis: u64,
}

impl BaraddurObserver {
    /// Creates a new `BaraddurObserver` instance with a fresh sender and receiver for events.
    ///
    /// # Arguments
    /// * `working_dir` - The path of the directory to observe.
    /// * `from_millis` - The debounce interval for handling events.
    ///
    /// # Returns
    /// A new `BaraddurObserver` instance.
    pub fn new(working_dir: PathBuf, from_millis: u64) -> Self {
        let (observer_sender, observer_receiver) = mpsc::unbounded_channel();

        info!("BaraddurObserver instance created with new event sender and receiver.");

        Self {
            observer_sender,
            observer_receiver,
            working_dir,
            from_millis,
        }
    }

    /// Awaits and retrieves the next event from the observer.
    ///
    /// # Returns
    /// A `GaladrielResult` wrapping the next event (`GaladrielEvents`).
    ///
    /// # Errors
    /// Returns an error if the channel is closed or if an IO error occurs.
    pub async fn next(&mut self) -> GaladrielResult<GaladrielEvents> {
        self.observer_receiver.recv().await.ok_or_else(|| {
            error!("Failed to receive Barad-dûr observer event: Channel closed unexpectedly or an IO error occurred");

            GaladrielError::raise_general_observer_error(
                ErrorKind::ObserverEventReceiveFailed,
                "Error while receiving response from Barad-dûr observer sender: No response received.",
                ErrorAction::Notify
            )
        })
    }

    /// Starts the observer in a separate asynchronous task, monitoring a directory for changes.
    ///
    /// # Arguments
    /// * `matcher` - A matcher object used to determine whether certain files should not be processed.
    ///
    /// # Returns
    /// A `JoinHandle` for the spawned async task, allowing for control over the background task.
    pub fn start(&self, matcher: Arc<RwLock<overrides::Override>>) -> JoinHandle<()> {
        let observer_sender = self.observer_sender.clone();
        let working_dir = self.working_dir.clone();
        let from_millis = self.from_millis.clone();

        // Spawn an asynchronous task to start the directory watch
        tokio::spawn(async move {
            info!(
                "Starting Barad-dûr observer with working directory {:?}",
                working_dir
            );

            if let Err(err) =
                Self::async_watch(matcher, observer_sender.clone(), working_dir, from_millis).await
            {
                let err = GaladrielError::raise_general_observer_error(
                    ErrorKind::AsyncWatcherInitializationFailed,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                error!("Failed to start async watch: {:?}", err);

                if let Err(err) = observer_sender.send(GaladrielEvents::Error(err)) {
                    error!(
                        "Failed to send async debouncer error to main runtime: {:?}",
                        err
                    );
                }
            }
        })
    }

    /// Initializes a debouncer for handling file system events with a specified delay.
    ///
    /// # Arguments
    /// * `observer_sender` - The sender for observer events, which forwards notifications to the main runtime.
    /// * `from_millis` - The debounce interval in milliseconds, controlling how frequently events are sent.
    ///
    /// # Returns
    /// A `GaladrielResult` wrapping a tuple containing the debouncer and receiver channels.
    fn async_debouncer(
        observer_sender: UnboundedSender<GaladrielEvents>,
        from_millis: u64,
    ) -> GaladrielResult<(
        Debouncer<RecommendedWatcher, RecommendedCache>,
        mpsc::Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>,
    )> {
        let (debouncer_sender, debouncer_receiver) = mpsc::channel(1);
        let handle_view = runtime::Handle::current();

        // Create a new debouncer with the provided interval
        let debouncer = new_debouncer(Duration::from_millis(from_millis), None, move |response| {
            let debouncer_sender = debouncer_sender.clone();
            let observer_sender = observer_sender.clone();

            handle_view.spawn(async move {
                if let Err(err) = debouncer_sender.send(response).await {
                    let err = GaladrielError::raise_general_observer_error(
                        ErrorKind::NotificationSendError,
                        &err.to_string(),
                        ErrorAction::Notify,
                    );

                    error!("Error sending response to debouncer channel: {:?}", err);

                    if let Err(err) = observer_sender.send(GaladrielEvents::Error(err)) {
                        error!(
                            "Failed to send error notification to main runtime: {:?}",
                            err
                        );
                    }
                }
            });
        })
        .map_err(|err| {
            GaladrielError::raise_critical_observer_error(
                ErrorKind::AsyncDebouncerCreationFailed,
                &err.to_string(),
                ErrorAction::Restart,
            )
        })?;

        info!(
            "Initialized async debouncer with a delay of {} ms",
            from_millis
        );

        Ok((debouncer, debouncer_receiver))
    }

    /// Starts watching the specified directory asynchronously for file system changes.
    ///
    /// # Arguments
    /// * `matcher` - A matcher object for filtering files.
    /// * `observer_sender` - The sender for observer events.
    /// * `working_dir` - The directory to watch.
    /// * `from_millis` - The debounce interval in milliseconds.
    ///
    /// # Returns
    /// A `GaladrielResult` wrapping the success or failure of the watch operation.
    async fn async_watch(
        matcher: Arc<RwLock<overrides::Override>>,
        observer_sender: UnboundedSender<GaladrielEvents>,
        working_dir: PathBuf,
        from_millis: u64,
    ) -> GaladrielResult<()> {
        // Set up debouncer and receiver channels
        let (mut debouncer, mut debouncer_receiver) =
            Self::async_debouncer(observer_sender.clone(), from_millis)?;

        info!("Starting to watch directory {:?}", working_dir);

        let start_time = Local::now();
        let ending_time = Local::now();
        let duration = ending_time - start_time;

        let notification = ShellscapeAlerts::create_success(
            start_time,
            ending_time,
            duration,
            &random_watch_message(),
        );

        // Send a header event to observer sender as an initial notification
        observer_sender
            .send(GaladrielEvents::Notify(notification))
            .map_err(|err| {
                GaladrielError::raise_general_observer_error(
                    ErrorKind::NotificationSendError,
                    &err.to_string(),
                    ErrorAction::Restart,
                )
            })?;

        // Set up the recursive watch on the specified directory
        debouncer
            .watch(working_dir.clone(), notify::RecursiveMode::Recursive)
            .map_err(|err| {
                error!(
                    "Failed to set up recursive watch on directory {:?}: {:?}",
                    working_dir, err
                );

                GaladrielError::raise_critical_observer_error(
                    ErrorKind::DebouncerWatchFailed,
                    &err.to_string(),
                    ErrorAction::Restart,
                )
            })?;

        // Path to the Galadriel config file and initial processing state
        let config_path = working_dir.join("galadriel.config.json");
        let mut processing_state = ProcessingState::Awaiting;

        // Spawn a task to handle the debounced events
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle closed observer sender channel
                    _ = observer_sender.closed() => {
                        warn!("Observer sender channel closed, stopping watch loop.");
                        break;
                    }
                    // Process debounced events from the receiver
                    debounced_result = debouncer_receiver.recv() => {
                        if let DebouncedWatch::Break = Self::match_debounced_result(
                            debounced_result,
                            Arc::clone(&matcher),
                            observer_sender.clone(),
                            &mut processing_state,
                            &config_path
                        ).await
                        {
                            break;
                        }
                    }
                }
            }
        })
        .await
        .map_err(|err| {
            error!("Error completing async watch task: {:?}", err);

            GaladrielError::raise_critical_observer_error(
                ErrorKind::AsyncDebouncerWatchError,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        info!("Async watch completed successfully.");
        Ok(())
    }

    /// Matches and handles debounced results received from the watcher.
    ///
    /// # Arguments
    /// * `debounced_result` - The result of the debounced watch, containing events or errors.
    /// * `matcher` - A shared matcher object for filtering files.
    /// * `observer_sender` - The sender for observer events.
    /// * `processing_state` - The current processing state of the observer.
    /// * `config_path` - Path to the configuration file for Galadriel.
    ///
    /// # Returns
    /// A `DebouncedWatch` enum indicating whether to continue or break the loop.
    async fn match_debounced_result(
        debounced_result: Option<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>,
        matcher: Arc<RwLock<overrides::Override>>,
        observer_sender: UnboundedSender<GaladrielEvents>,
        processing_state: &mut ProcessingState,
        config_path: &PathBuf,
    ) -> DebouncedWatch {
        match debounced_result {
            // Process valid debounced events
            Some(Ok(debounced_events)) => {
                let matcher = matcher.read().await;

                'looping: for debounced_event in debounced_events {
                    let path = debounced_event.paths[0].clone();

                    // Check if the path is a file and passes the matcher filter
                    if path.is_file() && !matcher.matched(&path, false).is_ignore() {
                        match debounced_event.kind {
                            // Handle modification events
                            EventKind::Modify(_modified) => {
                                info!("Detected modification event at path: {:?}", path);

                                Self::handle_current_event(
                                    observer_sender.clone(),
                                    processing_state,
                                    config_path,
                                    &path,
                                )
                                .await;

                                break 'looping;
                            }
                            // Handle file removal events
                            EventKind::Remove(_) => {
                                warn!("Detected removal event at path: {:?}", path);

                                break 'looping;
                            }
                            _ => {}
                        }
                    }
                }
            }
            // Handle errors in the debounced events receiver
            Some(Err(err)) => {
                let err = GaladrielError::raise_general_observer_error(
                    ErrorKind::DebouncedEventError,
                    &err[0].to_string(),
                    ErrorAction::Notify,
                );

                error!("Error in debounced events receiver: {:?}", err);

                if let Err(err) = observer_sender.send(GaladrielEvents::Error(err)) {
                    error!("Failed to send debouncer error to main runtime: {:?}", err);
                }

                return DebouncedWatch::Break;
            }
            None => {}
        }

        DebouncedWatch::Continue
    }

    /// Handles a file system event by sending notifications and updating the processing state.
    ///
    /// # Arguments
    /// * `observer_sender`: The sender for observer events.
    /// * `processing_state`: The current processing state.
    /// * `matcher`: The matcher for filtering files.
    /// * `config_path`: The path to the config file.
    /// * `path`: The path of the file or directory involved in the event.
    async fn handle_current_event(
        observer_sender: UnboundedSender<GaladrielEvents>,
        processing_state: &mut ProcessingState,
        config_path: &PathBuf,
        path: &PathBuf,
    ) {
        info!("Handling current event for path: {:?}", path);

        // If the current processing state is awaiting for processing.
        if *processing_state == ProcessingState::Awaiting {
            info!("Processing state is Awaiting. Setting to Running.");
            // Set the processing state to running processing.
            *processing_state = ProcessingState::Running;

            match path {
                // Reset configs if the current changed file is the config file.
                path if path == config_path => {
                    info!("Configuration file modified: {:?}", config_path);

                    if let Err(err) = observer_sender.send(GaladrielEvents::ReloadGaladrielConfigs)
                    {
                        error!(
                            "Error notifying main runtime to reload configurations: {:?}",
                            err
                        );
                    }
                }
                // Verifies if the current path is a file path and if it is not ignored.
                path if path.extension().map(|ext| ext == "nyr").unwrap_or(false) => {
                    info!("Recognized `.nyr` file modification at: {:?}", path);

                    Self::process_nenyr_file(observer_sender, path).await;
                }
                _ => {}
            }

            info!("Resetting processing state to Awaiting.");
            // Reset to awaiting process after the current process is completed.
            *processing_state = ProcessingState::Awaiting;
        }
    }

    async fn process_nenyr_file(observer_sender: UnboundedSender<GaladrielEvents>, path: &PathBuf) {
        if let Err(err) = observer_sender.send(GaladrielEvents::Parse(path.clone())) {
            error!("Something went wrong while sending the current Nenyr path to the main runtime. Error: {:?}", err);
        }
    }
}

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

    debug!(
        "Selected random watch subtitle message: {}",
        selected_message
    );

    selected_message
}

#[cfg(test)]
mod tests {
    use crate::shellscape::alerts::ShellscapeAlerts;

    use super::{BaraddurObserver, GaladrielEvents};
    use std::{path::PathBuf, sync::Arc};

    use chrono::Local;
    use ignore::overrides;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_new() {
        let observer = BaraddurObserver::new(PathBuf::from("."), 250);

        assert!(observer.observer_sender.clone().is_closed() == false);
        assert!(observer.observer_receiver.is_closed() == false);
    }

    #[tokio::test]
    async fn test_next_receives_event() {
        let mut observer = BaraddurObserver::new(PathBuf::from("."), 250);
        let sender = observer.observer_sender.clone();
        let notification = ShellscapeAlerts::create_information(Local::now(), "Test message");

        // Send an event from the sender side.
        let expected_event = GaladrielEvents::Notify(notification);
        sender.send(expected_event.clone()).unwrap();

        // Use the `next()` method to receive the event.
        let received_event = observer.next().await.unwrap();
        assert_eq!(received_event, expected_event);
    }

    #[tokio::test]
    async fn test_start_sends_initial_message() {
        let working_dir = PathBuf::from(".");
        let from_millis = 1000;
        let mut observer = BaraddurObserver::new(working_dir, from_millis);

        // Start observer with arbitrary values for matcher, directory, and duration.
        let matcher = overrides::OverrideBuilder::new(".").build().unwrap(); // Assume a default or mock implementation

        // Start the observer in an async task and confirm it doesn't panic.
        let _handle = observer.start(Arc::new(RwLock::new(matcher)));

        // Receive the initial starting message.
        let result = observer.next().await;
        assert!(matches!(result, Ok(GaladrielEvents::Notify(_))));
    }

    #[tokio::test]
    async fn test_async_watch_with_send_error() {
        let matcher = overrides::OverrideBuilder::new(".").build().unwrap();
        // Simulate a scenario where sending the observer event fails
        let matcher = Arc::new(RwLock::new(matcher));
        let (sender, _) = tokio::sync::mpsc::unbounded_channel::<GaladrielEvents>(); // Will not be used
        let working_dir = PathBuf::from(".");
        let debounce_interval = 100u64;

        let result =
            BaraddurObserver::async_watch(matcher, sender, working_dir, debounce_interval).await;

        // Ensure that an error is returned due to the observer sender failure
        assert!(result.is_err());
    }
}
