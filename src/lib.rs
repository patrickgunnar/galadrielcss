use std::{io::Stdout, net::SocketAddr, path::PathBuf, sync::Arc};

use baraddur::Baraddur;
use chrono::Local;
use configatron::{
    construct_exclude_matcher, get_port, load_galadriel_configs, switch_auto_naming,
    switch_minified_styles, switch_reset_styles, transform_configatron_to_json,
};
use error::{ErrorAction, ErrorKind, GaladrielError};
use events::{GaladrielAlerts, GaladrielEvents};
use lothlorien::LothlorienPipeline;
use palantir::Palantir;
use ratatui::prelude::CrosstermBackend;
use shellscape::{
    app::ShellscapeApp, commands::ShellscapeCommands, events::ShellscapeTerminalEvents,
    ui::ShellscapeInterface, Shellscape,
};
use synthesizer::Synthesizer;
use tokio::{net::TcpListener, sync::RwLock};
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;
use utils::replace_file::replace_file;

mod astroform;
mod asts;
mod baraddur;
mod configatron;
mod crealion;
pub mod error;
mod events;
mod formera;
mod gatekeeper;
mod injectron;
mod intaker;
mod lothlorien;
mod palantir;
mod shellscape;
mod synthesizer;
mod trailblazer;
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
}

