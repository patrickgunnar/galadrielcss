//! # Galadriel CSS
//!
//! `Galadriel CSS` is a comprehensive and efficient framework for managing CSS styles in complex, scalable applications. It employs a modular, hierarchical design that allows for precise control over how styles are applied and inherited across various components and modules within a project. The framework’s key strengths lie in its extensibility, performance, and ease of maintenance, making it well-suited for both simple and large-scale applications.
//!
//! ## Overview
//!
//! The `Galadriel CSS` framework is built with scalability and modularity in mind. It organizes styles into a hierarchical context system that makes it easy to manage global, layout-specific, and module-specific styles in a highly structured manner. This context-driven approach ensures that styles can be inherited, overridden, or extended without unnecessary duplication or conflicts.
//!
//! At the heart of `Galadriel CSS` is **Nenyr**, a domain-specific language (DSL) designed to declare styles, variables, animations, and more. These Nenyr definitions are then processed into efficient, utility-first CSS during the build process. The framework is optimized for performance and clean code organization, reducing the potential for redundant CSS rules and improving load times.
//!
//! Core concepts of `Galadriel CSS` include:
//!
//! - **Context Hierarchy**: Organizes styles into multiple layers (Central, Layout, and Module contexts) to handle different scopes of styling.
//! - **Class Inheritance**: Supports inheritance of styles across contexts, enabling modular and reusable style definitions.
//! - **Context Extension**: Enables contexts to extend other contexts, ensuring consistency and extensibility across styles.
//! - **Variable and Animation Uniqueness**: Guarantees that variables and animations are scoped and resolved per context, preventing unintended conflicts.
//!
//! ## Context Hierarchy
//!
//! `Galadriel CSS` organizes its styles into a flexible, hierarchical context system, where each context defines a set of styles that apply to different parts of the application. The primary contexts in this system are:
//!
//! - **Central Context**: The foundation of all styles in the application. This context defines global styles, themes, and settings that apply across the entire project. Common settings like typography, color schemes, and general style rules are defined here.
//!
//! - **Layout Context**: Responsible for styling groups of Module Contexts or component sections that form the overall layout of the application. Layout contexts inherit styles from the Central Context by default and can define additional rules specific to the group of Module Contexts or components.
//!
//! - **Module Context**: Each module or component in the application can have its own set of styles encapsulated within a Module Context. These styles are scoped and only apply to the specific module or component, ensuring that modular components do not inadvertently influence the rest of the layout. Module contexts can extend Layout Contexts and the Central Context by default, inheriting styles while adding or overriding properties as needed.
//!
//! The context hierarchy allows for highly specific and reusable style definitions, ensuring that each level of the application can have its own isolated set of styles while still maintaining a connection to the broader application-wide settings.
//!
//! ## Class Inheritance and Context Extension
//!
//! `Galadriel CSS` supports powerful mechanisms for inheritance and extension to ensure that styles are applied consistently and efficiently across the entire project:
//!
//! - **Class Inheritance**: When defining a style in a parent context (e.g., Central or Layout Context), the child contexts (e.g., Module Contexts) can inherit those styles. This inheritance allows for streamlined and consistent styling across multiple layers without redundant CSS definitions. A style defined in a higher context will automatically cascade down to child contexts unless specifically overridden.
//!
//! - **Context Extension**: `Galadriel CSS` allows for the extension of contexts to enable the reuse of styles and maintain consistency. For example, a Module Context (extends the Central Context by default) can extend a Layout Context, which in turn extends the Central Context. This hierarchy of context extension ensures that modules inherit the layout and global styles while being able to define their own additional or overriding rules. The ability to extend contexts prevents the need to redefine common styles at each level of the application, promoting DRY (Don't Repeat Yourself) principles and reducing the overall CSS footprint.
//!
//! These inheritance and extension mechanisms ensure that styles are always applied in a predictable and controlled manner, resulting in an efficient and maintainable CSS output.
//!
//! ## Variable and Animation Uniqueness
//!
//! One of the key features of `Galadriel CSS` is its approach to variables and animations, which are scoped and unique to their respective contexts. This ensures that even if different contexts use the same variable or animation name, they are treated as independent entities, thus preventing potential conflicts and ensuring that styles are applied consistently.
//!
//! - **Variables**: Variables in `Galadriel CSS` are context-specific, meaning that the same variable name in different contexts will reference different values. This ensures that changes in one context will not inadvertently affect other contexts. For example, a color variable defined in the Central Context will be distinct from a color variable defined in a Module Context, even if both use the same identifier. The framework always resolves to the closest variable value in the context hierarchy, ensuring that each context’s specific needs are met.
//!
//! - **Animations**: Like variables, animations are scoped to the context in which they are defined. This means that different contexts can define their own set of animations without fear of conflicts. Animations will always be resolved to the closest context, ensuring that the correct animation is applied based on the current context. This scoping approach allows for the modular use of animations while maintaining consistency across the application.
//!
//! ## Implementation Details
//!
//! The core components of `Galadriel CSS` are designed to manage context-specific styles, handle inheritance and extension, and process variables and animations with unique scoping:
//!
//! - **Context Management**: This component is responsible for defining, extending, and managing contexts. It ensures that contexts are properly nested and that inheritance and extension rules are applied correctly. It also resolves the scope of variables and animations according to the context hierarchy.
//!
//! - **Style Application**: The style application system processes the defined Nenyr styles and generates efficient, utility-first CSS. It applies the appropriate styles based on the context hierarchy and ensures that no redundant CSS rules are created. The result is a minimal, optimized CSS output that reflects the exact styles required for each context.
//!
//! - **Variable and Animation Handling**: This system manages the resolution of variables and animations within their respective contexts. It ensures that each context has access to its own variables and animations while maintaining isolation from other contexts. This handling ensures that the application of these values is consistent and correct throughout the entire hierarchy.
//!
//! ## Usage
//!
//! To use `Galadriel CSS` in your application, you must define your styles using the **Nenyr** DSL. This involves specifying the contexts and their relationships to one another. The Nenyr definitions will be processed during the build stage, generating optimized CSS that respects the context hierarchy and inheritance rules.
//!
//! ### Example Workflow:
//!
//! 1. **Define Contexts**: Create your Central, Layout, and Module contexts using Nenyr syntax. You can define global styles in the Central Context, layout styles in the Layout Context, and module-specific styles in the Module Context.
//!
//! 2. **Extend Contexts**: If needed, extend contexts to inherit styles from parent contexts. For example, a Module Context (extend the Central Context by default) can extend a Layout Context to inherit layout-related styles, and the Layout Context extend the Central Context by default for global styles.
//!
//! 3. **Build Process**: During the build process, Nenyr definitions are compiled into efficient, utility-first CSS. This CSS will reflect the styles defined in the contexts and follow the inheritance and extension rules as specified.
//!
//! 4. **Apply Styles**: The generated CSS will be applied to your application and the utility class names related to a Nenyr class will be applied to the components where they were specified, with styles dynamically determined based on the context hierarchy. Styles will be applied according to the closest context, with inherited and extended styles resolved automatically.
//!
//! By using `Galadriel CSS` in this manner, you can easily manage complex styles in a modular and scalable way, ensuring your application remains clean, maintainable, and performant.
//!
//! For further integration details, refer to the specific methods and functions documented in the module, which provide advanced features for managing contexts, variables, animations, and other styling elements within `Galadriel CSS`.

