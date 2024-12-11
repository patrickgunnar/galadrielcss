use tokio::sync::broadcast;

use crate::events::GaladrielAlerts;

pub fn send_palantir_notification(
    notification: GaladrielAlerts,
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
) {
    // Attempt to send the notification. Log an error if it fails.
    if let Err(err) = palantir_sender.send(notification) {
        tracing::error!("Failed to send alert: {:?}", err);
    }
}
