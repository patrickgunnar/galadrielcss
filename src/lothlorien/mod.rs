use std::{env, fs, net::SocketAddr, path::PathBuf};

use chrono::{DateTime, Local};
use events::ClientResponse;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use rand::Rng;
use request::{ContextType, Request, RequestType, ServerRequest};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task::{JoinError, JoinHandle},
};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tungstenite::Message;

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::{GaladrielAlerts, GaladrielEvents},
    utils::{get_updated_css::get_updated_css, get_utility_class_names::get_utility_class_names},
    GaladrielResult,
};

pub mod events;
pub mod request;

const GALADRIEL_TEMP_FILE_NAME: &str = "galadrielcss_lothlorien_pipeline_port.txt";

/// Represents a pipeline for managing events and connections in the Lothlórien system.
/// It manages communication through a set of channels and listeners, enabling the processing of events
/// between the pipeline and the runtime environment.
#[allow(dead_code)]
#[derive(Debug)]
pub struct LothlorienPipeline {
    // Sender for pipeline events
    pipeline_sender: UnboundedSender<GaladrielEvents>,
    // Receiver for pipeline events
    pipeline_receiver: UnboundedReceiver<GaladrielEvents>,

    // Sender for connected client events
    runtime_sender: broadcast::Sender<GaladrielEvents>,
    // Receiver for connected client events
    runtime_receiver: broadcast::Receiver<GaladrielEvents>,

    /// Broadcast sender for sending alerts (`GaladrielAlerts`).
    palantir_sender: broadcast::Sender<GaladrielAlerts>,

    // The socket address for binding the TCP listener
    socket_addr: String,
    // Temporary folder for storing system-related files
    systems_temp_folder: PathBuf,
}

impl LothlorienPipeline {
    /// Creates a new `LothlorienPipeline` instance with the specified port.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to use for binding the listener.
    /// * `palantir_sender`: A broadcast sender for alerts.
    ///
    /// # Returns
    ///
    /// A new `LothlorienPipeline` instance.
    pub fn new(port: String, palantir_sender: broadcast::Sender<GaladrielAlerts>) -> Self {
        // Create unbounded channels for pipeline and runtime communication
        let (pipeline_sender, pipeline_receiver) = mpsc::unbounded_channel();
        let (runtime_sender, runtime_receiver) = broadcast::channel(2);

        tracing::info!("Initializing LothlorienPipeline with port: {}", port);

        Self {
            palantir_sender,
            pipeline_sender,
            pipeline_receiver,
            runtime_sender,
            runtime_receiver,
            // Format the socket address using the given port
            socket_addr: format!("127.0.0.1:{}", port),
            // Set the temporary folder path to the system's temp directory
            systems_temp_folder: env::temp_dir(),
        }
    }