use std::{io::Stdout, net::SocketAddr, path::PathBuf, sync::Arc};

use baraddur::Baraddur;
use chrono::Local;
use configatron::{
    construct_exclude_matcher, get_minified_styles, get_port, load_galadriel_configs,
    switch_auto_naming, switch_minified_styles, switch_reset_styles, transform_configatron_to_json,
};
use error::{ErrorAction, ErrorKind, GaladrielError};
use events::{GaladrielAlerts, GaladrielEvents};
use ignore::overrides;
use lothlorien::Lothlorien;
use palantir::Palantir;
use ratatui::prelude::CrosstermBackend;
use shellscape::{
    app::ShellscapeApp, commands::ShellscapeCommands, events::ShellscapeTerminalEvents,
    ui::ShellscapeInterface, Shellscape,
};
use synthesizer::Synthesizer;
use tokio::{
    net::TcpListener,
    sync::{broadcast, RwLock},
};
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, FmtSubscriber, Layer,
};
use utils::{
    get_updated_css::get_updated_css, replace_file::replace_file,
    restore_abstract_syntax_trees::restore_abstract_syntax_trees,
    serialize_classes_tracking::serialize_classes_tracking, write_file::write_file,
};

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

/// Represents the runtime modes of Galadriel CSS.
#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielRuntimeKind {
    /// Development mode for real-time updates and testing.
    Development,
    /// Build mode for compiling and generating production-ready outputs.
    Build,
}

