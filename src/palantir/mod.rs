use chrono::Local;
use tokio::{sync, task::JoinHandle};

use crate::{
    asts::PALANTIR_ALERTS,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
};

/// Represents a communication channel for broadcasting and managing `GaladrielAlerts`.
///
/// The `Palantir` struct provides functionality for creating, sending, and managing
/// alerts, as well as maintaining a local cache of notifications.
#[derive(Debug)]
pub struct Palantir {
    /// The sender for broadcasting `GaladrielAlerts` notifications.
    palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
}

#[allow(dead_code)]
impl Palantir {
    /// Creates a new `Palantir` instance with a broadcast channel of capacity 100.
    ///
    /// # Returns
    /// A new instance of `Palantir`.
    pub fn new() -> Self {
        // Create a new broadcast channel for sending alerts, ignoring the receiver here.
        let (palantir_sender, _) = sync::broadcast::channel(100);

        tracing::info!("Palantir instance created with a broadcast channel of capacity 100.");

        // Return a new `Palantir` instance with the created sender.
        Self { palantir_sender }
    }

    /// Retrieves a clone of the broadcast sender for `GaladrielAlerts`.
    ///
    /// # Returns
    /// A cloned instance of the sender for broadcasting alerts.
    pub fn get_palantir_sender(&self) -> sync::broadcast::Sender<GaladrielAlerts> {
        tracing::debug!("Cloning the Palantir sender for alert broadcasting.");

        self.palantir_sender.clone()
    }

    /// Starts the alert watcher, which listens for incoming alerts and processes them.
    ///
    /// This function spawns a Tokio task that subscribes to the broadcast channel and
    /// continuously processes alerts, pushing them to the top of the alert cache.
    ///
    /// # Returns
    /// A `JoinHandle` representing the spawned Tokio task.
    pub fn start_alert_watcher(&self) -> JoinHandle<()> {
        let palantir_sender = self.get_palantir_sender();

        tracing::info!("Starting the alert watcher.");

        // Spawn an asynchronous task to handle incoming alerts.
        tokio::spawn(async move {
            // Subscribe to the broadcast channel.
            let mut palantir_receiver = palantir_sender.subscribe();

            // Continuously listen for incoming alerts.
            loop {
                tokio::select! {
                    // Await the next alert event from the receiver.
                    palantir_event = palantir_receiver.recv() => {
                        match palantir_event {
                            Ok(notification) => {
                                tracing::info!("Received a new alert: {:?}", notification);

                                // Push valid notifications to the top of the cache.
                                Self::push_top(notification);
                            },
                            Err(sync::broadcast::error::RecvError::Closed) => {
                                tracing::info!("Alert watcher channel closed, stopping the task.");

                                // Break the loop if the channel is closed.
                                break;
                            }
                            Err(err) => {
                                tracing::error!("Error receiving alert: {:?}", err);

                                // Handle other errors, converting them to `GaladrielError`.
                                let error = GaladrielError::raise_general_other_error(
                                    ErrorKind::NotificationReceiveError,
                                    &err.to_string(),
                                    ErrorAction::Notify
                                );

                                // Create an alert for the error and push it to the cache.
                                let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

                                Self::push_top(notification);
                            }
                        }
                    }
                }
            }
        })
    }

    /// Pushes a new alert to the top of the `PALANTIR_ALERTS` cache.
    ///
    /// # Parameters
    /// - `notification`: The `GaladrielAlerts` notification to push to the cache.
    fn push_top(notification: GaladrielAlerts) {
        tracing::debug!("Attempting to push alert to the top of the cache.");

        // Access the global PALANTIR_ALERTS map and retrieve the "alerts" entry.
        match PALANTIR_ALERTS.get_mut("alerts") {
            Some(ref mut palantir_alerts) => {
                // Insert the notification at the top of the list.
                palantir_alerts.value_mut().insert(0, notification);

                tracing::info!("Alert pushed to the top of the cache.");

                // Ensure the cache does not exceed 100 entries.
                if palantir_alerts.value().len() > 100 {
                    tracing::info!("Cache exceeded 100 alerts, removing the oldest alert.");
                    palantir_alerts.value_mut().pop();
                }
            }
            None => {
                tracing::error!("Failed to find 'alerts' in the PALANTIR_ALERTS map.");
            }
        }
    }

    /// Sends an alert to the broadcast channel.
    ///
    /// # Parameters
    /// - `notification`: The `GaladrielAlerts` notification to send.
    pub fn send_alert(&self, notification: GaladrielAlerts) {
        tracing::debug!("Attempting to send alert: {:?}", notification);

        // Retrieve a sender for broadcasting the alert.
        let sender = self.get_palantir_sender();

        // Attempt to send the alert, logging any errors that occur.
        if let Err(err) = sender.send(notification) {
            tracing::error!("Failed to send alert: {:?}", err);
        }
    }
}
