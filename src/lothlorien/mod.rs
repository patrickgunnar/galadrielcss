use axum::{extract::Path, routing, Router};
use chrono::Local;
use rand::Rng;
use serde::Deserialize;
use std::{env, path::PathBuf};

use tokio::{
    fs,
    net::TcpListener,
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::{GaladrielAlerts, GaladrielEvents},
    utils::{
        get_updated_css::get_updated_css, get_utility_class_names::get_utility_class_names,
        send_palantir_success_notification::send_palantir_success_notification,
        write_file::write_file,
    },
    GaladrielResult,
};

const GALADRIEL_TEMP_FILE_NAME: &str = "galadrielcss_lothlorien_pipeline_port.txt";

/// Represents different contexts in which styles are applied.
#[derive(Clone, PartialEq, Debug)]
pub enum ContextType {
    /// Central context, usually referring to global or main styles.
    Central,
    /// Layout context, typically related to layout-specific styles.
    Layout,
    /// Module context, generally referring to isolated or component-level styles.
    Module,
}

// Structure to deserialize utility collection parameters from HTTP requests.
#[derive(Deserialize)]
struct CollectUtilityParams {
    context_type: String, // Type of the context (e.g., "class", "layout").
    context_name: String, // Name of the context.
    class_name: String,   // Name of the class to be collected.
}

// Main server struct representing the Lothlorien server.
#[derive(Debug)]
pub struct Lothlorien {
    // Channel sender for server events.
    lothlorien_sender: mpsc::UnboundedSender<GaladrielEvents>,
    lothlorien_receiver: mpsc::UnboundedReceiver<GaladrielEvents>,

    // Broadcast sender for notifications.
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
    // Socket address for the server.
    socket_addr: String,
    // Path to the temporary folder for storing server metadata.
    temp_folder: PathBuf,
}

#[allow(dead_code)]
impl Lothlorien {
    // Constructor to create a new instance of the Lothlorien server.
    pub fn new(port: &str, palantir_sender: broadcast::Sender<GaladrielAlerts>) -> Self {
        // Create an unbounded channel for communication between the main runtime and server components.
        let (lothlorien_sender, lothlorien_receiver) = mpsc::unbounded_channel();

        Self {
            socket_addr: format!("127.0.0.1:{}", port), // Initialize the server's socket address.
            temp_folder: env::temp_dir(),               // Set the temporary folder path.
            lothlorien_sender,
            lothlorien_receiver,
            palantir_sender,
        }
    }

    // Getter for the server's sender channel.
    pub fn get_sender(&self) -> mpsc::UnboundedSender<GaladrielEvents> {
        self.lothlorien_sender.clone()
    }

    // Asynchronous method to receive the next server event.
    pub async fn next(&mut self) -> GaladrielResult<GaladrielEvents> {
        self.lothlorien_receiver.recv().await.ok_or_else(|| {
            GaladrielError::raise_general_pipeline_error(
                ErrorKind::ServerEventReceiveFailed,
                "Error while receiving response from Lothlórien server sender: No response received.",
                ErrorAction::Notify
            )
        })
    }

    // Register the server port in a temporary file.
    pub async fn register_server_port_in_temp(&self, port: u16) -> GaladrielResult<()> {
        tracing::info!("Registering server port {} in temp file.", port);

        // Construct the temp file path.
        let temp_file = self.temp_folder.join(GALADRIEL_TEMP_FILE_NAME);

        tracing::debug!("Temporary file path: {:?}", temp_file);

        write_file(
            self.temp_folder.clone(), // Temporary folder path.
            temp_file,                // Target temp file.
            format!("{port}"),        // Write the port number to the file.
            ErrorAction::Exit,
            ErrorKind::ServerPortRegistrationFailed,
            ErrorKind::ServerPortWriteError,
        )
        .await
    }

