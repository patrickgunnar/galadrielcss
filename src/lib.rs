use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use asts::{CLASSINATOR, STYLITRON};
use baraddur::BaraddurObserver;
use chrono::{DateTime, Local};
use configatron::{Configatron, ConfigurationJson};
use error::{ErrorAction, ErrorKind, GaladrielError};
use events::GaladrielEvents;
use formera::Formera;
use ignore::overrides;
use kickstartor::Kickstartor;
use lothlorien::LothlorienPipeline;
use nenyr::NenyrParser;
use shellscape::{
    alerts::ShellscapeAlerts, app::ShellscapeApp, commands::ShellscapeCommands, Shellscape,
};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    net::TcpListener,
    sync::{
        mpsc::{self, UnboundedSender},
        RwLock,
    },
};
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;

mod asts;
mod baraddur;
mod configatron;
mod crealion;
pub mod error;
mod events;
mod formera;
mod kickstartor;
mod lothlorien;
mod shellscape;
mod types;
mod utils;

#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielRuntimeKind {
    Development,
    Build,
    Update,
}

pub type GaladrielResult<T> = Result<T, GaladrielError>;

#[derive(Clone, PartialEq, Debug)]
pub struct GaladrielRuntime {
    runtime_mode: GaladrielRuntimeKind,
    working_dir: PathBuf,
    configatron: Configatron,
}

impl GaladrielRuntime {
    pub fn new(runtime_mode: GaladrielRuntimeKind, working_dir: PathBuf) -> Self {
        Self {
            runtime_mode,
            working_dir,
            configatron: Configatron::new(
                vec![],
                true,
                true,
                true,
                "0".to_string(),
                "0.0.0".to_string(),
            ),
        }
    }

    pub async fn run(&mut self) -> GaladrielResult<()> {
        match self.runtime_mode {
            GaladrielRuntimeKind::Development => self.start_development_mode().await,
            GaladrielRuntimeKind::Build => self.start_build_mode().await,
            GaladrielRuntimeKind::Update => Ok(()),
        }
    }

    async fn start_development_mode(&mut self) -> GaladrielResult<()> {
        // ===================================================================================================================
        // Creates the development logs subscriber.
        // Generates a subscriber for logging events to a file.
        //
        // This creates a log subscriber that writes logs to a file using the `tracing` library.
        // It sets up a rolling file appender with a log filename generated from `generate_log_filename`.
        // The subscriber is configured to log events with a severity level of `TRACE` or higher.
        // ===================================================================================================================
        let file_name = self.generate_log_filename(); // Generate the log filename.
        let file_appender = rolling::never("logs", file_name); // Create a rolling file appender that writes logs to the specified file.
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender); // Set up non-blocking log writing.