impl GaladrielRuntime {
    pub fn new(runtime_mode: GaladrielRuntimeKind, working_dir: PathBuf) -> Self {
        Self {
            runtime_mode,
            working_dir,
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
        load_galadriel_configs(&self.working_dir).await?;

        println!("Build process not implemented yet.");

        Ok(())
    }

    async fn configure_development_environment(&mut self) -> GaladrielResult<()> {
        // Load the galadriel configurations.
        load_galadriel_configs(&self.working_dir).await?;

        // Exclude matcher for file system monitoring
        let working_dir = self.working_dir.clone();
        let matcher = construct_exclude_matcher(&working_dir)?; // Create an exclude matcher based on the working directory.
        let atomically_matcher = Arc::new(RwLock::new(matcher)); // Wrap the matcher in an Arc and RwLock for thread-safe shared ownership and mutable access.

        // Initialize the Palantir alerts system.
        let palantir_alerts = Palantir::new();
        let palantir_sender = palantir_alerts.get_palantir_sender(); // Retrieve the Palantir sender from the palantir_alerts instance. This sender is used to send alerts to Palantir.
        let _start_alert_watcher = palantir_alerts.start_alert_watcher(); // Start the alert watcher using the palantir_alerts instance. This likely begins observing for new alerts or events.

        // Initialize the Shellscape terminal UI.
        let mut shellscape = Shellscape::new();
        let mut _shellscape_events = shellscape.create_events(250); // Event handler for Shellscape events
        let mut interface = shellscape.create_interface()?; // Terminal interface setup
        let mut shellscape_app = shellscape.create_app(palantir_sender.clone())?; // Application/state setup for Shellscape

        // Initialize the Lothlórien pipeline (WebSocket server for Galadriel CSS).
        let mut pipeline = LothlorienPipeline::new(get_port(), palantir_sender.clone());
        let pipeline_listener = pipeline.create_listener().await?; // Create WebSocket listener for pipeline
        let running_on_port = self.retrieve_port_from_local_addr(&pipeline_listener)?; // Extract port from the listener's local address
        let _listener_handler = pipeline.create_pipeline(pipeline_listener); // Start the WebSocket pipeline

        // Initialize the Barad-dûr file system observer.
        let mut baraddur_observer = Baraddur::new(250, working_dir, palantir_sender.clone());
        let matcher = Arc::clone(&atomically_matcher); // Clone the Arc reference to atomically_matcher for sharing it across threads safely.
        let (mut deb, deb_tx) = baraddur_observer.async_debouncer(matcher)?; // Call the async_debouncer method on the baraddur_observer instance, passing the cloned matcher. This returns a debouncer object and a sender (deb_tx).
        let matcher = Arc::clone(&atomically_matcher); // Clone the Arc reference to atomically_matcher for sharing it across threads safely.
        let _start_observation = baraddur_observer.watch(matcher, &mut deb, deb_tx.clone()); // Start observing with the baraddur_observer, using the matcher, the debouncer (mut deb), and the deb_tx sender for debouncing.

        shellscape_app.reset_server_running_on_port(running_on_port); // Set the running port.
        pipeline.register_server_port_in_temp(running_on_port)?; // Register the pipeline's server port in temporary storage.
        interface.invoke()?; // Start the Shellscape terminal interface rendering.

        let working_dir = self.working_dir.clone();
        let matcher = Arc::clone(&atomically_matcher);

        // Initialize and process all Nenyr files at the beginning of the development cycle.
        Synthesizer::new(true, matcher, palantir_sender.clone())
            .process(&working_dir)
            .await;

        // Transition to development runtime.
        let development_runtime_result = self
            .development_runtime(
                &mut pipeline,
                &mut shellscape,
                &mut shellscape_app,
                &mut baraddur_observer,
                &mut interface,
            )
            .await;

        // Clean up: Remove the temporary server port and abort the interface.
        pipeline.remove_server_port_in_temp()?;
        interface.abort()?;

        development_runtime_result
    }

    async fn development_runtime(
        &mut self,
        pipeline: &mut LothlorienPipeline,
        shellscape: &mut Shellscape,
        shellscape_app: &mut ShellscapeApp,
        baraddur_observer: &mut Baraddur,
        interface: &mut ShellscapeInterface<CrosstermBackend<Stdout>>,
    ) -> GaladrielResult<()> {
        tracing::info!("Galadriel CSS development runtime initiated.");

        // Get runtime sender for Lothlórien pipeline
        let pipeline_sender = pipeline.get_runtime_sender();

        loop {
            // Render the Shellscape terminal interface, handle potential errors.
            if let Err(err) = interface.render(shellscape_app) {
                tracing::error!("{:?}", err);

                return Err(err);
            }

            tokio::select! {
                // Handle events from the Lothlórien pipeline.
                pipeline_res = pipeline.next() => {
                    match pipeline_res {
                        // Handle error events from the Lothlórien pipeline and notify the application.
                        Ok(GaladrielEvents::Error(err)) => {
                            tracing::error!("{:?}", err);

                            return Err(err);
                        }
                        // Handle errors from the Lothlórien pipeline and notify the application.
                        Err(err) => {
                            tracing::error!("{:?}", err);

                            return Err(err);
                        }
                        _ => {}
                    }
                }
                // Handle events from the Baraddur observer (file system).
                baraddur_res = baraddur_observer.next() => {
                    match baraddur_res {
                        // Handle asynchronous debouncer errors from the observer and notify the application.
                        Ok(GaladrielEvents::Error(err)) => {
                            tracing::error!("{:?}", err);

                            return Err(err);
                        }
                        // Handle errors from the Barad-dûr observer and notify the application.
                        Err(err) => {
                            tracing::error!("{:?}", err);

                            return Err(err);
                        }
                        Ok(event) => {
                            if let Err(err) = pipeline_sender.send(event) {
                                tracing::error!("Something went wrong while sending event from observer to the server: Error: {:?}", err);
                            }
                        }
                    }
                }
                // Handle events from the Shellscape terminal interface.
                shellscape_res = shellscape.next() => {
                    match shellscape_res {
                        // Handle a valid event from the Shellscape terminal interface.
                        Ok(event) => {
                            // Exit the loop if the terminate command is received.
                            if let ShellscapeCommands::Terminate = self.handle_shellscape_event(event, shellscape, shellscape_app).await {
                                break;
                            }
                        }
                        // Handle errors that occur while processing the Shellscape event.
                        Err(err) => {
                            tracing::error!("{:?}", err);

                            return Err(err);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_shellscape_event(
        &mut self,
        event: ShellscapeTerminalEvents,
        shellscape: &mut Shellscape,
        shellscape_app: &mut ShellscapeApp,
    ) -> ShellscapeCommands {
        // Match the event to its corresponding Shellscape command.
        match shellscape.match_shellscape_event(event) {
            ShellscapeCommands::Terminate => {
                return ShellscapeCommands::Terminate;
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
                switch_reset_styles();

                if let Err(err) = self.replace_configurations_file().await {
                    shellscape_app
                        .add_alert(GaladrielAlerts::create_galadriel_error(Local::now(), err));
                }
            }
            ShellscapeCommands::ToggleMinifiedStyles => {
                switch_minified_styles();

                if let Err(err) = self.replace_configurations_file().await {
                    shellscape_app
                        .add_alert(GaladrielAlerts::create_galadriel_error(Local::now(), err));
                }
            }
            ShellscapeCommands::ToggleAutoNaming => {
                switch_auto_naming();

                if let Err(err) = self.replace_configurations_file().await {
                    shellscape_app
                        .add_alert(GaladrielAlerts::create_galadriel_error(Local::now(), err));
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
                if dock_area.left() <= column
                    && column <= dock_area.right()
                    && dock_area.top() <= row
                    && row <= dock_area.bottom()
                {
                    // If the event is within the dock area, reset the scroll for the dock downwards
                    shellscape_app.reset_dock_scroll_down();

                // Check if the column and row of the event fall within the boundaries of the alerts area
                // Check if 'column' is within the notification's left and right boundaries
                // Check if 'row' is within the notification's top and bottom boundaries
                } else if notify_area.left() <= column
                    && column <= notify_area.right()
                    && notify_area.top() <= row
                    && row <= notify_area.bottom()
                {
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
                if dock_area.left() <= column
                    && column <= dock_area.right()
                    && dock_area.top() <= row
                    && row <= dock_area.bottom()
                {
                    // If the event is within the dock area, reset the scroll for the dock upwards
                    shellscape_app.reset_dock_scroll_up();

                // Check if the column and row of the event fall within the boundaries of the alerts area
                // Check if 'column' is within the notification's left and right boundaries
                // Check if 'row' is within the notification's top and bottom boundaries
                } else if notify_area.left() <= column
                    && column <= notify_area.right()
                    && notify_area.top() <= row
                    && row <= notify_area.bottom()
                {
                    // If the event is within the alerts area, reset the scroll for alerts upwards
                    shellscape_app.reset_alerts_scroll_up();
                }
            }
            _ => {}
        }

        ShellscapeCommands::None
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

    async fn replace_configurations_file(&mut self) -> GaladrielResult<()> {
        let config_path = self.working_dir.join("galadriel.config.json");
        let serialized_configs = transform_configatron_to_json()?;

        replace_file(
            config_path,
            &serialized_configs,
            ErrorKind::GaladrielConfigOpenFileError,
            ErrorAction::Notify,
            ErrorKind::GaladrielConfigFileWriteError,
            ErrorAction::Notify,
        )
        .await
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