    // Remove the temporary file containing the server port.
    pub async fn remove_server_port_in_temp(&self) -> GaladrielResult<()> {
        // Locate the temp file.
        let temp_file = self.temp_folder.join(GALADRIEL_TEMP_FILE_NAME);

        tracing::info!("Attempting to remove temp file: {:?}", temp_file);

        if temp_file.exists() {
            tracing::debug!("Temp file exists. Proceeding with removal.");

            // Check if the file exists and remove it if it does.
            fs::remove_file(temp_file).await.map_err(|err| {
                GaladrielError::raise_general_pipeline_error(
                    ErrorKind::ServerPortRemovalFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })?;
        }

        Ok(())
    }

    // Create a TCP listener bound to the server's socket address.
    pub async fn create_listener(&self) -> GaladrielResult<TcpListener> {
        tracing::info!("Creating listener...");

        tokio::net::TcpListener::bind(self.socket_addr.clone())
            .await
            .map_err(|err| {
                GaladrielError::raise_general_pipeline_error(
                    ErrorKind::SocketAddressBindingError,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })
    }

    // Start the server's main event stream and set up routes.
    pub fn stream_sync(&self, socket_addr: TcpListener) -> JoinHandle<()> {
        tracing::info!("Starting server stream synchronization.");

        let lothlorien_sender = self.lothlorien_sender.clone();
        let palantir_sender = self.palantir_sender.clone();

        tokio::spawn(async move {
            tracing::info!("Configuring Axum server routes.");

            let app = Router::new()
                // Define a route for fetching CSS files.
                .route("/fetch-css", routing::get(Self::fetch_css))
                // Define a route for collecting utility class names.
                .route(
                    "/collect-utility-class-names/:context_type/:context_name/:class_name",
                    routing::get(Self::collect_utility_names),
                );

            tracing::info!("Starting Axum server with graceful shutdown.");

            // Start the Axum server with graceful shutdown support.
            let app_server = axum::serve(socket_addr, app).with_graceful_shutdown(
                Self::shutdown_signal(palantir_sender.clone(), lothlorien_sender.clone()),
            );

            send_palantir_success_notification(
                &Self::random_server_subheading_message(),
                Local::now(),
                palantir_sender.clone(),
            );

            // Handle server start errors.
            match app_server.await {
                Ok(()) => {
                    tracing::info!("Axum server shut down gracefully.");
                }
                Err(err) => {
                    tracing::error!("Axum server encountered an error: {}", err);

                    Self::send_galadriel_error(
                        err,
                        ErrorKind::ServerBidingError,
                        lothlorien_sender.clone(),
                    );
                }
            }
        })
    }

    // Handles a request to fetch the latest CSS.
    async fn fetch_css() -> String {
        get_updated_css()
    }

    // Handles a request to collect utility class names based on context type and parameters.
    async fn collect_utility_names(Path(params): Path<CollectUtilityParams>) -> String {
        // Destructure the incoming parameters.
        let CollectUtilityParams {
            context_type,
            context_name,
            class_name,
        } = params;

        // Match the context type to determine its specific configuration.
        let (context_type, context_name, class_name) = match context_type.as_str() {
            "@class" => (ContextType::Central, None, context_name),
            "@layout" => (ContextType::Layout, Some(context_name), class_name),
            "@module" => (ContextType::Module, Some(context_name), class_name),
            _ => return "".to_string(),
        };

        // Fetch and return the utility class names based on the context.
        get_utility_class_names(context_type, context_name, class_name)
    }

    // Awaits a shutdown signal and gracefully stops the server.
    async fn shutdown_signal(
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        lothlorien_sender: mpsc::UnboundedSender<GaladrielEvents>,
    ) {
        let mut palantir_receiver = palantir_sender.subscribe();

        tokio::select! {
            _ = lothlorien_sender.closed() => {}
            Err(broadcast::error::RecvError::Closed) = palantir_receiver.recv() => {}
        }
    }

    // Sends an error event using the Lothlórien sender channel.
    fn send_galadriel_error(
        err: std::io::Error,
        error_kind: ErrorKind,
        lothlorien_sender: mpsc::UnboundedSender<GaladrielEvents>,
    ) {
        tracing::error!("{:?}", err);

        let error = GaladrielError::raise_critical_other_error(
            error_kind,
            &err.to_string(),
            ErrorAction::Notify,
        );

        if let Err(err) = lothlorien_sender.send(GaladrielEvents::Error(error)) {
            tracing::error!(
                "Something went wrong while sending Galadriel event: {:?}",
                err
            );
        }
    }

    // Generates a random motivational server startup message.
    fn random_server_subheading_message() -> String {
        let messages = [
            "The light of Eärendil shines. Lothlórien is ready to begin your journey.",
            "The stars of Lothlórien guide your path. The system is fully operational.",
            "As the Mallorn trees bloom, Lothlórien is prepared for your commands.",
            "The Mirror of Galadriel is clear—development is ready to proceed.",
            "Lothlórien is fully operational and ready for development.",
        ];

        let idx = rand::thread_rng().gen_range(0..messages.len());
        let selected_message = messages[idx].to_string();

        tracing::debug!("Selected random subheading message: {}", selected_message);

        selected_message
    }
}
