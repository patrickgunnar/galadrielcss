use chrono::{DateTime, Local};
use tokio::sync::broadcast;

use crate::{
    error::GaladrielError, events::GaladrielAlerts,
    utils::send_palantir_notification::send_palantir_notification,
};

pub fn send_palantir_error_notification(
    error: GaladrielError,
    starting_time: DateTime<Local>,
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
) {
    tracing::error!("Error occurred during operation. Details: {:?}", error);

    // Create an error notification with the starting timestamp and error details.
    let notification = GaladrielAlerts::create_galadriel_error(starting_time, error);

    // Send the error notification.
    send_palantir_notification(notification, palantir_sender.clone());
}