/// A result type specific to Galadriel CSS operations, wrapping standard Rust results with `GaladrielError`.
pub type GaladrielResult<T> = Result<T, GaladrielError>;

/// Represents the runtime environment for Galadriel CSS.
#[derive(Clone, PartialEq, Debug)]
pub struct GaladrielRuntime {
    /// The mode in which the runtime is operating.
    runtime_mode: GaladrielRuntimeKind,
    /// The working directory for file operations and configurations.
    working_dir: PathBuf,
}

impl GaladrielRuntime {
    /// Creates a new Galadriel runtime instance with the specified mode and working directory.
    ///
    /// # Arguments
    ///
    /// * `runtime_mode` - The runtime mode (Development, or Build).
    /// * `working_dir` - The directory used for runtime operations.
    pub fn new(runtime_mode: GaladrielRuntimeKind, working_dir: PathBuf) -> Self {
        Self {
            runtime_mode,
            working_dir,
        }
    }

    /// Executes the runtime logic based on the current runtime mode.
    ///
    /// # Returns
    ///
    /// A `GaladrielResult` indicating success or failure.
    pub async fn run(&mut self) -> GaladrielResult<()> {
        match self.runtime_mode {
            GaladrielRuntimeKind::Development => self.start_development_mode().await,
            GaladrielRuntimeKind::Build => self.start_build_mode().await,
        }
    }

    /// Starts the development mode, setting up the environment and logging.
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

        tracing::debug!("Log subscriber built and configured.");

