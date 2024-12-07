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
    configatron::{get_auto_naming, load_galadriel_configs, reconstruct_exclude_matcher},
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::{GaladrielAlerts, GaladrielEvents},
    formera::Formera,
    gatekeeper::remove_path_from_gatekeeper,
    intaker::remove_context_from_intaker::remove_context_from_intaker,
    GaladrielResult,
};

pub mod events;

#[derive(Debug)]
pub struct Baraddur {
    baraddur_sender: mpsc::UnboundedSender<GaladrielEvents>,
    baraddur_receiver: mpsc::UnboundedReceiver<GaladrielEvents>,
    palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    working_dir: PathBuf,
    from_millis: u64,
}

impl Baraddur {
    pub fn new(
        from_millis: u64,
        working_dir: PathBuf,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) -> Self {
        let (baraddur_sender, baraddur_receiver) = mpsc::unbounded_channel();

        Self {
            baraddur_sender,
            baraddur_receiver,
            palantir_sender,
            working_dir,
            from_millis,
        }
    }

    pub async fn next(&mut self) -> GaladrielResult<GaladrielEvents> {
        self.baraddur_receiver.recv().await.ok_or_else(|| {
            GaladrielError::raise_general_observer_error(
                ErrorKind::ObserverEventReceiveFailed,
                "Error while receiving response from Barad-dûr observer sender: No response received.",
                ErrorAction::Notify
            )
        })
    }

    pub fn async_debouncer(
        &self,
        matcher: Arc<RwLock<overrides::Override>>,
    ) -> GaladrielResult<(
        Debouncer<RecommendedWatcher, RecommendedCache>,
        sync::broadcast::Sender<Vec<BaraddurEventProcessor>>,
    )> {
        let (debouncer_sender, _) = sync::broadcast::channel(100);
        let handle_view = runtime::Handle::current();

        let palantir_sender = self.palantir_sender.clone();
        let debouncer_tx = debouncer_sender.clone();
        let working_dir = self.working_dir.clone();

        let debouncer = new_debouncer(
            tokio::time::Duration::from_millis(self.from_millis),
            None,
            move |event_result| {
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
            GaladrielError::raise_critical_observer_error(
                ErrorKind::AsyncDebouncerCreationFailed,
                &err.to_string(),
                ErrorAction::Restart,
            )
        })?;

        Ok((debouncer, debouncer_sender))
    }

    pub async fn match_async_debouncer_result(
        configuration_path: &PathBuf,
        matcher: Arc<RwLock<overrides::Override>>,
        event_result: Result<Vec<DebouncedEvent>, Vec<notify::Error>>,
        debouncer_sender: sync::broadcast::Sender<Vec<BaraddurEventProcessor>>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        match event_result {
            Ok(debounced_events) => {
                let mut processing_events: Vec<BaraddurEventProcessor> = Vec::new();
                let mut rename_state = BaraddurRenameEventState::None;
                let matcher = matcher.read().await;

                Self::process_debounced_events(
                    configuration_path,
                    &matcher,
                    debounced_events,
                    &mut rename_state,
                    &mut processing_events,
                );

                if !processing_events.is_empty() {
                    if let Err(err) = debouncer_sender.send(processing_events) {
                        tracing::error!("Failed to send debounced events: {:?}", err);
                    }
                }
            }
            Err(errs) => {
                for err in errs {
                    let error = GaladrielError::raise_general_observer_error(
                        ErrorKind::DebouncedEventError,
                        &err.to_string(),
                        ErrorAction::Notify,
                    );

                    let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

                    Self::send_palantir_notification(notification, palantir_sender.clone());
                }
            }
        }
    }

