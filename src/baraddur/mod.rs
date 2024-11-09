use std::{path::PathBuf, time::Duration};

use ignore::overrides;
use notify::{EventKind, RecommendedWatcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, RecommendedCache};
use rand::Rng;
use tokio::{
    runtime,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

#[derive(Clone, PartialEq, Debug)]
pub enum ObserverEvents {
    AsyncDebouncerError(String),
    StartingMessage(String),
    ModifiedPath(String),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ProcessingState {
    Running,
    Awaiting,
}

/// A struct to observe changes in a directory using an event-driven approach.
#[derive(Debug)]
pub struct BaraddurObserver {
    // Sender for observer events
    observer_sender: UnboundedSender<ObserverEvents>,
    // Receiver for observer events
    observer_receiver: UnboundedReceiver<ObserverEvents>,
}

impl BaraddurObserver {
    /// Creates a new `BaraddurObserver` instance with a fresh sender and receiver for events.
    ///
    /// # Returns
    /// A new `BaraddurObserver` instance.
    pub fn new() -> Self {
        let (observer_sender, observer_receiver) = mpsc::unbounded_channel();

        info!("BaraddurObserver instance created with new event sender and receiver.");

        Self {
            observer_sender,
            observer_receiver,
        }
    }

    /// Awaits and retrieves the next event from the observer.
    ///
    /// # Returns
    /// A `GaladrielResult` wrapping the next event (`ObserverEvents`).
    ///
    /// # Errors
    /// Returns an error if the channel is closed or an IO error occurs.
    pub async fn next(&mut self) -> GaladrielResult<ObserverEvents> {
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
    /// * `matcher`: A matcher object used to determine whether certain files should be processed.
    /// * `working_dir`: The directory to watch for file changes.
    /// * `from_millis`: The debounce interval in milliseconds.
    ///
    /// # Returns
    /// A `JoinHandle` for the spawned async task.
    pub fn start(
        &self,
        matcher: overrides::Override,
        working_dir: PathBuf,
        from_millis: u64,
    ) -> JoinHandle<()> {
        let observer_sender = self.observer_sender.clone();

        // Spawn an asynchronous task to start the directory watch
        tokio::spawn(async move {
            info!(
                "Starting Barad-dûr observer with working directory {:?}",
                working_dir
            );

            if let Err(err) =
                Self::async_watch(matcher, observer_sender.clone(), working_dir, from_millis).await
            {
                error!("Failed to start async watch: {:?}", err);

                let notification = ObserverEvents::AsyncDebouncerError(err.to_string());
                if let Err(err) = observer_sender.send(notification) {
                    error!("Failed to send async debouncer error: {:?}", err);
                }
            }
        })
    }

    /// Initializes a debouncer for handling file system events with a specified delay.
    ///
    /// # Arguments
    /// * `observer_sender`: The sender for observer events.
    /// * `from_millis`: The debounce interval in milliseconds.
    ///
    /// # Returns
    /// A `GaladrielResult` wrapping a tuple of the debouncer and receiver.
    fn async_debouncer(
        observer_sender: UnboundedSender<ObserverEvents>,
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
                    error!("Error sending response to debouncer channel: {:?}", err);

                    let notification = ObserverEvents::AsyncDebouncerError(err.to_string());
                    if let Err(err) = observer_sender.send(notification) {
                        error!("Failed to send error notification to observer: {:?}", err);
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
    /// * `matcher`: A matcher object for filtering files.
    /// * `observer_sender`: The sender for observer events.
    /// * `working_dir`: The directory to watch.
    /// * `from_millis`: The debounce interval in milliseconds.
    ///
    /// # Returns
    /// A `GaladrielResult` wrapping the success or failure of the watch operation.
    async fn async_watch(
        matcher: overrides::Override,
        observer_sender: UnboundedSender<ObserverEvents>,
        working_dir: PathBuf,
        from_millis: u64,
    ) -> GaladrielResult<()> {
        let (mut debouncer, mut debouncer_receiver) =
            Self::async_debouncer(observer_sender.clone(), from_millis)?;

        info!("Starting to watch directory {:?}", working_dir);
        observer_sender
            .send(ObserverEvents::StartingMessage(random_watch_message()))
            .map_err(|err| {
                GaladrielError::raise_general_observer_error(
                    ErrorKind::NotificationSendError,
                    &err.to_string(),
                    ErrorAction::Restart,
                )
            })?;

        // Set up the recursive watch on the directory
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
                    _ = observer_sender.closed() => {
                        warn!("Observer sender channel closed, stopping watch loop.");
                        break;
                    }
                    result = debouncer_receiver.recv() => {
                        match result {
                            Some(Ok(debounced_events)) => {
                                let debounced_event = debounced_events[0].clone();
                                let path = debounced_event.paths[0].clone();

                                if path.is_file() && !matcher.matched(&path, false).is_ignore() {
                                    info!("Received debounced event: {:?}", debounced_events);

                                    Self::process_buffered_events(
                                        observer_sender.clone(),
                                        &mut processing_state,
                                        &config_path,
                                        debounced_event,
                                        path
                                    ).await;
                                }
                            }

                            Some(Err(err)) => {
                                error!("Error in debounced events receiver: {:?}", err);

                                let notification = ObserverEvents::AsyncDebouncerError(err[0].to_string());
                                if let Err(err) = observer_sender.send(notification) {
                                    error!("Failed to send async debouncer error notification: {:?}", err);
                                }

                                break;
                            }
                            None => {}
                        }
                    }
                }
            }
        })
        .await.map_err(|err| {
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

    /// Processes buffered file system events by handling modifications and removals.
    ///
    /// # Arguments
    /// * `observer_sender`: The sender for observer events.
    /// * `processing_state`: The current processing state.
    /// * `matcher`: The matcher for filtering files.
    /// * `config_path`: The path to the config file.
    /// * `debounced_event`: The debounced event to process.
    /// * `path`: The path of the file or directory involved in the event.
    async fn process_buffered_events(
        observer_sender: UnboundedSender<ObserverEvents>,
        processing_state: &mut ProcessingState,
        config_path: &PathBuf,
        debounced_event: DebouncedEvent,
        path: PathBuf,
    ) {
        //info!("Processing buffered event: {:?}", debounced_event.kind);

        match debounced_event.kind {
            EventKind::Modify(_modified) => {
                info!("Detected modification event at path: {:?}", path);

                Self::handle_current_event(
                    observer_sender.clone(),
                    processing_state,
                    config_path,
                    &path,
                )
                .await;
            }
            EventKind::Remove(_) => {
                warn!("Detected removal event at path: {:?}", path);
            }
            _ => {}
        }
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
        observer_sender: UnboundedSender<ObserverEvents>,
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

                    let notification =
                        ObserverEvents::ModifiedPath(path.to_string_lossy().to_string());
                    if let Err(err) = observer_sender.send(notification) {
                        error!("Failed to send notification: {:?}", err);
                    }
                }
                // Verifies if the current path is a file path and if it is not ignored.
                path if path.extension().map(|ext| ext == "nyr").unwrap_or(false) => {
                    info!("Recognized .nyr file modification at: {:?}", path);

                    let notification =
                        ObserverEvents::ModifiedPath(path.to_string_lossy().to_string());

                    if let Err(err) = observer_sender.send(notification) {
                        error!(
                            "Failed to send .nyr file modification notification: {:?}",
                            err
                        );
                    }
                }
                _ => {}
            }

            info!("Resetting processing state to Awaiting.");
            // Reset to awaiting process after the current process is completed.
            *processing_state = ProcessingState::Awaiting;
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
    use super::{BaraddurObserver, ObserverEvents};
    use std::path::PathBuf;

    use ignore::overrides;

    #[tokio::test]
    async fn test_new() {
        let observer = BaraddurObserver::new();

        assert!(observer.observer_sender.clone().is_closed() == false);
        assert!(observer.observer_receiver.is_closed() == false);
    }

    #[tokio::test]
    async fn test_next_receives_event() {
        let mut observer = BaraddurObserver::new();
        let sender = observer.observer_sender.clone();

        // Send an event from the sender side.
        let expected_event = ObserverEvents::StartingMessage("Testing event".to_string());
        sender.send(expected_event.clone()).unwrap();

        // Use the `next()` method to receive the event.
        let received_event = observer.next().await.unwrap();
        assert_eq!(received_event, expected_event);
    }

    #[tokio::test]
    async fn test_start_sends_initial_message() {
        let mut observer = BaraddurObserver::new();

        // Start observer with arbitrary values for matcher, directory, and duration.
        let matcher = overrides::OverrideBuilder::new(".").build().unwrap(); // Assume a default or mock implementation
        let working_dir = PathBuf::from(".");
        let from_millis = 1000;

        // Start the observer in an async task and confirm it doesn't panic.
        let _handle = observer.start(matcher, working_dir, from_millis);

        // Receive the initial starting message.
        let result = observer.next().await;
        assert!(matches!(result, Ok(ObserverEvents::StartingMessage(_))));
    }
}
