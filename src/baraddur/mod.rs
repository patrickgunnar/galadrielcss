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

use crate::GaladrielResult;

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

#[derive(Debug)]
pub struct BaraddurObserver {
    observer_sender: UnboundedSender<ObserverEvents>,
    observer_receiver: UnboundedReceiver<ObserverEvents>,
}

impl BaraddurObserver {
    pub fn new() -> Self {
        let (observer_sender, observer_receiver) = mpsc::unbounded_channel();

        Self {
            observer_sender,
            observer_receiver,
        }
    }

    pub async fn next(&mut self) -> GaladrielResult<ObserverEvents> {
        self.observer_receiver.recv().await.ok_or_else(|| {
            tracing::error!("Failed to receive Barad-dûr observer event: Channel closed unexpectedly or an IO error occurred");

            Box::<dyn std::error::Error>::from("Error while receiving response from Barad-dûr observer sender: No response received.")
        })
    }

    pub fn start(
        &self,
        matcher: overrides::Override,
        working_dir: PathBuf,
        from_millis: u64,
    ) -> JoinHandle<()> {
        let observer_sender = self.observer_sender.clone();

        tokio::spawn(async move {
            if let Err(err) =
                Self::async_watch(matcher, observer_sender.clone(), working_dir, from_millis).await
            {
                let notification = ObserverEvents::AsyncDebouncerError(err.to_string());
                if let Err(err) = observer_sender.send(notification) {
                    tracing::error!("{:?}", err);
                }
            }
        })
    }

    fn async_debouncer(
        observer_sender: UnboundedSender<ObserverEvents>,
        from_millis: u64,
    ) -> GaladrielResult<(
        Debouncer<RecommendedWatcher, RecommendedCache>,
        mpsc::Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>,
    )> {
        let (debouncer_sender, debouncer_receiver) = mpsc::channel(1);
        let handle_view = runtime::Handle::current();

        let debouncer = new_debouncer(Duration::from_millis(from_millis), None, move |response| {
            let debouncer_sender = debouncer_sender.clone();
            let observer_sender = observer_sender.clone();

            handle_view.spawn(async move {
                if let Err(err) = debouncer_sender.send(response).await {
                    tracing::error!("{:?}", err);

                    let notification = ObserverEvents::AsyncDebouncerError(err.to_string());
                    if let Err(err) = observer_sender.send(notification) {
                        tracing::error!("{:?}", err);
                    }
                }
            });
        })?;

        Ok((debouncer, debouncer_receiver))
    }

    async fn async_watch(
        matcher: overrides::Override,
        observer_sender: UnboundedSender<ObserverEvents>,
        working_dir: PathBuf,
        from_millis: u64,
    ) -> GaladrielResult<()> {
        let (mut debouncer, mut debouncer_receiver) =
            Self::async_debouncer(observer_sender.clone(), from_millis)?;

        observer_sender.send(ObserverEvents::StartingMessage(random_watch_message()))?;
        debouncer.watch(working_dir.clone(), notify::RecursiveMode::Recursive)?;

        let config_path = working_dir.join("galadriel.config.json");
        let mut processing_state = ProcessingState::Awaiting;

        let result = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = observer_sender.closed() => {
                        break;
                    }
                    result = debouncer_receiver.recv() => {
                        match result {
                            Some(Ok(debounced_events)) => {
                                Self::process_buffered_events(
                                    observer_sender.clone(),
                                    &mut processing_state,
                                    &matcher,
                                    &config_path,
                                    debounced_events[0].clone()
                                ).await;
                            }

                            Some(Err(err)) => {
                                tracing::error!("{:?}", err);

                                let notification = ObserverEvents::AsyncDebouncerError(err[0].to_string());
                                if let Err(err) = observer_sender.send(notification) {
                                    tracing::error!("{:?}", err);
                                }

                                break;
                            }
                            None => {}
                        }
                    }
                }
            }
        })
        .await;

        match result {
            Ok(()) => Ok(()),
            Err(err) => Err(Box::<dyn std::error::Error>::from(err.to_string())),
        }
    }

    async fn process_buffered_events(
        observer_sender: UnboundedSender<ObserverEvents>,
        processing_state: &mut ProcessingState,
        matcher: &overrides::Override,
        config_path: &PathBuf,
        debounced_event: DebouncedEvent,
    ) {
        match debounced_event.kind {
            EventKind::Modify(_modified) => {
                let path = debounced_event.paths[0].clone();

                Self::handle_current_event(
                    observer_sender.clone(),
                    processing_state,
                    matcher,
                    config_path,
                    &path,
                )
                .await;
            }
            EventKind::Remove(_) => {}
            _ => {}
        }
    }

    async fn handle_current_event(
        observer_sender: UnboundedSender<ObserverEvents>,
        processing_state: &mut ProcessingState,
        matcher: &overrides::Override,
        config_path: &PathBuf,
        path: &PathBuf,
    ) {
        if *processing_state == ProcessingState::Awaiting {
            *processing_state = ProcessingState::Running;

            if path == config_path {
                let notification = ObserverEvents::ModifiedPath(path.to_string_lossy().to_string());
                if let Err(err) = observer_sender.send(notification) {
                    tracing::error!("{:?}", err);
                }
            } else if path.is_file() && !matcher.matched(&path, false).is_ignore() {
                match path.extension() {
                    Some(ext) if ext == "nyr" => {
                        let notification =
                            ObserverEvents::ModifiedPath(path.to_string_lossy().to_string());

                        if let Err(err) = observer_sender.send(notification) {
                            tracing::error!("{:?}", err);
                        }
                    }
                    _ => {}
                }
            }

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

    String::from(messages[idx])
}