    fn process_debounced_events(
        configuration_path: &PathBuf,
        matcher: &overrides::Override,
        debounced_events: Vec<DebouncedEvent>,
        rename_state: &mut BaraddurRenameEventState,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        debounced_events.iter().for_each(|debounced_event| {
            debounced_event.paths.iter().for_each(|path| {
                if Self::is_configuration_event(path, configuration_path) {
                    Self::process_configuration_event(debounced_event.kind, processing_events);
                } else if Self::is_nenyr_event(path, matcher) {
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

    fn process_configuration_event(
        debounced_event_kind: EventKind,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        if let BaraddurEventProcessorKind::None =
            Self::process_debounced_events_for_configs(debounced_event_kind)
        {
            return;
        }

        let reload_configs = BaraddurEventProcessor::ReloadGaladrielConfigs;
        Self::add_event_if_not_exists(reload_configs, processing_events);
    }

    fn process_nenyr_event(
        path: &PathBuf,
        debounced_event_kind: EventKind,
        rename_state: &mut BaraddurRenameEventState,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        let event_kind =
            Self::process_debounced_events_for_nenyr(debounced_event_kind, rename_state);

        if let BaraddurEventProcessorKind::None = event_kind {
            return;
        }

        let processing_event = BaraddurEventProcessor::ProcessEvent {
            kind: event_kind,
            path: path.to_owned(),
        };

        Self::add_event_if_not_exists(processing_event, processing_events);
    }

    fn add_event_if_not_exists(
        processing_event: BaraddurEventProcessor,
        processing_events: &mut Vec<BaraddurEventProcessor>,
    ) {
        if !processing_events.contains(&processing_event) {
            processing_events.push(processing_event);
        }
    }

    fn is_configuration_event(path: &PathBuf, configuration_path: &PathBuf) -> bool {
        path == configuration_path
    }

    fn is_nenyr_event(path: &PathBuf, matcher: &overrides::Override) -> bool {
        !matcher.matched(path, false).is_ignore()
            && path.extension().map(|ext| ext == "nyr").unwrap_or(false)
    }

    fn process_debounced_events_for_configs(
        debounced_event_kind: EventKind,
    ) -> BaraddurEventProcessorKind {
        match debounced_event_kind {
            EventKind::Modify(modified_kind) => match modified_kind {
                ModifyKind::Any | ModifyKind::Data(_) => BaraddurEventProcessorKind::Modify,
                _ => BaraddurEventProcessorKind::None,
            },
            EventKind::Create(CreateKind::File) => BaraddurEventProcessorKind::Modify,
            _ => BaraddurEventProcessorKind::None,
        }
    }

    fn process_debounced_events_for_nenyr(
        debounced_event_kind: EventKind,
        rename_state: &mut BaraddurRenameEventState,
    ) -> BaraddurEventProcessorKind {
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

    pub fn watch(
        &self,
        matcher: Arc<RwLock<overrides::Override>>,
        debouncer: &mut Debouncer<RecommendedWatcher, RecommendedCache>,
        debouncer_sender: sync::broadcast::Sender<Vec<BaraddurEventProcessor>>,
    ) -> JoinHandle<()> {
        let baraddur_sender = self.baraddur_sender.clone();
        let palantir_sender = self.palantir_sender.clone();
        let working_dir = self.working_dir.clone();

        let mut palantir_receiver = palantir_sender.subscribe();
        let mut debouncer_receiver = debouncer_sender.subscribe();
        let mut nenyr_parser = NenyrParser::new();

        if let Err(err) = debouncer.watch(working_dir.clone(), notify::RecursiveMode::Recursive) {
            let error = GaladrielError::raise_critical_observer_error(
                ErrorKind::DebouncerWatchFailed,
                &err.to_string(),
                ErrorAction::Restart,
            );

            if let Err(err) = baraddur_sender.send(GaladrielEvents::Error(error)) {
                tracing::error!(
                    "Failed to send error notification to main runtime: {:?}",
                    err
                );
            }
        }

        let starting_time = Local::now();
        let notification = GaladrielAlerts::create_success(
            starting_time,
            starting_time,
            starting_time - starting_time,
            &Self::random_watch_message(),
        );

        Self::send_palantir_notification(notification, palantir_sender.clone());

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = baraddur_sender.closed() => {
                        break;
                    }
                    Err(sync::broadcast::error::RecvError::Closed) = palantir_receiver.recv() => {
                        break;
                    }
                    debounced_event_result = debouncer_receiver.recv() => {
                        Self::match_debounced_result(
                            &working_dir,
                            &mut nenyr_parser,
                            Arc::clone(&matcher),
                            palantir_sender.clone(),
                            &debounced_event_result
                        )
                        .await;
                    }
                }
            }
        })
    }

    async fn match_debounced_result(
        working_dir: &PathBuf,
        nenyr_parser: &mut NenyrParser,
        matcher: Arc<RwLock<overrides::Override>>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
        debounced_event_result: &Result<
            Vec<BaraddurEventProcessor>,
            sync::broadcast::error::RecvError,
        >,
    ) {
        match debounced_event_result {
            Ok(debounced_events) => {
                for debounced_event in debounced_events {
                    match debounced_event {
                        BaraddurEventProcessor::ReloadGaladrielConfigs => {
                            Self::reload_galadriel_configs(
                                working_dir,
                                Arc::clone(&matcher),
                                palantir_sender.clone(),
                            )
                            .await;
                        }
                        BaraddurEventProcessor::ProcessEvent { kind, path } => {
                            Self::match_processing_event_kind(
                                path,
                                kind,
                                nenyr_parser,
                                palantir_sender.clone(),
                            )
                            .await;
                        }
                    }
                }
            }
            Err(err) => {
                let error = GaladrielError::raise_general_observer_error(
                    ErrorKind::Other,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

                Self::send_palantir_notification(notification, palantir_sender.clone());
            }
        }
    }

    async fn reload_galadriel_configs(
        working_dir: &PathBuf,
        matcher: Arc<RwLock<overrides::Override>>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        let starting_time = Local::now();

        match load_galadriel_configs(working_dir).await {
            Ok(()) => {
                match reconstruct_exclude_matcher(working_dir, matcher).await {
                    Ok(notification) => {
                        Self::send_palantir_notification(notification, palantir_sender.clone());
                    }
                    Err(error) => {
                        Self::send_palantir_error_notification(
                            error,
                            starting_time,
                            palantir_sender.clone(),
                        );
                    }
                }

                Self::send_palantir_success_notification(
                    "Galadriel CSS configurations updated successfully. System is now operating with the latest configuration.",
                    starting_time,
                    palantir_sender.clone()
                );
            }
            Err(error) => {
                Self::send_palantir_error_notification(
                    error,
                    starting_time,
                    palantir_sender.clone(),
                );
            }
        }
    }

    async fn match_processing_event_kind(
        current_path: &PathBuf,
        processing_event_kind: &BaraddurEventProcessorKind,
        nenyr_parser: &mut NenyrParser,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        match processing_event_kind {
            BaraddurEventProcessorKind::Modify => {
                Self::process_nenyr_file(
                    current_path.to_owned(),
                    nenyr_parser,
                    palantir_sender.clone(),
                )
                .await;
            }
            BaraddurEventProcessorKind::Remove => {
                let file_path = current_path.to_string_lossy().to_string();

                remove_path_from_gatekeeper(&file_path);
                remove_context_from_intaker(&file_path);
            }
            _ => {}
        }
    }

    async fn process_nenyr_file(
        current_path: PathBuf,
        nenyr_parser: &mut NenyrParser,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        let stringified_path = current_path.to_string_lossy().to_string();
        let is_auto_naming = get_auto_naming();
        let starting_time = Local::now();

        let notification = GaladrielAlerts::create_information(
            starting_time,
            &format!("Initiating parsing of: {:?}", stringified_path),
        );

        Self::send_palantir_notification(notification, palantir_sender.clone());

        let mut formera = Formera::new(current_path, is_auto_naming, palantir_sender.clone());

        match formera.start(nenyr_parser).await {
            Ok(()) => {
                Self::send_palantir_success_notification(
                    &format!("Successfully parsed Nenyr file: {:?}", stringified_path),
                    starting_time,
                    palantir_sender.clone(),
                );
            }
            Err(GaladrielError::NenyrError { start_time, error }) => {
                let notification =
                    GaladrielAlerts::create_nenyr_error(start_time.to_owned(), error.to_owned());

                Self::send_palantir_notification(notification, palantir_sender.clone());
            }
            Err(error) => {
                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );
            }
        }
    }

    fn send_palantir_success_notification(
        message: &str,
        starting_time: DateTime<Local>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        let ending_time = Local::now();
        let duration = ending_time - starting_time;
        let notification =
            GaladrielAlerts::create_success(starting_time, ending_time, duration, message);

        Self::send_palantir_notification(notification, palantir_sender.clone());
    }

    fn send_palantir_error_notification(
        error: GaladrielError,
        starting_time: DateTime<Local>,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        let notification = GaladrielAlerts::create_galadriel_error(starting_time, error);
        Self::send_palantir_notification(notification, palantir_sender.clone());
    }

    fn send_palantir_notification(
        notification: GaladrielAlerts,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) {
        if let Err(err) = palantir_sender.send(notification) {
            tracing::error!("Failed to send alert: {:?}", err);
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

        tracing::debug!(
            "Selected random watch subtitle message: {}",
            selected_message
        );

        selected_message
    }
}