    /// Creates a TCP listener bound to the socket address defined for this pipeline.
    ///
    /// # Returns
    ///
    /// A result containing either the `TcpListener` instance or an error.
    pub async fn create_listener(&self) -> GaladrielResult<TcpListener> {
        tracing::info!("Attempting to bind to socket address: {}", self.socket_addr);

        TcpListener::bind(self.socket_addr.clone())
            .await
            .map_err(|err| {
                tracing::error!(
                    "Failed to bind to socket address '{}': {:?}",
                    self.socket_addr,
                    err
                );

                GaladrielError::raise_general_pipeline_error(
                    ErrorKind::SocketAddressBindingError,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })
    }

    /// Starts the pipeline listener in a new asynchronous task, which accepts client
    /// connections and processes them asynchronously.
    ///
    /// This function listens for incoming client connections on the provided `TcpListener`,
    /// spawning a new asynchronous task to handle each connection. If an error occurs during
    /// connection handling, it sends an error notification to the main pipeline. The listener
    /// will shut down gracefully if the pipeline sender is closed.
    ///
    /// # Arguments
    ///
    /// * `listener` - The `TcpListener` to use for accepting client connections.
    ///
    /// # Returns
    ///
    /// A `JoinHandle<()>` that can be used to monitor the spawned task.
    pub fn create_pipeline(&self, listener: TcpListener) -> JoinHandle<()> {
        // Clone the pipeline, palantir and runtime senders to be used inside the spawned task
        let pipeline_sender = self.pipeline_sender.clone();
        let runtime_sender = self.runtime_sender.clone();
        let palantir_sender = self.palantir_sender.clone();

        // Subscribe the palantir and runtime sender
        let mut runtime_receiver = runtime_sender.subscribe();
        let mut palantir_receiver = palantir_sender.subscribe();

        Self::send_palantir_success_notification(
            &Self::random_server_subheading_message(),
            Local::now(),
            palantir_sender.clone(),
        );

        tracing::info!("Starting pipeline listener. Awaiting client connections...");

        // Spawn a new asynchronous task to handle incoming connections
        tokio::spawn(async move {
            // Infinite loop to continuously accept and process client connections
            loop {
                tokio::select! {
                    // If the pipeline sender is closed, log a warning and gracefully exit
                    _ = pipeline_sender.closed() => {
                       tracing::warn!("Pipeline sender has been closed. Shutting down listener gracefully.");
                        break;
                    }
                    // If the runtime sender is closed, log a warning and gracefully exit
                    Err(broadcast::error::RecvError::Closed) = runtime_receiver.recv() => {
                       tracing::warn!("Runtime sender has been closed. Shutting down listener gracefully.");
                        break;
                    }
                    // Exit the loop if the Palantir receiver is closed.
                    Err(broadcast::error::RecvError::Closed) = palantir_receiver.recv() => {
                        tracing::info!("Palantir sender has been closed. Shutting down listener gracefully.");
                        break;
                    }
                    // Continuously accept incoming client connections
                    connection = listener.accept() => {
                        Self::handle_client_connection(
                            pipeline_sender.clone(),
                            palantir_sender.clone(),
                            runtime_sender.clone(),
                            connection
                        )
                        .await;
                    }
                }
            }
        })
    }

    /// Handles an incoming client connection, either establishing communication
    /// or handling connection errors appropriately.
    ///
    /// # Parameters
    /// - `pipeline_sender`: An `UnboundedSender` used to send events to the pipeline.
    /// - `palantir_sender`: A `broadcast::Sender` used to send alerts to monitoring systems.
    /// - `runtime_sender`: A `broadcast::Sender` used to communicate client events to the runtime.
    /// - `connection`: A `Result` containing either the successfully accepted client connection
    ///   (`(TcpStream, SocketAddr)`) or an error (`std::io::Error`).
    async fn handle_client_connection(
        pipeline_sender: UnboundedSender<GaladrielEvents>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        runtime_sender: broadcast::Sender<GaladrielEvents>,
        connection: Result<(TcpStream, SocketAddr), std::io::Error>,
    ) {
        match connection {
            // If connection is successfully accepted, spawn a new task to handle the stream
            Ok((stream, _)) => {
                tracing::info!("Accepted new connection from client.");

                let palantir_tx = palantir_sender.clone();

                // Spawn a new task to handle the client stream
                let stream_sync_result = tokio::spawn(async move {
                    Self::stream_sync(stream, pipeline_sender, palantir_tx, runtime_sender).await
                })
                .await;

                // Handle the result of the stream handling task
                Self::handle_stream_sync_result(palantir_sender.clone(), stream_sync_result);
            }
            // If an error occurs while accepting the connection, log the error and notify the pipeline
            Err(err) => {
                let error = GaladrielError::raise_general_pipeline_error(
                    ErrorKind::ConnectionInitializationError,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                tracing::error!("Failed to accept incoming client connection: {:?}", error);

                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );
            }
        }
    }

    /// Handles the result of a client stream synchronization process.
    ///
    /// # Parameters
    /// - `palantir_sender`: A `broadcast::Sender` used to send alerts or notifications to the monitoring system.
    /// - `stream_sync_result`: A nested `Result` where:
    ///   - The outer `Result` indicates whether the task completed successfully or encountered a `JoinError`.
    ///   - The inner `Result` indicates the success or failure of the stream handling process (`GaladrielError`).
    fn handle_stream_sync_result(
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        stream_sync_result: Result<Result<(), GaladrielError>, JoinError>,
    ) {
        match stream_sync_result {
            // If the stream is handled successfully, log the success
            Ok(Ok(_)) => {
                tracing::info!("Connection handled successfully.");

                Self::send_palantir_success_notification(
                    "Client has successfully disconnected from the Galadriel CSS server. No further events will be sent to this client.",
                    Local::now(),
                    palantir_sender.clone()
                );
            }
            // If an error occurs while processing the connection, log the error
            Ok(Err(error)) => {
                tracing::error!(
                    "Error occurred while processing client connection: {:?}",
                    error
                );

                // Send an error notification to the pipeline sender
                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );
            }
            // If an unexpected error occurs, log the error and notify the pipeline
            Err(err) => {
                let error = GaladrielError::raise_general_pipeline_error(
                    ErrorKind::ConnectionTerminationError,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                tracing::error!("Unexpected error in handling connection: {:?}", err);

                // Send an error notification to the pipeline sender
                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );
            }
        }
    }

    /// Receives the next event from the pipeline receiver.
    ///
    /// # Returns
    ///
    /// A result containing either the received event or an error.
    pub async fn next(&mut self) -> GaladrielResult<GaladrielEvents> {
        self.pipeline_receiver.recv().await.ok_or_else(|| {
           tracing::error!("Failed to receive Lothlórien pipeline event: Channel closed unexpectedly or an IO error occurred");

            GaladrielError::raise_general_pipeline_error(
                ErrorKind::ServerEventReceiveFailed,
                "Error while receiving response from Lothlórien pipeline sender: No response received.",
                ErrorAction::Notify
            )
        })
    }

    /// Retrieves the `runtime_sender` to send events to connected clients.
    ///
    /// # Returns
    ///
    /// The `broadcast::Sender<GaladrielEvents>` for sending events.
    pub fn get_runtime_sender(&self) -> broadcast::Sender<GaladrielEvents> {
        tracing::info!("Retrieving runtime sender.");
        self.runtime_sender.clone()
    }

    /// Registers the server's port in a temporary file for later retrieval.
    ///
    /// # Arguments
    ///
    /// * `port` - The server port to register.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn register_server_port_in_temp(&self, port: u16) -> GaladrielResult<()> {
        use std::io::Write;

        let systems_temp_file = self.systems_temp_folder.join(GALADRIEL_TEMP_FILE_NAME);