        // Set logs subscriber.
        tracing::subscriber::set_global_default(subscriber).map_err(|err| {
            tracing::error!("Failed to set log subscriber: {:?}", err.to_string());

            GaladrielError::raise_critical_runtime_error(
                ErrorKind::TracingSubscriberInitializationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        tracing::info!("Log subscriber set successfully.");

        // Configure the development runtime environment.
        self.configure_development_environment().await
    }

    /// Starts the build mode, processing styles and generating the final output.
    async fn start_build_mode(&mut self) -> GaladrielResult<()> {
        // Creates the build logs subscriber.
        let subscriber = tracing_subscriber::registry().with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(tracing_subscriber::filter::LevelFilter::ERROR),
        );

        // Starts the build subscriber.
        subscriber.try_init().map_err(|err| {
            tracing::error!("Failed to set log subscriber: {:?}", err.to_string());

            GaladrielError::raise_critical_runtime_error(
                ErrorKind::TracingSubscriberInitializationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        tracing::info!("Starting build mode.");

        // Load the galadriel configurations.
        load_galadriel_configs(&self.working_dir).await?;

        tracing::debug!("Loaded Galadriel CSS configurations.");

        // Exclude matcher for file system monitoring
        let working_dir = self.working_dir.clone();
        let matcher = construct_exclude_matcher(&working_dir)?; // Create an exclude matcher based on the working directory.
        let atomically_matcher = Arc::new(RwLock::new(matcher)); // Wrap the matcher in an Arc and RwLock for thread-safe shared ownership and mutable access.

        tracing::debug!("Created exclude matcher for working directory.");

        // Initialize the Palantir alerts system.
        let palantir_alerts = Palantir::new();
        let palantir_sender = palantir_alerts.get_palantir_sender(); // Retrieve the Palantir sender from the palantir_alerts instance. This sender is used to send alerts to Palantir.
        let _start_alert_watcher = palantir_alerts.start_alert_watcher(true); // Start the alert watcher using the palantir_alerts instance. This likely begins observing for new alerts or events.

        tracing::info!("Initialized Palantir alerts system.");
        tracing::info!("Started Nenyr file processing.");

        // Start the build process for all Nenyr files.
        Synthesizer::new(true, atomically_matcher, palantir_sender.clone())
            .process(true, &working_dir)
            .await;

        tracing::info!("Nenyr file processing finished.");

        // Get the most up-to-dated CSS.
        let css = get_updated_css();
        // Get the most up-to-dated Nenyr classes tracking maps
        let tracking = serialize_classes_tracking();

        tracing::debug!("Retrieved updated CSS and class tracking maps.");

        // Formats the final json.
        let folder_path = working_dir.join(".galadrielcss");
        let final_json_path = folder_path.join("galadrielcss.json");
        let final_json = format!("{{\"css\": {:?}, \"trackingClasses\": {}}}", css, tracking);

        // Creates the final json containing the CSS and Nenyr classes tracking map at root dir + `/.galadrielcss/galadrielcss.json`.
        write_file(
            folder_path,
            final_json_path,
            final_json,
            ErrorAction::Exit,
            ErrorKind::FileCreationError,
            ErrorKind::FileWriteError,
        )
        .await?;

        tracing::info!("Build process completed and final JSON file written.");

        Ok(())
    }

    /// Configures the development environment for Galadriel CSS.
    async fn configure_development_environment(&mut self) -> GaladrielResult<()> {
        tracing::info!("Configuring development environment.");

        // Load the galadriel configurations.
        load_galadriel_configs(&self.working_dir).await?;

        tracing::debug!("Loaded Galadriel configurations.");

        // Exclude matcher for file system monitoring
        let working_dir = self.working_dir.clone();
        let matcher = construct_exclude_matcher(&working_dir)?; // Create an exclude matcher based on the working directory.
        let atomically_matcher = Arc::new(RwLock::new(matcher)); // Wrap the matcher in an Arc and RwLock for thread-safe shared ownership and mutable access.

        tracing::debug!("Created exclude matcher for working directory.");

        // Initialize the Palantir alerts system.
        let palantir_alerts = Palantir::new();
        let palantir_sender = palantir_alerts.get_palantir_sender(); // Retrieve the Palantir sender from the palantir_alerts instance. This sender is used to send alerts to Palantir.
        let _start_alert_watcher = palantir_alerts.start_alert_watcher(false); // Start the alert watcher using the palantir_alerts instance. This likely begins observing for new alerts or events.

        tracing::info!("Initialized Palantir alerts system.");

        // Initialize the Shellscape terminal UI.
        let mut shellscape = Shellscape::new();
        let mut _shellscape_events = shellscape.create_events(250); // Event handler for Shellscape events
        let mut interface = shellscape.create_interface()?; // Terminal interface setup
        let mut shellscape_app = shellscape.create_app(palantir_sender.clone())?; // Application/state setup for Shellscape

        tracing::debug!("Initialized Shellscape UI and application state.");

        // Initialize the Lothlórien pipeline (axum server for Galadriel CSS).
        let mut pipeline = Lothlorien::new(&get_port(), palantir_sender.clone()); // Create a new instance of Lothlórien, passing the port and a clone of the palantir_sender for communication.
        let listener = pipeline.create_listener().await?; // Asynchronously create a TCP listener for the pipeline, binding it to the socket address.
        let socket_addr = self.get_local_addr_from_listener(&listener)?; // Retrieve the local address (IP and port) of the listener to know where the server is bound.
        let socket_port = socket_addr.port(); // Extract the port number from the socket address for further use.
        let _server_handler = pipeline.stream_sync(listener); // Start the axum server with the listener, handling incoming requests asynchronously.

        tracing::debug!(port = %socket_port, "Initialized Lothlórien server.");
        tracing::info!("Started server pipeline for Lothlórien.");

        // Initialize the Barad-dûr file system observer.
        let mut baraddur_observer = Baraddur::new(250, working_dir, palantir_sender.clone());
        let matcher = Arc::clone(&atomically_matcher); // Clone the Arc reference to atomically_matcher for sharing it across threads safely.
        let (mut deb, deb_tx) = baraddur_observer.async_debouncer(matcher)?; // Call the async_debouncer method on the baraddur_observer instance, passing the cloned matcher. This returns a debouncer object and a sender (deb_tx).
        let matcher = Arc::clone(&atomically_matcher); // Clone the Arc reference to atomically_matcher for sharing it across threads safely.
        let _start_observation = baraddur_observer.watch(matcher, &mut deb, deb_tx.clone()); // Start observing with the baraddur_observer, using the matcher, the debouncer (mut deb), and the deb_tx sender for debouncing.

        tracing::info!("Started Barad-dûr file system observer.");

        shellscape_app.reset_server_running_on_port(socket_port); // Set the running port.
        pipeline.register_server_port_in_temp(socket_port).await?; // Register the pipeline's server port in temporary storage.
        interface.invoke()?; // Start the Shellscape terminal interface rendering.

        tracing::debug!(
            "Registered pipeline server port in temporary storage: {}",
            socket_port
        );
        tracing::debug!("Started Shellscape terminal interface...");

        let working_dir = self.working_dir.clone();
        let matcher = Arc::clone(&atomically_matcher);

        tracing::info!("Starting initial Nenyr file processing...");

        // Initialize and process all Nenyr files at the beginning of the development cycle.
        Synthesizer::new(true, matcher, palantir_sender.clone())
            .process(get_minified_styles(), &working_dir)
            .await;

        tracing::info!("Initial Nenyr file processing finished.");
        tracing::info!("Transitioning to development runtime...");

        // Transition to development runtime.
        let development_runtime_result = self
            .development_runtime(
                &working_dir,
                &mut pipeline,
                &mut shellscape,
                &mut shellscape_app,
                &mut baraddur_observer,
                Arc::clone(&atomically_matcher),
                palantir_sender,
                &mut interface,
            )
            .await;

        // Clean up: Remove the temporary server port and abort the interface.
        pipeline.remove_server_port_in_temp().await?;
        interface.abort()?;

        tracing::debug!(
            "Removed server port from temporary storage: {}",
            socket_port
        );

        tracing::debug!("Aborted Shellscape interface...");
        tracing::info!("Development environment finalized.");

        development_runtime_result
    }

    /// Runs the development runtime loop, handling events from various sources.
    async fn development_runtime(
        &mut self,
        working_dir: &PathBuf,
        pipeline: &mut Lothlorien,
        shellscape: &mut Shellscape,
        shellscape_app: &mut ShellscapeApp,
        baraddur_observer: &mut Baraddur,
        matcher: Arc<RwLock<overrides::Override>>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        interface: &mut ShellscapeInterface<CrosstermBackend<Stdout>>,
    ) -> GaladrielResult<()> {
        tracing::info!("Galadriel CSS development runtime initiated.");

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
                    }
                }
                // Handle events from the Shellscape terminal interface.
                shellscape_res = shellscape.next() => {
                    match shellscape_res {
                        // Handle a valid event from the Shellscape terminal interface.
                        Ok(event) => {
                            // Exit the loop if the terminate command is received.
                            if let ShellscapeCommands::Terminate = self.handle_shellscape_event(
                                working_dir,
                                shellscape,
                                event,
                                shellscape_app,
                                Arc::clone(&matcher),
                                palantir_sender.clone(),
                            ).await {
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

    /// Handles events received from the Shellscape terminal and triggers the appropriate actions.
    ///
    /// # Parameters
    /// - `working_dir`: The working directory path of the application.
    /// - `shellscape`: A mutable reference to the Shellscape instance.
    /// - `event`: The specific event emitted by Shellscape to be handled.
    /// - `shellscape_app`: A mutable reference to the ShellscapeApp instance.
    /// - `matcher`: A shared, thread-safe reference to the Override matcher configuration.
    /// - `palantir_sender`: Broadcast sender for sending Galadriel alerts.
    ///
    /// # Returns
    /// - `ShellscapeCommands`: The command to be executed as a result of processing the event.
    async fn handle_shellscape_event(
        &mut self,
        working_dir: &PathBuf,
        shellscape: &mut Shellscape,
        event: ShellscapeTerminalEvents,
        shellscape_app: &mut ShellscapeApp,
        matcher: Arc<RwLock<overrides::Override>>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) -> ShellscapeCommands {
        // Match the event to its corresponding Shellscape command.
        match shellscape.match_shellscape_event(event) {
            ShellscapeCommands::Terminate => {
                // Handle termination command.
                return ShellscapeCommands::Terminate;
            }
            ShellscapeCommands::ResetAllAsts => {
                // Restore abstract syntax trees to their default state.
                self.restore_abstract_syntax_trees_to_default(
                    working_dir,
                    matcher,
                    palantir_sender,
                )
                .await;
            }
            ShellscapeCommands::ScrollNotificationsUp => {
                // Scroll notifications upwards.
                shellscape_app.reset_alerts_scroll_down();
            }
            ShellscapeCommands::ScrollNotificationsDown => {
                // Scroll notifications downwards.
                shellscape_app.reset_alerts_scroll_up();
            }
            ShellscapeCommands::ScrollDockUp => {
                // Scroll dock upwards.
                shellscape_app.reset_dock_scroll_down();
            }
            ShellscapeCommands::ScrollDockDown => {
                // Scroll dock downwards.
                shellscape_app.reset_dock_scroll_up();
            }
            ShellscapeCommands::ToggleResetStyles => {
                // Toggle reset styles and update configurations.
                switch_reset_styles();

                if let Err(err) = self.replace_configurations_file().await {
                    shellscape_app
                        .add_alert(GaladrielAlerts::create_galadriel_error(Local::now(), err));
                }
            }
            ShellscapeCommands::ToggleMinifiedStyles => {
                // Toggle minified styles and update configurations.
                switch_minified_styles();

                if let Err(err) = self.replace_configurations_file().await {
                    shellscape_app
                        .add_alert(GaladrielAlerts::create_galadriel_error(Local::now(), err));
                }
            }
            ShellscapeCommands::ToggleAutoNaming => {
                // Toggle auto-naming feature and update configurations.
                switch_auto_naming();

                if let Err(err) = self.replace_configurations_file().await {
                    shellscape_app
                        .add_alert(GaladrielAlerts::create_galadriel_error(Local::now(), err));
                }
            }
            ShellscapeCommands::ClearAlertsTable => {
                // Clear all alerts from the alerts cache.
                shellscape_app.clear_alerts();
            }
            ShellscapeCommands::VewShortcuts => {
                // Display keyboard shortcuts in the alerts table.
                shellscape_app.add_shortcut_alert();
            }
            ShellscapeCommands::ViewLicense => {
                // Display license information in the alerts table.
                shellscape_app.add_license_alert();
            }
            ShellscapeCommands::MakeDonation => {
                // Display donation information in the alerts table.
                shellscape_app.add_donation_alert();
            }
            ShellscapeCommands::ContributeAsDev => {
                // Display contribution information in the alerts table.
                shellscape_app.add_contribute_alert();
            }
            ShellscapeCommands::AboutAuthor => {
                // Display information about the author in the alerts table.
                shellscape_app.add_creator_alert();
            }
            ShellscapeCommands::ScrollUp { column, row } => {
                // Handle scrolling up based on the event's column and row.

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
                // Handle scrolling down based on the event's column and row.

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

    /// Restores the abstract syntax trees (ASTs) to their default state and notifies the server of changes.
    ///
    /// This method performs the following steps:
    /// 1. Restores the ASTs to their default state.
    /// 2. Reprocess the Nenyr contexts of the application.
    /// 3. Notifies the connected integration client to reload the CSS and reload all components of the application.
    async fn restore_abstract_syntax_trees_to_default(
        &self,
        working_dir: &PathBuf,
        matcher: Arc<RwLock<overrides::Override>>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) {
        tracing::info!("Starting restoration of abstract syntax trees to default state.");

        // Step 1: Reset ASTs to their default state.
        restore_abstract_syntax_trees();

        tracing::info!(
            "Reprocessing Nenyr contexts to repopulate abstract syntax trees with updated styles."
        );

        // Step 2: Create a `Synthesizer` to process styles with the restored ASTs.
        Synthesizer::new(true, matcher, palantir_sender.clone())
            .process(get_minified_styles(), working_dir)
            .await;
    }

    /// Replaces the configuration file with updated settings.
    ///
    /// # Returns
    /// * `GaladrielResult<()>` - Indicates success or provides an error if the process fails.
    async fn replace_configurations_file(&mut self) -> GaladrielResult<()> {
        tracing::info!("Starting configuration file replacement.");

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

    // Retrieves the local address of the provided TCP listener.
    fn get_local_addr_from_listener(&self, listener: &TcpListener) -> GaladrielResult<SocketAddr> {
        // Attempt to fetch the local address associated with the listener.
        listener.local_addr().map_err(|err| {
            GaladrielError::raise_critical_runtime_error(
                ErrorKind::ServerLocalAddrFetchFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })
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