        // Build and return the log subscriber.
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE) // Set the maximum log level to TRACE.
            .with_writer(non_blocking) // Use the non-blocking writer.
            .finish(); // Finalize the subscriber configuration.

        // Set logs subscriber.
        tracing::subscriber::set_global_default(subscriber).map_err(|err| {
            tracing::error!("Failed to set log subscriber: {:?}", err.to_string());

            GaladrielError::raise_critical_runtime_error(
                ErrorKind::TracingSubscriberInitializationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        // Configure the development runtime environment.
        self.configure_development_environment().await
    }

    async fn start_build_mode(&mut self) -> GaladrielResult<()> {
        self.load_galadriel_config()?;

        println!("Build process not implemented yet.");

        Ok(())
    }

    async fn configure_development_environment(&mut self) -> GaladrielResult<()> {
        // Load the galadriel configurations.
        self.load_galadriel_config()?;

        let mut kickstartor = Kickstartor::new(
            self.configatron.get_exclude(),
            self.configatron.get_auto_naming(),
        );

        // TODO: Set an initial state for the UI.
        // TODO: Handle the initial parsing error.
        // Processing Nenyr files for initial setup.
        kickstartor.process_nyr_files().await?;

        tracing::info!("Nenyr files processed successfully. Initial styles AST set successfully.");

        // TODO: Pass the initial UI state for the dev runtime.
        // Transition to development runtime.
        self.development_runtime().await.map_err(|err| {
            GaladrielError::raise_critical_runtime_error(
                ErrorKind::Other,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })
    }

    async fn development_runtime(&mut self) -> GaladrielResult<()> {
        let mut nenyr_parser = NenyrParser::new();
        let (alerts_sender, mut alerts_receiver) = mpsc::unbounded_channel::<ShellscapeAlerts>();

        // Initialize the Shellscape terminal UI.
        let mut shellscape = Shellscape::new();
        let mut _shellscape_events = shellscape.create_events(250); // Event handler for Shellscape events
        let mut interface = shellscape.create_interface()?; // Terminal interface setup
        let mut shellscape_app = shellscape.create_app(self.configatron.clone())?; // Application/state setup for Shellscape

        // Initialize the Lothlórien pipeline (WebSocket server for Galadriel CSS).
        let mut pipeline = LothlorienPipeline::new(self.configatron.get_port());
        let pipeline_listener = pipeline.create_listener().await?; // Create WebSocket listener for pipeline
        let running_on_port = self.retrieve_port_from_local_addr(&pipeline_listener)?; // Extract port from the listener's local address
        let _listener_handler = pipeline.create_pipeline(pipeline_listener); // Start the WebSocket pipeline
        let mut _runtime_sender = pipeline.get_runtime_sender(); // Get runtime sender for Lothlórien pipeline

        // Initialize the Barad-dûr file system observer.
        let mut observer = BaraddurObserver::new(self.working_dir.clone(), 250);
        let matcher = self.construct_exclude_matcher()?; // Exclude matcher for file system monitoring
        let atomically_matcher = Arc::new(RwLock::new(matcher));
        let _observer_handler = observer.start(Arc::clone(&atomically_matcher)); // Start observing file changes

        // Set the running port.
        shellscape_app.reset_server_running_on_port(running_on_port);
        // Register the pipeline's server port in temporary storage.
        pipeline.register_server_port_in_temp(running_on_port)?;
        // Start the Shellscape terminal interface rendering.
        interface.invoke()?;

        tracing::info!("Galadriel CSS development runtime initiated.");

        loop {
            // Render the Shellscape terminal interface, handle potential errors.
            if let Err(err) = interface.render(&mut shellscape_app) {
                // TODO: handle the error.

                println!("{:?}", err);
            }

            // TODO: Move the initial parsing operation into here, after the UI, server and observer had stated.
            // TODO: Make the alerts from the initial parsing be reflected in real time with the UI.

            // TODO: Implement comprehensive error handling for potential issues here, designing a robust mechanism to manage different error types effectively.

            tokio::select! {
                alerts_res = alerts_receiver.recv() => {
                    self.match_alerts_events(alerts_res, &mut shellscape_app);
                }
                // Handle events from the Lothlórien pipeline.
                pipeline_res = pipeline.next() => {
                    match pipeline_res {
                        // Handle error events from the Lothlórien pipeline and notify the application.
                        Ok(GaladrielEvents::Error(err)) => {
                            shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(
                                Local::now(),
                                err,
                            ));

                            // TODO: handle the error.
                        }
                        // Handle notification event from the Lothlórien pipeline.
                        Ok(GaladrielEvents::Notify(notification)) => {
                            shellscape_app.add_alert(notification);
                        }
                        // Handle errors from the Lothlórien pipeline and notify the application.
                        Err(err) => {
                            shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(
                                Local::now(),
                                err,
                            ));

                            // TODO: handle the error.
                        }
                        _ => {}
                    }
                }
                // Handle events from the Baraddur observer (file system).
                baraddur_res = observer.next() => {
                    match baraddur_res {
                        // Handle asynchronous debouncer errors from the observer and notify the application.
                        Ok(GaladrielEvents::Error(err)) => {
                            shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(
                                Local::now(),
                                err,
                            ));

                            // TODO: handle the error.
                        }
                        // Handle other events from the Barad-dûr observer by matching them to corresponding actions.
                        Ok(event) => {
                            self.match_observer_events(
                                event,
                                &mut shellscape_app,
                                Arc::clone(&atomically_matcher),
                                alerts_sender.clone(),
                                &mut nenyr_parser,
                            ).await;
                        }
                        // Handle errors from the Barad-dûr observer and notify the application.
                        Err(err) => {
                            shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(
                                Local::now(),
                                err,
                            ));

                            // TODO: handle the error.
                        }
                    }
                }
                // Handle events from the Shellscape terminal interface.
                shellscape_res = shellscape.next() => {
                    match shellscape_res {
                        // Handle a valid event from the Shellscape terminal interface.
                        Ok(event) => {
                            // Match the event to its corresponding Shellscape command.
                            match shellscape.match_shellscape_event(event) {
                                // Exit the loop if the terminate command is received.
                                ShellscapeCommands::Terminate => {
                                    break;
                                }
                                ShellscapeCommands::ScrollNotificationsUp => {
                                    shellscape_app.reset_alerts_scroll_down();
                                }
                                ShellscapeCommands::ScrollNotificationsDown => {
                                    shellscape_app.reset_alerts_scroll_up();
                                }
                                ShellscapeCommands::ScrollDockUp => {
                                    shellscape_app.reset_dock_scroll_down();
                                }
                                ShellscapeCommands::ScrollDockDown => {
                                    shellscape_app.reset_dock_scroll_up();
                                }
                                ShellscapeCommands::ToggleResetStyles => {
                                    self.configatron.toggle_reset_styles();

                                    if let Err(err) = self.replace_configurations_file().await {
                                        shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(Local::now(), err));
                                    }
                                }
                                ShellscapeCommands::ToggleMinifiedStyles => {
                                    self.configatron.toggle_minified_styles();

                                    if let Err(err) = self.replace_configurations_file().await {
                                        shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(Local::now(), err));
                                    }
                                }
                                ShellscapeCommands::ToggleAutoNaming => {
                                    self.configatron.toggle_auto_naming();

                                    if let Err(err) = self.replace_configurations_file().await {
                                        shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(Local::now(), err));
                                    }
                                }
                                ShellscapeCommands::ClearAlertsTable => {
                                    shellscape_app.clear_alerts();
                                }
                                ShellscapeCommands::VewShortcuts => {
                                    shellscape_app.add_shortcut_alert();
                                }
                                ShellscapeCommands::ViewLicense => {
                                    shellscape_app.add_license_alert();
                                }
                                ShellscapeCommands::MakeDonation => {
                                    shellscape_app.add_donation_alert();
                                }
                                ShellscapeCommands::ContributeAsDev => {
                                    shellscape_app.add_contribute_alert();
                                }
                                ShellscapeCommands::AboutAuthor => {
                                    shellscape_app.add_about_author_alert();
                                }
                                ShellscapeCommands::ScrollUp { column, row } => {
                                    // Get the current areas for the dock and alerts
                                    let dock_area = shellscape_app.get_dock_area();
                                    let notify_area = shellscape_app.get_alerts_area();

                                    // Check if the column and row of the event fall within the boundaries of the dock area
                                    // Check if 'column' is within the dock's left and right boundaries
                                    // Check if 'row' is within the dock's top and bottom boundaries
                                    if dock_area.left() <= column && column <= dock_area.right()
                                        && dock_area.top() <= row && row <= dock_area.bottom() {
                                        // If the event is within the dock area, reset the scroll for the dock downwards
                                        shellscape_app.reset_dock_scroll_down();

                                    // Check if the column and row of the event fall within the boundaries of the alerts area
                                    // Check if 'column' is within the notification's left and right boundaries
                                    // Check if 'row' is within the notification's top and bottom boundaries
                                    } else if notify_area.left() <= column && column <= notify_area.right()
                                        && notify_area.top() <= row && row <= notify_area.bottom() {
                                        // If the event is within the alerts area, reset the scroll for alerts downwards
                                        shellscape_app.reset_alerts_scroll_down();
                                    }
                                }
                                ShellscapeCommands::ScrollDown { column, row } => {
                                    // Get the current areas for the dock and alerts
                                    let dock_area = shellscape_app.get_dock_area();
                                    let notify_area = shellscape_app.get_alerts_area();

                                    // Check if the column and row of the event fall within the boundaries of the dock area
                                    // Check if 'column' is within the dock's left and right boundaries
                                    // Check if 'row' is within the dock's top and bottom boundaries
                                    if dock_area.left() <= column && column <= dock_area.right()
                                        && dock_area.top() <= row && row <= dock_area.bottom() {
                                        // If the event is within the dock area, reset the scroll for the dock upwards
                                        shellscape_app.reset_dock_scroll_up();

                                    // Check if the column and row of the event fall within the boundaries of the alerts area
                                    // Check if 'column' is within the notification's left and right boundaries
                                    // Check if 'row' is within the notification's top and bottom boundaries
                                    } else if notify_area.left() <= column && column <= notify_area.right()
                                        && notify_area.top() <= row && row <= notify_area.bottom() {
                                        // If the event is within the alerts area, reset the scroll for alerts upwards
                                        shellscape_app.reset_alerts_scroll_up();
                                    }
                                }
                                _ => {}
                            }
                        }
                        // Handle errors that occur while processing the Shellscape event.
                        Err(err) => {
                            // TODO: handle the error.

                            println!("{:?}", err);
                        }
                    }
                }
            }
        }

        // Clean up: Remove the temporary server port and abort the interface.
        pipeline.remove_server_port_in_temp()?;
        interface.abort()?;

        Ok(())
    }

    fn match_alerts_events(
        &mut self,
        event: Option<ShellscapeAlerts>,
        shellscape_app: &mut ShellscapeApp,
    ) {
        match event {
            Some(alert) => {
                shellscape_app.add_alert(alert);
            }
            None => {}
        }
    }

    /// Asynchronously handles incoming observer events and updates the Shellscape app
    /// based on the received `GaladrielEvents`.
    ///
    /// # Parameters
    /// - `event`: The observer event (`GaladrielEvents`) to process.
    /// - `shellscape_app`: A mutable reference to the Shellscape app for updating UI states and alerts.
    /// - `atomically_matcher`: A thread-safe reference-counted handle to manage configuration overrides.
    async fn match_observer_events(
        &mut self,
        event: GaladrielEvents,
        shellscape_app: &mut ShellscapeApp,
        atomically_matcher: Arc<RwLock<overrides::Override>>,
        sender: UnboundedSender<ShellscapeAlerts>,
        nenyr_parser: &mut NenyrParser,
    ) {
        match event {
            // If a notification event is received, add it directly to the Shellscape app.
            GaladrielEvents::Notify(notification) => {
                shellscape_app.add_alert(notification);
            }
            GaladrielEvents::Parse(path) => {
                let start_time = Local::now();
                let stringified_path = path.to_string_lossy().to_string();
                let notification = ShellscapeAlerts::create_information(
                    start_time,
                    &format!("Initiating parsing of: {:?}", stringified_path),
                );

                shellscape_app.add_alert(notification);

                let mut formera = Formera::new(path, self.configatron.get_auto_naming(), sender);

                match formera.start(nenyr_parser).await {
                    Ok(()) => {
                        let ending_time = Local::now();
                        let duration = ending_time - start_time;
                        let notification = ShellscapeAlerts::create_success(
                            start_time,
                            ending_time,
                            duration,
                            &format!("Successfully parsed Nenyr file: {:?}", stringified_path),
                        );

                        shellscape_app.add_alert(notification);

                        println!("{:?}\n", *STYLITRON);
                        println!("{:?}", *CLASSINATOR);
                    }
                    Err(GaladrielError::NenyrError { start_time, error }) => {
                        shellscape_app.add_alert(ShellscapeAlerts::create_nenyr_error(
                            start_time.to_owned(),
                            error.to_owned(),
                        ));
                    }
                    Err(err) => {
                        shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(
                            Local::now(),
                            err.to_owned(),
                        ));
                    }
                }
            }
            // Handle reloading of Galadriel configuration when requested by the event.
            GaladrielEvents::ReloadGaladrielConfigs => {
                let start_time = Local::now();

                // Attempt to load the latest Galadriel configuration.
                match self.load_galadriel_config() {
                    Ok(()) => {
                        // If successful, reconstruct the exclude matcher asynchronously.
                        self.reconstruct_exclude_matcher(
                            start_time,
                            atomically_matcher,
                            shellscape_app,
                        )
                        .await;

                        let ending_time = Local::now();
                        let duration = ending_time - start_time;
                        let notification = ShellscapeAlerts::create_success(
                            start_time,
                            ending_time,
                            duration,
                            "Galadriel CSS configurations updated successfully. System is now operating with the latest configuration.",
                        );

                        // Update the Shellscape app's state with the new configuration and add a success notification.
                        shellscape_app.reset_configs_state(self.configatron.clone());
                        shellscape_app.add_alert(notification);
                    }
                    // Log an error if configuration loading fails, and notify the Shellscape app.
                    Err(err) => {
                        tracing::error!(
                            "Unable to load Galadriel CSS configurations. Encountered error: {:?}",
                            err
                        );

                        shellscape_app
                            .add_alert(ShellscapeAlerts::create_galadriel_error(start_time, err));
                    }
                }
            }
            _ => {}
        }
    }

    /// Reconstructs the exclude matcher configuration asynchronously and updates the Shellscape app.
    ///
    /// # Arguments
    /// * `start_time` - The time at which reconstruction started, for logging and notification purposes.
    /// * `atomically_matcher` - A thread-safe matcher object to be updated with the new configuration.
    /// * `shellscape_app` - The application instance to update based on the new matcher configuration.
    async fn reconstruct_exclude_matcher(
        &self,
        start_time: DateTime<Local>,
        atomically_matcher: Arc<RwLock<overrides::Override>>,
        shellscape_app: &mut ShellscapeApp,
    ) {
        let mut matcher = atomically_matcher.write().await;

        match self.construct_exclude_matcher() {
            Ok(new_matcher) => {
                tracing::info!("Successfully applied new exclude matcher configuration.");

                *matcher = new_matcher;

                let ending_time = Local::now();
                let duration = ending_time - start_time;

                ShellscapeAlerts::create_success(start_time, ending_time, duration, "");
            }
            Err(err) => {
                tracing::error!(
                    "Failed to apply new exclude matcher configuration: {:?}",
                    err
                );

                shellscape_app.add_alert(ShellscapeAlerts::create_galadriel_error(start_time, err));
            }
        }
    }

    /// Extracts the local address from a TCP listener, handling any errors encountered.
    ///
    /// # Arguments
    /// * `listener` - The TCP listener instance to retrieve the address from.
    ///
    /// # Returns
    /// * A `GaladrielResult` containing the extracted `SocketAddr` or an error if extraction failed.
    fn extract_local_addr_from_listener(
        &self,
        listener: &TcpListener,
    ) -> GaladrielResult<SocketAddr> {
        listener.local_addr().map_err(|err| {
            GaladrielError::raise_critical_runtime_error(
                ErrorKind::ServerLocalAddrFetchFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })
    }

    /// Retrieves the port number from the local address of a TCP listener.
    ///
    /// # Arguments
    /// * `listener` - The TCP listener instance to retrieve the port number from.
    ///
    /// # Returns
    /// * A `GaladrielResult` containing the extracted port number or an error if extraction failed.
    fn retrieve_port_from_local_addr(&self, listener: &TcpListener) -> GaladrielResult<u16> {
        let local_addr = self.extract_local_addr_from_listener(listener)?;

        Ok(local_addr.port())
    }

    /// Constructs an exclude matcher for filtering files or directories.
    ///
    /// This method uses a builder pattern to construct an `Override` object
    /// that holds a list of paths to exclude from processing. It reads the exclude
    /// paths from the configuration, formats them correctly, and returns the built
    /// `Override` object for further use.
    ///
    /// # Returns
    /// Returns a `GaladrielResult` containing the built `Override` object or an error.
    fn construct_exclude_matcher(&self) -> GaladrielResult<overrides::Override> {
        // Initialize the override builder with the working directory.
        let mut overrides = overrides::OverrideBuilder::new(self.working_dir.clone());

        // Iterate through the list of excludes from the configuration and add them to the matcher.
        for exclude in self.configatron.get_exclude().iter() {
            overrides
                .add(&format!("!/{}", exclude.trim_start_matches("/")))
                .map_err(|err| {
                    GaladrielError::raise_general_other_error(
                        ErrorKind::ExcludeMatcherCreationError,
                        &err.to_string(),
                        ErrorAction::Notify,
                    )
                })?;
        }

        tracing::info!(
            "Exclude matcher constructed with {} patterns.",
            self.configatron.get_exclude().len()
        );

        // Return the built override object.
        overrides.build().map_err(|err| {
            GaladrielError::raise_general_other_error(
                ErrorKind::ExcludeMatcherBuildFailed,
                &err.to_string(),
                ErrorAction::Notify,
            )
        })
    }

    /// Loads the Galadriel CSS configuration from a JSON file.
    ///
    /// This method reads the `galadriel.config.json` file from the working directory
    /// and deserializes it into a `ConfigurationJson` struct. Then, it uses the configuration
    /// data to create and apply a new `Configatron` instance to the `GaladrielRuntime`.
    ///
    /// If loading and parsing the configuration is successful, the `configatron` is updated.
    /// If an error occurs during file reading or JSON parsing, it logs the error and returns a result.
    ///
    /// # Returns
    /// Returns `GaladrielResult<()>` indicating success or failure of the configuration load.
    fn load_galadriel_config(&mut self) -> GaladrielResult<()> {
        // Define the path to the Galadriel configuration file.
        let config_path = self.working_dir.join("galadriel.config.json");

        if config_path.exists() {
            tracing::debug!("Loading Galadriel CSS configuration from {:?}", config_path);

            // Attempt to read the configuration file as a string.
            match std::fs::read_to_string(config_path) {
                Ok(raw_config) => {
                    // Deserialize the JSON string into the ConfigurationJson struct.
                    let config_json: ConfigurationJson = serde_json::from_str(&raw_config)
                        .map_err(|err| {
                            GaladrielError::raise_general_other_error(
                                ErrorKind::ConfigFileParsingError,
                                &err.to_string(),
                                ErrorAction::Notify,
                            )
                        })?;

                    // Create a new Configatron instance with the deserialized data.
                    let configatron = Configatron::new(
                        config_json.exclude,
                        config_json.auto_naming,
                        config_json.reset_styles,
                        config_json.minified_styles,
                        config_json.port,
                        config_json.version,
                    );

                    // Apply the new configatron to the runtime.
                    self.configatron = configatron;
                    tracing::info!("Galadriel CSS configuration loaded and applied successfully.");
                }
                Err(err) => {
                    tracing::error!("Failed to read Galadriel CSS configuration file: {:?}", err);

                    return Err(GaladrielError::raise_general_other_error(
                        ErrorKind::ConfigFileReadError,
                        &err.to_string(),
                        ErrorAction::Notify,
                    ));
                }
            }
        } else {
            // Create a new Configatron instance with the deserialized data.
            self.configatron =
                Configatron::new(vec![], true, true, true, "0".to_string(), "*".to_string());

            tracing::warn!("Galadriel CSS is starting with default configurations as `galadriel.config.json` was not found in the root directory.");
        }

        Ok(())
    }

    async fn replace_configurations_file(&mut self) -> GaladrielResult<()> {
        let configs_json = self.configatron.generate_configs_json();
        let config_path = self.working_dir.join("galadriel.config.json");

        match OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(config_path)
            .await
        {
            Ok(mut file) => match serde_json::to_vec_pretty(&configs_json) {
                Ok(bytes) => {
                    if let Err(err) = file.write_all(&bytes).await {
                        return Err(GaladrielError::raise_general_runtime_error(
                            ErrorKind::GaladrielConfigFileWriteError,
                            &err.to_string(),
                            ErrorAction::Notify,
                        ));
                    }
                }
                Err(err) => {
                    return Err(GaladrielError::raise_general_runtime_error(
                        ErrorKind::GaladrielConfigSerdeSerializationError,
                        &err.to_string(),
                        ErrorAction::Notify,
                    ));
                }
            },
            Err(err) => {
                return Err(GaladrielError::raise_general_runtime_error(
                    ErrorKind::GaladrielConfigOpenFileError,
                    &err.to_string(),
                    ErrorAction::Notify,
                ));
            }
        }

        Ok(())
    }

    /// Generates a log filename based on the current timestamp.
    ///
    /// This method creates a filename for log files by formatting the current local
    /// time into a string that includes the year, month, day, hour, minute, and second.
    ///
    /// # Returns
    /// Returns a string containing the generated log filename, such as `galadrielcss_log_2024-11-07_14-35-25.log`.
    fn generate_log_filename(&self) -> String {
        // Get the current timestamp and format it as a string.
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

        // Return the log filename using the formatted timestamp.
        format!("galadrielcss_log_{}.log", timestamp)
    }
}
