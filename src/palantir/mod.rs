use chrono::Local;
use tokio::{sync, task::JoinHandle};

use crate::{
    asts::PALANTIR_ALERTS,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
};

#[derive(Debug)]
pub struct Palantir {
    palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
}

impl Palantir {
    pub fn new() -> Self {
        let (palantir_sender, _) = sync::broadcast::channel(100);

        Self { palantir_sender }
    }

    pub fn get_palantir_sender(&self) -> sync::broadcast::Sender<GaladrielAlerts> {
        self.palantir_sender.clone()
    }

    pub fn start_alert_watcher(&self) -> JoinHandle<()> {
        let palantir_sender = self.palantir_sender.clone();

        tokio::spawn(async move {
            let mut palantir_receiver = palantir_sender.subscribe();

            loop {
                tokio::select! {
                    palantir_event = palantir_receiver.recv() => {
                        match palantir_event {
                            Ok(notification) => {
                                Self::push_top(notification);
                            },
                            Err(sync::broadcast::error::RecvError::Closed) => {
                                break;
                            }
                            Err(err) => {
                                let error = GaladrielError::raise_general_other_error(
                                    ErrorKind::NotificationReceiveError,
                                    &err.to_string(),
                                    ErrorAction::Notify
                                );

                                let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

                                Self::push_top(notification);
                            }
                        }
                    }
                }
            }
        })
    }

    fn push_top(notification: GaladrielAlerts) {
        match PALANTIR_ALERTS.get_mut("alerts") {
            Some(ref mut palantir_alerts) => {
                palantir_alerts.value_mut().insert(0, notification);

                if palantir_alerts.value().len() > 100 {
                    palantir_alerts.value_mut().pop();
                }
            }
            None => {
                tracing::error!("Failed to find 'alerts' in the PALANTIR_ALERTS map.");
            }
        }
    }

    pub fn send_alert(&self, notification: GaladrielAlerts) {
        let sender = self.get_palantir_sender();

        if let Err(err) = sender.send(notification) {
            tracing::error!("{:?}", err);
        }
    }
}
