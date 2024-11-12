use std::{env, fs, path::PathBuf};

use chrono::Local;
use events::{ClientResponse, ConnectedClientEvents};
use futures::{SinkExt, StreamExt};
use rand::Rng;
use request::{ContextType, Request, RequestType, ServerRequest};
use tokio::{
    net::TcpListener,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task::JoinHandle,
};
use tokio_tungstenite::accept_async;
use tracing::{debug, error, info, warn};
use tungstenite::Message;

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielEvents,
    shellscape::notifications::ShellscapeNotifications,
    GaladrielResult,
};

pub mod events;
mod request;

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
    runtime_sender: broadcast::Sender<ConnectedClientEvents>,
    // Receiver for connected client events
    runtime_receiver: broadcast::Receiver<ConnectedClientEvents>,

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
    ///
    /// # Returns
    ///
    /// A new `LothlorienPipeline` instance.
    pub fn new(port: String) -> Self {
        // Create unbounded channels for pipeline and runtime communication
        let (pipeline_sender, pipeline_receiver) = mpsc::unbounded_channel();
        let (runtime_sender, runtime_receiver) = broadcast::channel(2);

        info!("Initializing LothlorienPipeline with port: {}", port);

        Self {
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
        info!("Attempting to bind to socket address: {}", self.socket_addr);

        TcpListener::bind(self.socket_addr.clone())
            .await
            .map_err(|err| {
                error!(
                    "Failed to bind to socket address '{}': {:?}",
                    self.socket_addr, err
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
        // Clone the pipeline and runtime senders to be used inside the spawned task
        let _sender = self.pipeline_sender.clone();
        let _runtime_sender = self.runtime_sender.clone();

        let start_time = Local::now();
        let ending_time = Local::now();
        let duration = ending_time - start_time;

        let notification = ShellscapeNotifications::create_success(
            start_time,
            ending_time,
            duration,
            &random_server_subheading_message(),
        );

        if let Err(err) = _sender.send(GaladrielEvents::Notify(notification)) {
            error!("Failed to send the server's header message to the main runtime. Error details: {:?}", err);
        }

        info!("Starting pipeline listener. Awaiting client connections...");

        // Spawn a new asynchronous task to handle incoming connections
        tokio::spawn(async move {
            // Infinite loop to continuously accept and process client connections
            loop {
                tokio::select! {
                    // If the pipeline sender is closed, log a warning and gracefully exit
                    _ = _sender.closed() => {
                        warn!("Pipeline sender has been closed. Shutting down listener gracefully.");

                        break;
                    }
                    // Continuously accept incoming client connections
                    connection = listener.accept() => {
                        match connection {
                            // If connection is successfully accepted, spawn a new task to handle the stream
                            Ok((stream, _)) => {
                                info!("Accepted new connection from client.");

                                let _pipeline_sender = _sender.clone();
                                let _runtime_sender = _runtime_sender.clone();

                                // Spawn a new task to handle the client stream
                                let result = tokio::spawn(async move {
                                    Self::stream_sync(stream, _pipeline_sender, _runtime_sender).await
                                }).await;

                                // Handle the result of the stream handling task
                                match result {
                                    // If the stream is handled successfully, log the success
                                    Ok(Ok(_)) => {
                                        info!("Connection handled successfully.");

                                        let notification = ShellscapeNotifications::create_information(
                                            Local::now(),
                                            "Client has successfully disconnected from the Galadriel CSS server. No further events will be sent to this client."
                                        );

                                        if let Err(err) = _sender.send(GaladrielEvents::Notify(notification)) {
                                            error!(
                                                "Failed to send client disconnection notification to the main runtime. Error details: {:?}",
                                                err
                                            );
                                        }
                                    }
                                    // If an error occurs while processing the connection, log the error
                                    Ok(Err(err)) => {
                                        error!("Error occurred while processing client connection: {:?}", err);

                                        // Send an error notification to the pipeline sender
                                        if let Err(err) = _sender.send(GaladrielEvents::Error(err)) {
                                            error!("Failed to notify the main runtime about error: {:?}", err);
                                        }
                                    }
                                    // If an unexpected error occurs, log the error and notify the pipeline
                                    Err(err) => {
                                        let err = GaladrielError::raise_general_pipeline_error(
                                            ErrorKind::ConnectionTerminationError,
                                            &err.to_string(),
                                            ErrorAction::Notify
                                        );

                                        error!("Unexpected error in handling connection: {:?}", err);

                                        // Send an error notification to the pipeline sender
                                        if let Err(err) = _sender.send(GaladrielEvents::Error(err)) {
                                            error!("Failed to notify the main runtime about error: {:?}", err);
                                        }
                                    }
                                }
                            }
                            // If an error occurs while accepting the connection, log the error and notify the pipeline
                            Err(err) => {
                                let err = GaladrielError::raise_general_pipeline_error(
                                    ErrorKind::ConnectionInitializationError,
                                    &err.to_string(),
                                    ErrorAction::Notify
                                );

                                error!("Failed to accept incoming client connection: {:?}", err);

                                // Send an error notification to the pipeline sender
                                if let Err(err) = _sender.send(GaladrielEvents::Error(err)) {
                                    error!("Failed to notify the main runtime about error: {:?}", err);
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    /// Receives the next event from the pipeline receiver.
    ///
    /// # Returns
    ///
    /// A result containing either the received event or an error.
    pub async fn next(&mut self) -> GaladrielResult<GaladrielEvents> {
        self.pipeline_receiver.recv().await.ok_or_else(|| {
            error!("Failed to receive Lothlórien pipeline event: Channel closed unexpectedly or an IO error occurred");

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
    /// The `broadcast::Sender<ConnectedClientEvents>` for sending events.
    pub fn get_runtime_sender(&self) -> broadcast::Sender<ConnectedClientEvents> {
        info!("Retrieving runtime sender.");
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

        let systems_temp_file = self
            .systems_temp_folder
            .join("galadrielcss_lothlorien_pipeline_port.txt");

        let mut file = fs::File::create(&systems_temp_file).map_err(|err| {
            GaladrielError::raise_general_pipeline_error(
                ErrorKind::ServerPortRegistrationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        info!(
            "Registering server port {} in temporary file: {:?}",
            port, systems_temp_file
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
        let systems_temp_file = self
            .systems_temp_folder
            .join("galadrielcss_lothlorien_pipeline_port.txt");

        if systems_temp_file.exists() {
            info!(
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
            warn!(
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
        runtime_sender: broadcast::Sender<ConnectedClientEvents>,
    ) -> GaladrielResult<()> {
        // Establishes a WebSocket connection and splits it into sender and receiver components.
        let (mut stream_sender, mut stream_receiver) = accept_async(stream)
            .await
            .map_err(|err| {
                error!(
                    "Failed to establish WebSocket connection with client stream: {:?}",
                    err
                );

                GaladrielError::raise_critical_pipeline_error(
                    ErrorKind::ServerSyncAcceptFailed,
                    &err.to_string(),
                    ErrorAction::Notify,
                )
            })?
            .split();

        // Subscribes to the runtime sender to receive events.
        let mut runtime_receiver = runtime_sender.subscribe();
        let message = "Galadriel CSS server is ready! 🚀\n\nYou can start styling your project with Galadriel CSS and see instant updates as changes are made.\n\nHappy coding, and may your styles be ever beautiful!";

        info!("Successfully established WebSocket connection. Sending initial greeting message to client.");

        // Sends the initial greeting message to the client and handles any errors during the send operation.
        stream_sender
            .send(Message::Text(message.to_string()))
            .await
            .map_err(|err| {
                error!(
                    "Failed to send initial greeting to client. Error: {:?}",
                    err
                );

                GaladrielError::raise_general_pipeline_error(
                    ErrorKind::NotificationSendError,
                    &format!("Initial message send failed: {}", err.to_string()),
                    ErrorAction::Notify,
                )
            })?;

        let notification = ShellscapeNotifications::create_information(
            Local::now(),
            "A new client has successfully connected to the Galadriel server and is now ready to request and receive events."
        );

        if let Err(err) = pipeline_sender.send(GaladrielEvents::Notify(notification)) {
            error!(
                "Failed to send client connection notification to main runtime. Error details: {:?}",
                err
            );
        }

        loop {
            tokio::select! {
                // If the pipeline sender is closed, the loop terminates gracefully.
                _ = pipeline_sender.closed() => {
                    warn!("Pipeline sender has been closed. Closing the stream synchronization process.");

                    break;
                }
                // Receives events from the runtime system.
                _runtime_res = runtime_receiver.recv() => {}
                // Receives events from the connected integration client.
                client_response = stream_receiver.next() => {
                    match Self::handle_stream_response(&client_response, pipeline_sender.clone()) {
                        ClientResponse::Break  => {
                            break;
                        }
                        ClientResponse::Text(data) => {
                            if let Err(err) = stream_sender.send(Message::Text(data)).await {
                                let err = GaladrielError::raise_general_pipeline_error(
                                    ErrorKind::NotificationSendError,
                                    &err.to_string(),
                                    ErrorAction::Notify
                                );

                                error!(
                                    "Failed to send data to the connected client. Original error: {:?}",
                                    err
                                );

                                let notification = ShellscapeNotifications::create_galadriel_error(Local::now(), err);

                                if let Err(err) = pipeline_sender.send(GaladrielEvents::Notify(notification)) {
                                    error!(
                                        "Failed to send error notification to the main runtime. Error: {:?}",
                                        err
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_stream_response(
        client_response: &Option<Result<Message, tungstenite::Error>>,
        pipeline_sender: UnboundedSender<GaladrielEvents>,
    ) -> ClientResponse {
        match client_response {
            // If no more data is received, the client has disconnected.
            None => {
                info!("Client has disconnected. Terminating stream synchronization.");

                return ClientResponse::Break;
            }
            // If an error occurs while receiving the client's message, handle it.
            Some(Err(err)) => {
                error!(
                    "Error receiving message from client: {:?}. Disconnecting client.",
                    err
                );

                // Raises a general pipeline error and notifies the runtime of the error.
                let err = GaladrielError::raise_general_pipeline_error(
                    ErrorKind::ClientResponseError,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                if let Err(err) = pipeline_sender.send(GaladrielEvents::Error(err)) {
                    error!("Failed to notify runtime of error: {:?}", err);
                }

                return ClientResponse::Break;
            }
            // Successfully received a message from the client.
            Some(Ok(event)) => {
                info!("Received event from client: {:?}", event);

                // Processes the received event and handles the associated request.
                match Self::process_response(event) {
                    Ok(ServerRequest { request, .. }) if request == Request::BreakConnection => {
                        return ClientResponse::Break;
                    }
                    Ok(request) => {
                        // Handles the processed request and sends notifications to the runtime.
                        let data = Self::process_request(request);

                        info!("Successfully processed request: {:?}", data);

                        return ClientResponse::Text(data);
                    }
                    // Handles errors while processing the response.
                    Err(err) => {
                        error!(
                            "An error occurred while processing the client's request: {:?}",
                            err
                        );

                        // Notifies the runtime of the error during request processing.
                        if let Err(err) = pipeline_sender.send(GaladrielEvents::Error(err)) {
                            error!(
                                "Failed to notify runtime of error during request processing: {:?}",
                                err
                            );
                        }

                        return ClientResponse::Continue;
                    }
                }
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
            Message::Text(response) => {
                // Split the response string by semicolon and collect into a vector of strings
                let tokens: Vec<String> = response.split(";").map(|v| v.to_owned()).collect();

                // Ensure there are at least two tokens: `request type` and `client name`
                if tokens.len() < 2 {
                    error!("Invalid request format: Expected at least 2 tokens but received fewer. Tokens: {:?}", tokens);

                    // Return an error with detailed explanation if fewer than 2 tokens are provided
                    return Err(GaladrielError::raise_general_pipeline_error(
                        ErrorKind::MissingRequestTokens,
                        "Invalid request format: Expected at least 2 tokens but received fewer. Please provide both a `request type` and a `client name`, separated by a semicolon `;`. For example: `fetch-updated-css;Client Name`.",
                        ErrorAction::Ignore,
                    ));
                }

                debug!("Processing request with tokens: {:?}", tokens);

                // Extract the `request type` and `client name` from the tokens
                let request_type = Self::get_request_type(&tokens[0])?;
                let client_name = tokens[1].clone();

                // Handle different types of requests based on the `request_type`
                match request_type {
                    RequestType::FetchUpdatedCSS => {
                        debug!("Request type: FetchUpdatedCSS");

                        // Return a request to fetch updated CSS for the given client
                        return Ok(ServerRequest::new(client_name, Request::FetchUpdatedCSS));
                    }
                    RequestType::CollectClassList => {
                        // If the request type is `CollectClassList`, ensure there are at least 3 tokens
                        if tokens.len() < 3 {
                            error!("Collect class list request requires an additional token for class details. Tokens: {:?}", tokens);

                            // Return an error if the class token is missing
                            return Err(GaladrielError::raise_general_pipeline_error(
                                ErrorKind::MissingRequestTokens,
                                "Collect class list request requires an additional token for class details. Ensure at least 3 tokens are present.",
                                ErrorAction::Ignore,
                            ));
                        }

                        let class_token = tokens[2].clone();

                        debug!(
                            "Request type: CollectClassList with class token: {}",
                            class_token
                        );

                        // Call a separate function to handle the class list request
                        return Self::build_collect_class_list_request(class_token, client_name);
                    }
                }
            }
            Message::Close(_) => {
                return Ok(ServerRequest::new("".to_string(), Request::BreakConnection));
            }
            _ => {}
        }

        error!("Unsupported message format received. Only `Message::Text` is supported for processing.");

        // Return an error for unsupported message types
        Err(GaladrielError::raise_general_pipeline_error(
            ErrorKind::UnsupportedRequestToken,
            "Unsupported message format received. Only `Message::Text` is supported for processing.",
            ErrorAction::Ignore,
        ))
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
        if target_class.len() < 2 {
            error!("Invalid class token format: Expected at least 2 tokens but received fewer. Token: {}", class_token);

            // Return an error if the class token format is invalid
            return Err(GaladrielError::raise_general_pipeline_error(
                ErrorKind::MissingRequestTokens,
                "Invalid class token format: Expected at least 2 tokens but received fewer. Provide both a `context type` and a `class name`, separated by a colon `:`.",
                ErrorAction::Ignore,
            ));
        }

        debug!(
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

                debug!("Context type: Central");

                // Return the request for the Central context
                return Ok(ServerRequest::new(client_name, request));
            }
            _ => {
                // For non-Central contexts, ensure there are at least 3 tokens: `context_type`, `context_name`, and `class_name`
                if target_class.len() < 3 {
                    error!("Invalid format for non-Central context: Expected `context type`, `context name`, and `class name`, separated by colons `:`. Tokens: {:?}", target_class);

                    // Return an error if the format is invalid for non-Central contexts
                    return Err(GaladrielError::raise_general_pipeline_error(
                        ErrorKind::MissingRequestTokens,
                        "Invalid format for non-Central context: Expected `context type`, `context name`, and `class name`, separated by colons `:`. Ensure at least 3 tokens are provided.",
                        ErrorAction::Ignore,
                    ));
                }

                // Extract the `context_name` and `class_name` for non-Central contexts
                let context_name = target_class[1].clone();
                let class_name = target_class[2].clone();

                debug!(
                    "Context type: {:?}, context name: {}, class name: {}",
                    context_type, context_name, class_name
                );

                // Build and return the request for non-Central contexts
                let request =
                    Request::new_class_list_request(context_type, Some(context_name), class_name);

                return Ok(ServerRequest::new(client_name, request));
            }
        }
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
                debug!("Request token: collect-class-list");

                Ok(RequestType::CollectClassList)
            }
            // If the token matches "fetch-updated-css", return `RequestType::FetchUpdatedCSS`.
            "fetch-updated-css" => {
                debug!("Request token: fetch-updated-css");

                Ok(RequestType::FetchUpdatedCSS)
            }
            // If the token is neither of the above, log an error and return a pipeline error.
            _ => {
                error!("Invalid request token: '{}'. Expected one of 'collect-class-list' or 'fetch-updated-css'.", request_token);

                return Err(GaladrielError::raise_general_pipeline_error(
                    ErrorKind::RequestTokenInvalid,
                    &format!("Invalid request token: '{}'. Expected one of 'collect-class-list' or 'fetch-updated-css'.", request_token),
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
                debug!("Context token: @class");

                Ok(ContextType::Central)
            }
            // If the token matches "@layout", return `ContextType::Layout`.
            "@layout" => {
                debug!("Context token: @layout");

                Ok(ContextType::Layout)
            }
            // If the token matches "@module", return `ContextType::Module`.
            "@module" => {
                debug!("Context token: @module");

                Ok(ContextType::Module)
            }
            // If the token is neither of the above, log an error and return a pipeline error.
            _ => {
                error!("Invalid context token: '{}'. Expected one of '@class', '@layout', or '@module'.", context_token);

                return Err(GaladrielError::raise_general_pipeline_error(
                    ErrorKind::RequestTokenInvalid,
                    &format!("Invalid context token: '{}'. Expected one of '@class', '@layout', or '@module'.", context_token),
                    ErrorAction::Ignore,
                ));
            }
        }
    }

    fn process_request(request: ServerRequest) -> String {
        let _client_name = request.client_name;

        match request.request {
            Request::CollectClassList {
                context_type: _,
                context_name: _,
                class_name: _,
            } => {}
            Request::FetchUpdatedCSS => {
                // TODO: Implement a caching mechanism for the CSS, so that it is only updated when
                // TODO: changes to the CSS AST occur, reducing unnecessary re-renders and improving performance.
            }
            _ => {}
        }

        "some-data".to_string()
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

    debug!("Selected random subheading message: {}", selected_message);

    selected_message
}

#[cfg(test)]
mod tests {
    use tungstenite::Message;

    use crate::{
        error::ErrorKind,
        lothlorien::request::{ContextType, Request, RequestType},
    };

    use super::LothlorienPipeline;

    use std::env;

    #[tokio::test]
    async fn test_initialization() {
        let pipeline = LothlorienPipeline::new("8080".to_string());

        // Check if fields are initialized as expected
        assert_eq!(pipeline.socket_addr, "127.0.0.1:8080");
        assert_eq!(pipeline.systems_temp_folder, env::temp_dir());
    }

    #[tokio::test]
    async fn test_register_server_port_in_temp() {
        let pipeline = LothlorienPipeline::new("8080".to_string());

        // Register the server port in the temporary file
        let port = 8080;

        assert_eq!(
            format!("{:?}", pipeline.register_server_port_in_temp(port)),
            "Ok(())".to_string()
        );
    }

    #[tokio::test]
    async fn test_remove_server_port_in_temp() {
        let pipeline = LothlorienPipeline::new("8080".to_string());

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
