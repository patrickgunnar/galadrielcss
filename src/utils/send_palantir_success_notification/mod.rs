use chrono::{DateTime, Local};
use tokio::sync::broadcast;

use crate::{
    events::GaladrielAlerts, utils::send_palantir_notification::send_palantir_notification,
};

pub fn send_palantir_success_notification(
    message: &str,
    starting_time: DateTime<Local>,
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
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
    send_palantir_notification(notification, palantir_sender.clone());
}