        let mut file = fs::File::create(&systems_temp_file).map_err(|err| {
            GaladrielError::raise_general_pipeline_error(
                ErrorKind::ServerPortRegistrationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        tracing::info!(
            "Registering server port {} in temporary file: {:?}",
            port,
            systems_temp_file
        );

        write!(file, "{}", port).map_err(|err| {
            GaladrielError::raise_general_pipeline_error(
                ErrorKind::ServerPortWriteError,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        Ok(())
    }

    /// Removes the server's port registration file from the temporary folder.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn remove_server_port_in_temp(&self) -> GaladrielResult<()> {
        let systems_temp_file = self.systems_temp_folder.join(GALADRIEL_TEMP_FILE_NAME);

        if systems_temp_file.exists() {
            tracing::info!(
                "Removing server port registration file: {:?}",
                systems_temp_file
            );

            fs::remove_file(systems_temp_file).map_err(|err| {
                GaladrielError::raise_general_pipeline_error(
                    ErrorKind::ServerPortRemovalFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })?;
        } else {
            tracing::warn!(
                "Server port registration file does not exist: {:?}",
                systems_temp_file
            );
        }

        Ok(())
    }

    /// Handles the synchronization of streams for each connected client.
    /// It manages incoming and outgoing messages between the client and the server,
    /// and sends events to the pipeline and runtime as needed.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream representing the connection with the client.
    /// * `pipeline_sender` - The sender for sending pipeline events to the runtime, such as notifications or errors.
    /// * `runtime_sender` - The sender for broadcasting client events to the runtime system.
    ///
    /// # Returns
    ///
    /// Returns a `GaladrielResult<()>`, which indicates the success or failure of the synchronization process.
    /// A successful synchronization returns `Ok(())`, while failures are reported through `Err` with the appropriate error details.
    ///
    /// # Errors
    ///
    /// This function may return errors related to WebSocket connection failures, message sending issues, or event processing failures.
    async fn stream_sync(
        stream: tokio::net::TcpStream,
        pipeline_sender: UnboundedSender<GaladrielEvents>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        runtime_sender: broadcast::Sender<GaladrielEvents>,
    ) -> GaladrielResult<()> {
        // Establishes a WebSocket connection and splits it into sender and receiver components.
        let (mut stream_sender, mut stream_receiver) = Self::accept_sync(stream).await?.split();

        // Subscribes to the runtime sender to receive events.
        let mut runtime_receiver = runtime_sender.subscribe();
        let mut palantir_receiver = palantir_sender.subscribe();

        // Send initial message to the connected client.
        Self::send_server_notification_to_client(
            "Galadriel CSS server is ready! 🚀\n\nYou can start styling your project with Galadriel CSS and see instant updates as changes are made.\n\nHappy coding, and may your styles be ever beautiful!",
            palantir_sender.clone(),
            &mut stream_sender
        ).await;

        // Send successful connection message to the client.
        Self::send_palantir_success_notification(
            "A new client has successfully connected to the Galadriel server and is now ready to request and receive events.",
            Local::now(),
            palantir_sender.clone()
        );

        tracing::info!("Successfully established WebSocket connection. Sending initial greeting message to client.");

        // Observer the integration client connection.
        Self::process_stream_sync_watch(
            pipeline_sender,
            palantir_sender,
            &mut palantir_receiver,
            &mut stream_receiver,
            &mut runtime_receiver,
            &mut stream_sender,
        )
        .await
    }

    /// Processes the stream synchronization watch, continuously monitoring and handling
    /// messages between the server and client through WebSocket communication.
    ///
    /// # Parameters
    /// - `pipeline_sender`: An `UnboundedSender` to send events to the pipeline.
    /// - `palantir_sender`: A `broadcast::Sender` used for sending alerts to the monitoring system.
    /// - `palantir_receiver`: A mutable reference to a `broadcast::Receiver` that receives close event from the monitoring system.
    /// - `stream_receiver`: A mutable reference to the `SplitStream` of the WebSocket connection, used to receive messages from the client.
    /// - `runtime_receiver`: A mutable reference to a `broadcast::Receiver` that listens for runtime events.
    /// - `stream_sender`: A mutable reference to the `SplitSink` of the WebSocket connection, used to send messages to the client.
    ///
    /// # Returns
    /// - `GaladrielResult<()>`: Returns `Ok(())` on successful execution or an error encapsulated in `GaladrielResult`.
    async fn process_stream_sync_watch(
        pipeline_sender: UnboundedSender<GaladrielEvents>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        palantir_receiver: &mut broadcast::Receiver<GaladrielAlerts>,
        stream_receiver: &mut SplitStream<WebSocketStream<TcpStream>>,
        runtime_receiver: &mut broadcast::Receiver<GaladrielEvents>,
        stream_sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> GaladrielResult<()> {
        loop {
            tokio::select! {
                // If the pipeline sender is closed, the loop terminates gracefully.
                _ = pipeline_sender.closed() => {
                   tracing::warn!("Pipeline sender has been closed. Closing the stream synchronization process.");
                    break;
                }
                // Exit the loop if the Palantir receiver is closed.
                Err(broadcast::error::RecvError::Closed) = palantir_receiver.recv() => {
                    tracing::info!("Palantir sender has been closed. Shutting down listener gracefully.");
                    break;
                }
                // Receives events from the runtime system.
                runtime_response = runtime_receiver.recv() => {
                    match runtime_response {
                        // If receives an event from runtime, processes it.
                        Ok(event) => {
                            // Send the current event to the connected client.
                            Self::dispatch_event_to_client(
                                event,
                                palantir_sender.clone(),
                                stream_sender
                            ).await;
                        }
                        // If the runtime sender is closed, log a warning and gracefully exit
                        Err(broadcast::error::RecvError::Closed) => {
                            tracing::warn!("Runtime sender has been closed. Shutting down listener gracefully.");
                            break;
                        }
                        _ => {}
                    }
                }
                // Receives events from the connected integration client.
                client_response = stream_receiver.next() => {
                    match Self::handle_stream_response(palantir_sender.clone(), &client_response) {
                        ClientResponse::Break  => {
                            break;
                        }
                        ClientResponse::Text(data) => {
                            // Send the formatted message to the client using the WebSocket stream.
                            Self::send_server_notification_to_client(
                                &data,
                                palantir_sender.clone(),
                                stream_sender
                            )
                            .await;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// This asynchronous function is responsible for dispatching events to a connected client
    /// over a WebSocket connection. It takes different event types and formats them into a
    /// string that will be sent to the client. Based on the type of event, a specific format
    /// is chosen, and the event is sent using the `send_server_notification_to_client` method.
    ///
    /// # Parameters
    /// - `event`: The event to be dispatched to the client. It is of type `GaladrielEvents`,
    ///   which determines the content of the message that will be sent.
    /// - `palantir_sender`: A broadcast sender used for sending alerts.
    /// - `stream_sender`: A mutable reference to the `SplitSink` of the WebSocket stream.
    ///   It is used to send the formatted event message to the connected client.
    async fn dispatch_event_to_client(
        event: GaladrielEvents,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        stream_sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    ) {
        // Determine the message to send based on the event type.
        // Each case matches a different event, and formats a corresponding string message.
        let send_to_client = match event {
            // If the event is "RefreshCSS", prepare a message to refresh the CSS on the client.
            GaladrielEvents::RefreshCSS => {
                format!("refresh-css;{}", get_updated_css())
            }
            // If the event is "RefreshFromRoot", prepare a message to refresh the application starting from the root.
            GaladrielEvents::RefreshFromRoot => "refresh-from-root".to_string(),
            // If the event is "RefreshComponent", prepare a message to refresh a single component in the application.
            GaladrielEvents::RefreshComponent(file_file) => {
                format!("refresh-component;{}", file_file)
            }
            // If the event is "RefreshFromLayoutParent", prepare a message to refresh components starting from the folder path.
            GaladrielEvents::RefreshFromLayoutParent(folder_path) => {
                format!("refresh-from-layout-parent;{}", folder_path)
            }
            _ => return,
        };

        tracing::debug!("Sending event to connected client: {}", send_to_client);

        // Send the formatted message to the client using the WebSocket stream.
        Self::send_server_notification_to_client(
            &send_to_client,
            palantir_sender.clone(),
            stream_sender,
        )
        .await;
    }

    /// Accepts an incoming TCP stream and upgrades it to a WebSocket connection.
    ///
    /// # Parameters
    /// - `stream`: The incoming TCP stream to be upgraded.
    ///
    /// # Returns
    /// - `GaladrielResult<WebSocketStream<tokio::net::TcpStream>>`: Returns the WebSocket stream on success
    ///   or an error wrapped in `GaladrielResult` on failure.
    async fn accept_sync(
        stream: tokio::net::TcpStream,
    ) -> GaladrielResult<WebSocketStream<tokio::net::TcpStream>> {
        // Establishes a WebSocket connection.
        accept_async(stream).await.map_err(|err| {
            tracing::error!(
                "Failed to establish WebSocket connection with client stream: {:?}",
                err
            );

            GaladrielError::raise_critical_pipeline_error(
                ErrorKind::ServerSyncAcceptFailed,
                &err.to_string(),
                ErrorAction::Notify,
            )
        })
    }

    /// Sends a notification message to the connected client.
    ///
    /// # Parameters
    /// - `message`: The message string to be sent to the client.
    /// - `palantir_sender`: A broadcast sender used for sending alerts.
    /// - `stream_sender`: A mutable reference to the WebSocket stream's sink for sending messages.
    async fn send_server_notification_to_client(
        message: &str,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        stream_sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    ) {
        // Sends message to the client and handles any errors during the send operation.
        if let Err(err) = stream_sender.send(Message::Text(message.to_string())).await {
            tracing::error!(
                "Failed to send notification to connected client. Error: {:?}",
                err
            );

            let error = GaladrielError::raise_general_pipeline_error(
                ErrorKind::NotificationSendError,
                &format!(
                    "Failed to send notification to connected client. Error: {}",
                    err.to_string()
                ),
                ErrorAction::Notify,
            );

            Self::send_palantir_error_notification(error, Local::now(), palantir_sender.clone());
        }
    }

    /// Handles responses received from the client via WebSocket.
    ///
    /// # Parameters
    /// - `palantir_sender`: A broadcast sender for sending alerts.
    /// - `client_response`: An optional result containing the client's message or an error.
    ///
    /// # Returns
    /// - `ClientResponse`: Represents the outcome of processing the client's response.
    fn handle_stream_response(
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
        client_response: &Option<Result<Message, tungstenite::Error>>,
    ) -> ClientResponse {
        match client_response {
            // Successfully received a message from the client.
            Some(Ok(event)) => {
                tracing::info!("Received event from client: {:?}", event);

                // Processes the received event and handles the associated request.
                Self::process_stream_event(event, palantir_sender.clone())
            }
            // If no more data is received, the client has disconnected.
            None => {
                tracing::info!("Client has disconnected. Terminating stream synchronization.");

                return ClientResponse::Break;
            }
            // If an error occurs while receiving the client's message, handle it.
            Some(Err(err)) => {
                tracing::error!(
                    "Error receiving message from client: {:?}. Disconnecting client.",
                    err
                );

                // Raises a general pipeline error and notifies the runtime of the error.
                let error = GaladrielError::raise_general_pipeline_error(
                    ErrorKind::ClientResponseError,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );

                return ClientResponse::Break;
            }
        }
    }

    /// Processes a received WebSocket event and handles associated client requests.
    ///
    /// # Parameters
    /// - `event`: The WebSocket message received from the client.
    /// - `palantir_sender`: A broadcast sender for sending alerts.
    ///
    /// # Returns
    /// - `ClientResponse`: Represents the outcome of processing the event.
    fn process_stream_event(
        event: &Message,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) -> ClientResponse {
        // Processes the received stream event and handles the associated request.
        match Self::process_response(event) {
            Ok(ServerRequest { request, .. }) if request == Request::BreakConnection => {
                return ClientResponse::Break;
            }
            Ok(request) => {
                // Handles the processed request and sends notifications to the runtime.
                let data = Self::process_request(request);

                tracing::info!("Successfully processed request: {:?}", data);

                return ClientResponse::Text(data);
            }
            // Handles errors while processing the response.
            Err(error) => {
                tracing::error!(
                    "An error occurred while processing the client's request: {:?}",
                    error
                );

                // Notifies the runtime of the error during request processing.
                Self::send_palantir_error_notification(
                    error,
                    Local::now(),
                    palantir_sender.clone(),
                );

                return ClientResponse::Continue;
            }
        }
    }

    /// Processes a `Message` event and extracts relevant tokens to create a `ServerRequest`.
    /// This function processes a text message, validates the format, and constructs a `ServerRequest` based on the request type.
    ///
    /// # Parameters
    /// - `event`: The `Message` to process, expected to be of type `Message::Text`.
    ///
    /// # Returns
    /// A `GaladrielResult<ServerRequest>`. It returns an `Ok(ServerRequest)` if the message is valid, or an error if any issues occur during processing.
    fn process_response(event: &Message) -> GaladrielResult<ServerRequest> {
        // Check if the message is of type `Message::Text`
        match event {
            Message::Text(text_response) => {
                return Self::process_text_response(text_response);
            }
            Message::Close(_) => {
                return Ok(ServerRequest::new("".to_string(), Request::BreakConnection));
            }
            _ => {}
        }

        tracing::error!("Unsupported message format received from integration client. Only `Message::Text` is supported for processing.");

        // Return an error for unsupported message types
        Err(GaladrielError::raise_general_pipeline_error(
            ErrorKind::UnsupportedRequestToken,
            "Unsupported message format received from integration client. Only `Message::Text` is supported for processing.",
            ErrorAction::Ignore,
        ))
    }

    /// Processes a textual response from the client and converts it into a server request.
    ///
    /// # Parameters
    /// - `text_response`: A string slice containing the client's response.
    ///
    /// # Returns
    /// - `GaladrielResult<ServerRequest>`: A server request object if parsing and validation succeed,
    ///   or an error wrapped in `GaladrielResult` if any validation fails.
    fn process_text_response(text_response: &str) -> GaladrielResult<ServerRequest> {
        // Split the response string by semicolon and collect into a vector of strings
        let tokens: Vec<String> = text_response.split(";").map(|v| v.to_owned()).collect();

        // Ensure there are at least two tokens: `request type` and `client name`
        Self::token_less_than(
            2,
            &tokens,
            ErrorKind::MissingRequestTokens,
            ErrorAction::Ignore,
            "The integration client submitted an invalid request format: Expected at least 2 tokens but received fewer. Please provide both a `request type` and a `client name`, separated by a semicolon (`;`). For example: `fetch-updated-css;Client Name`.",
        )?;

        tracing::debug!("Processing request with tokens: {:?}", tokens);

        // Extract the `request type` and `client name` from the tokens
        let request_type = Self::get_request_type(&tokens[0])?;
        let client_name = tokens[1].clone();

        // Handle different types of requests based on the `request_type`
        match request_type {
            RequestType::FetchUpdatedCSS => {
                tracing::debug!("Request type: FetchUpdatedCSS");

                // Return a request to fetch updated CSS for the given client
                return Ok(ServerRequest::new(client_name, Request::FetchUpdatedCSS));
            }
            RequestType::CollectClassList => {
                // If the request type is `CollectClassList`, ensure there are at least 3 tokens
                Self::token_less_than(
                    3,
                    &tokens,
                    ErrorKind::MissingRequestTokens,
                    ErrorAction::Ignore,
                    "The integration client submitted a 'collect-class-list' request missing an additional token for class details. Ensure at least 3 tokens are present.",
                )?;

                let class_token = tokens[2].clone();

                tracing::debug!(
                    "Request type: CollectClassList with class token: {}",
                    class_token
                );

                // Call a separate function to handle the class list request
                return Self::build_collect_class_list_request(class_token, client_name);
            }
        }
    }

    /// Builds a `ServerRequest` for collecting a class list based on the provided class token and client name.
    ///
    /// # Parameters
    /// - `class_token`: A string containing the class token, expected to be in the format `context_type:class_name`.
    /// - `client_name`: The name of the client making the request.
    ///
    /// # Returns
    /// A `GaladrielResult<ServerRequest>`. Returns an `Ok(ServerRequest)` if the class token format is valid, or an error if the format is incorrect.
    fn build_collect_class_list_request(
        class_token: String,
        client_name: String,
    ) -> GaladrielResult<ServerRequest> {
        // Split the class token by colon (:) into a vector
        let target_class: Vec<String> = class_token.split(":").map(|v| v.to_owned()).collect();

        // Ensure there are at least two tokens in the class token: `context_type` and `class_name`
        Self::token_less_than(
            2,
            &target_class,
            ErrorKind::MissingRequestTokens,
            ErrorAction::Ignore,
            "The integration client submitted a request with an invalid class token format. Expected at least 2 tokens but received fewer. Provide both a `context type` and a `class name`, separated by a colon (`:`).",
        )?;

        tracing::debug!(
            "Building collect class list request with target class: {:?}",
            target_class
        );

        // Extract the context type from the first token
        let context_type = Self::get_context_type(&target_class[0])?;

        // Handle different context types for the class request
        match context_type {
            ContextType::Central => {
                // For `Central` context, the class name is the second token
                let class_name = target_class[1].clone();
                let request = Request::new_class_list_request(context_type, None, class_name);

                tracing::debug!("Context type: Central");

                // Return the request for the Central context
                return Ok(ServerRequest::new(client_name, request));
            }
            _ => {
                // For non-Central contexts, ensure there are at least 3 tokens: `context_type`, `context_name`, and `class_name`
                Self::token_less_than(
                    3,
                    &target_class,
                    ErrorKind::MissingRequestTokens,
                    ErrorAction::Ignore,
                    "The integration client submitted a request in an invalid format for a non-Central context. Expected format: `context type`, `context name`, and `class name`, separated by colons (`:`). Ensure the request contains at least three tokens in the correct format.",
                )?;

                // Extract the `context_name` and `class_name` for non-Central contexts
                let context_name = target_class[1].clone();
                let class_name = target_class[2].clone();

                tracing::debug!(
                    "Context type: {:?}, context name: {}, class name: {}",
                    context_type,
                    context_name,
                    class_name
                );

                // Build and return the request for non-Central contexts
                let request =
                    Request::new_class_list_request(context_type, Some(context_name), class_name);

                return Ok(ServerRequest::new(client_name, request));
            }
        }
    }

    /// Validates that the number of tokens is not less than the specified threshold.
    ///
    /// # Parameters
    /// - `less_than`: The minimum required number of tokens.
    /// - `tokens`: A vector containing the tokens to validate.
    /// - `error_kind`: The kind of error to raise if the validation fails.
    /// - `error_action`: The action to take if the validation fails.
    /// - `error_message`: A detailed message explaining the validation failure.
    ///
    /// # Returns
    /// - `GaladrielResult<()>`: An `Ok` result if validation passes, or an error if validation fails.
    fn token_less_than(
        less_than: usize,
        tokens: &Vec<String>,
        error_kind: ErrorKind,
        error_action: ErrorAction,
        error_message: &str,
    ) -> GaladrielResult<()> {
        if tokens.len() < less_than {
            tracing::error!("The integration client submitted a request with an invalid format: Expected at least {less_than} tokens but received fewer. Tokens: {:?}", tokens);

            // Return an error with detailed explanation if fewer than `less_than` tokens are provided
            return Err(GaladrielError::raise_general_pipeline_error(
                error_kind,
                error_message,
                error_action,
            ));
        }

        Ok(())
    }

    /// Retrieves the request type based on the provided `request_token`.
    ///
    /// # Arguments
    ///
    /// * `request_token` - A string slice that represents the request token.
    ///
    /// # Returns
    ///
    /// This function returns a `GaladrielResult<RequestType>`. If the token is valid,
    /// it returns the corresponding `RequestType` variant. Otherwise, it returns an error.
    ///
    /// # Errors
    ///
    /// Returns a `GaladrielError` if the request token is invalid.
    fn get_request_type(request_token: &str) -> GaladrielResult<RequestType> {
        match request_token {
            // If the token matches "collect-class-list", return `RequestType::CollectClassList`.
            "collect-class-list" => {
                tracing::debug!("Request token: collect-class-list");

                Ok(RequestType::CollectClassList)
            }
            // If the token matches "fetch-updated-css", return `RequestType::FetchUpdatedCSS`.
            "fetch-updated-css" => {
                tracing::debug!("Request token: fetch-updated-css");

                Ok(RequestType::FetchUpdatedCSS)
            }
            // If the token is neither of the above, log an error and return a pipeline error.
            _ => {
                tracing::error!("The integration client submitted an invalid request token: '{}'. Expected one of 'collect-class-list' or 'fetch-updated-css'.", request_token);

                return Err(GaladrielError::raise_general_pipeline_error(
                    ErrorKind::RequestTokenInvalid,
                    &format!("The integration client submitted an invalid request token: '{}'. Expected one of 'collect-class-list' or 'fetch-updated-css'.", request_token),
                    ErrorAction::Ignore,
                ));
            }
        }
    }

    /// Retrieves the context type based on the provided `context_token`.
    ///
    /// # Arguments
    ///
    /// * `context_token` - A string slice that represents the context token.
    ///
    /// # Returns
    ///
    /// This function returns a `GaladrielResult<ContextType>`. If the token is valid,
    /// it returns the corresponding `ContextType` variant. Otherwise, it returns an error.
    ///
    /// # Errors
    ///
    /// Returns a `GaladrielError` if the context token is invalid.
    fn get_context_type(context_token: &str) -> GaladrielResult<ContextType> {
        match context_token {
            // If the token matches "@class", return `ContextType::Central`.
            "@class" => {
                tracing::debug!("Context token: @class");

                Ok(ContextType::Central)
            }
            // If the token matches "@layout", return `ContextType::Layout`.
            "@layout" => {
                tracing::debug!("Context token: @layout");

                Ok(ContextType::Layout)
            }
            // If the token matches "@module", return `ContextType::Module`.
            "@module" => {
                tracing::debug!("Context token: @module");

                Ok(ContextType::Module)
            }
            // If the token is neither of the above, log an error and return a pipeline error.
            _ => {
                tracing::error!("The integration client submitted an invalid context token: '{}'. Expected one of '@class', '@layout', or '@module'.", context_token);

                return Err(GaladrielError::raise_general_pipeline_error(
                    ErrorKind::RequestTokenInvalid,
                    &format!("The integration client submitted an invalid context token: '{}'. Expected one of '@class', '@layout', or '@module'.", context_token),
                    ErrorAction::Ignore,
                ));
            }
        }
    }

    fn process_request(request: ServerRequest) -> String {
        let _client_name = request.client_name;

        match request.request {
            Request::CollectClassList {
                context_type,
                context_name,
                class_name,
            } => {
                return get_utility_class_names(context_type, context_name, class_name);
            }
            Request::FetchUpdatedCSS => {
                return get_updated_css();
            }
            _ => {}
        }

        String::new()
    }

    /// Sends a success notification to Palantir with details about the operation's duration.
    ///
    /// # Parameters
    /// - `message`: A success message describing the operation.
    /// - `starting_time`: The timestamp when the operation started.
    /// - `palantir_sender`: Sender used to broadcast the success notification.
    fn send_palantir_success_notification(
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
        Self::send_palantir_notification(notification, palantir_sender.clone());
    }

    /// Sends an error notification to Palantir with details about the encountered error.
    ///
    /// # Parameters
    /// - `error`: The error encountered during the operation.
    /// - `starting_time`: The timestamp when the operation started.
    /// - `palantir_sender`: Sender used to broadcast the error notification.
    fn send_palantir_error_notification(
        error: GaladrielError,
        starting_time: DateTime<Local>,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) {
        tracing::error!("Error occurred during operation. Details: {:?}", error);

        // Create an error notification with the starting timestamp and error details.
        let notification = GaladrielAlerts::create_galadriel_error(starting_time, error);

        // Send the error notification.
        Self::send_palantir_notification(notification, palantir_sender.clone());
    }

    /// Sends a notification to Palantir. Handles errors if the notification fails to send.
    ///
    /// # Parameters
    /// - `notification`: The notification to be sent.
    /// - `palantir_sender`: Sender used to broadcast the notification.
    fn send_palantir_notification(
        notification: GaladrielAlerts,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) {
        // Attempt to send the notification. Log an error if it fails.
        if let Err(err) = palantir_sender.send(notification) {
            tracing::error!("Failed to send alert: {:?}", err);
        }
    }

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

#[cfg(test)]
mod tests {
    use tokio::sync::broadcast;
    use tungstenite::Message;

    use crate::{
        error::ErrorKind,
        lothlorien::request::{ContextType, Request, RequestType},
    };

    use super::LothlorienPipeline;

    use std::env;

    #[tokio::test]
    async fn test_initialization() {
        let (sender, _) = broadcast::channel(10);
        let pipeline = LothlorienPipeline::new("8080".to_string(), sender);

        // Check if fields are initialized as expected
        assert_eq!(pipeline.socket_addr, "127.0.0.1:8080");
        assert_eq!(pipeline.systems_temp_folder, env::temp_dir());
    }

    #[tokio::test]
    async fn test_register_server_port_in_temp() {
        let (sender, _) = broadcast::channel(10);
        let pipeline = LothlorienPipeline::new("8080".to_string(), sender);

        // Register the server port in the temporary file
        let port = 8080;

        assert_eq!(
            format!("{:?}", pipeline.register_server_port_in_temp(port)),
            "Ok(())".to_string()
        );
    }

    #[tokio::test]
    async fn test_remove_server_port_in_temp() {
        let (sender, _) = broadcast::channel(10);
        let pipeline = LothlorienPipeline::new("8080".to_string(), sender);

        // Check if the file was removed
        assert_eq!(
            format!("{:?}", pipeline.remove_server_port_in_temp()),
            "Ok(())".to_string()
        );
    }

    #[test]
    fn test_process_response_valid_fetch_updated_css() {
        let message = Message::Text("fetch-updated-css;ClientA".to_string());
        let result = LothlorienPipeline::process_response(&message);

        assert!(result.is_ok());

        if let Ok(server_request) = result {
            assert_eq!(server_request.client_name, "ClientA");

            match server_request.request {
                Request::FetchUpdatedCSS => {}
                _ => panic!("Expected FetchUpdatedCSS request type"),
            }
        }
    }

    #[test]
    fn test_process_response_invalid_format_missing_token() {
        let message = Message::Text("fetch-updated-css".to_string());
        let result = LothlorienPipeline::process_response(&message);

        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.get_kind(), ErrorKind::MissingRequestTokens);
        }
    }

    #[test]
    fn test_process_response_valid_collect_class_list() {
        let message =
            Message::Text("collect-class-list;ClientB;@module:context_name:class_name".to_string());
        let result = LothlorienPipeline::process_response(&message);

        assert!(result.is_ok());

        if let Ok(server_request) = result {
            assert_eq!(server_request.client_name, "ClientB");

            match server_request.request {
                Request::CollectClassList {
                    context_type,
                    context_name,
                    class_name,
                } => {
                    assert_eq!(context_type, ContextType::Module);
                    assert_eq!(context_name, Some("context_name".to_string()));
                    assert_eq!(class_name, "class_name");
                }
                _ => panic!("Expected CollectClassList request type"),
            }
        }
    }

    #[test]
    fn test_process_response_invalid_collect_class_list_format() {
        let message = Message::Text("collect-class-list;ClientC".to_string());
        let result = LothlorienPipeline::process_response(&message);

        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.get_kind(), ErrorKind::MissingRequestTokens);
        }
    }

    #[test]
    fn test_build_collect_class_list_request_valid_central() {
        let result = LothlorienPipeline::build_collect_class_list_request(
            "@class:class_name".to_string(),
            "ClientD".to_string(),
        );

        assert!(result.is_ok());

        if let Ok(server_request) = result {
            assert_eq!(server_request.client_name, "ClientD");

            match server_request.request {
                Request::CollectClassList {
                    context_type,
                    context_name: _,
                    class_name,
                } => {
                    assert_eq!(context_type, ContextType::Central);
                    assert_eq!(class_name, "class_name");
                }
                _ => panic!("Expected CollectClassList request type"),
            }
        }
    }

    #[test]
    fn test_build_collect_class_list_request_invalid_class_token() {
        let result = LothlorienPipeline::build_collect_class_list_request(
            "central".to_string(),
            "ClientE".to_string(),
        );

        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.get_kind(), ErrorKind::MissingRequestTokens);
        }
    }

    #[test]
    fn test_build_collect_class_list_request_non_central() {
        let result = LothlorienPipeline::build_collect_class_list_request(
            "@layout:context_name:class_name".to_string(),
            "ClientF".to_string(),
        );

        assert!(result.is_ok());

        if let Ok(server_request) = result {
            assert_eq!(server_request.client_name, "ClientF");

            match server_request.request {
                Request::CollectClassList {
                    context_type,
                    context_name,
                    class_name,
                } => {
                    assert_eq!(context_type, ContextType::Layout);
                    assert_eq!(context_name, Some("context_name".to_string()));
                    assert_eq!(class_name, "class_name");
                }
                _ => panic!("Expected CollectClassList request type"),
            }
        }
    }

    #[test]
    fn test_build_collect_class_list_request_invalid_non_central() {
        let result = LothlorienPipeline::build_collect_class_list_request(
            "context_type:context_name".to_string(),
            "ClientG".to_string(),
        );

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.get_kind(), ErrorKind::RequestTokenInvalid);
        }
    }

    #[test]
    fn test_get_request_type_valid_collect_class_list() {
        let result = LothlorienPipeline::get_request_type("collect-class-list");
        assert_eq!(result, Ok(RequestType::CollectClassList));
    }

    #[test]
    fn test_get_request_type_valid_fetch_updated_css() {
        let result = LothlorienPipeline::get_request_type("fetch-updated-css");
        assert_eq!(result, Ok(RequestType::FetchUpdatedCSS));
    }

    #[test]
    fn test_get_request_type_invalid_token() {
        let result = LothlorienPipeline::get_request_type("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_context_type_valid_class() {
        let result = LothlorienPipeline::get_context_type("@class");
        assert_eq!(result, Ok(ContextType::Central));
    }

    #[test]
    fn test_get_context_type_valid_layout() {
        let result = LothlorienPipeline::get_context_type("@layout");
        assert_eq!(result, Ok(ContextType::Layout));
    }

    #[test]
    fn test_get_context_type_valid_module() {
        let result = LothlorienPipeline::get_context_type("@module");
        assert_eq!(result, Ok(ContextType::Module));
    }

    #[test]
    fn test_get_context_type_invalid_token() {
        let result = LothlorienPipeline::get_context_type("invalid-context");
        assert!(result.is_err());
    }
}
