use std::{env, fs, path::PathBuf};

use futures::{SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task::JoinHandle,
};
use tokio_tungstenite::accept_async;
use tracing::{error, info, warn};
use tungstenite::Message;

use crate::{GaladrielFuture, GaladrielResult};

#[derive(Clone, PartialEq, Debug)]
pub enum LothlorienEvents {}

#[derive(Clone, PartialEq, Debug)]
pub enum ConnectedClientEvents {}

/// Represents a pipeline for managing events and connections in the Lothlórien system.
/// It manages communication through a set of channels and listeners, enabling the processing of events
/// between the pipeline and the runtime environment.
#[allow(dead_code)]
#[derive(Debug)]
pub struct LothlorienPipeline {
    // Sender for pipeline events
    pipeline_sender: UnboundedSender<LothlorienEvents>,
    // Receiver for pipeline events
    pipeline_receiver: UnboundedReceiver<LothlorienEvents>,

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

                Box::<dyn std::error::Error>::from(err.to_string())
            })
    }

    /// Starts the pipeline listener in a new asynchronous task.
    ///
    /// # Arguments
    ///
    /// * `listener` - The `TcpListener` to use for accepting client connections.
    ///
    /// # Returns
    ///
    /// A `JoinHandle<()>` to allow monitoring of the task.
    pub fn create_pipeline(&self, listener: TcpListener) -> JoinHandle<()> {
        let _sender = self.pipeline_sender.clone();
        let _runtime_sender = self.runtime_sender.clone();

        info!("Starting pipeline listener...");

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Check if the pipeline sender has been closed
                    _ = _sender.closed() => {
                        warn!("Pipeline sender closed. Shutting down listener.");

                        break;
                    }
                    // Continuously accept connections from the Lothlórien pipeline listener.
                    connection = listener.accept() => {
                        match connection {
                            Ok((stream, _)) => {
                                info!("Accepted new connection from client.");

                                let _pipeline_sender = _sender.clone();
                                let _runtime_sender = _runtime_sender.clone();

                                let result = tokio::spawn(async move {
                                    Self::stream_sync(stream, _pipeline_sender, _runtime_sender).await
                                }).await;

                                match result {
                                    Ok(Ok(_)) => {}
                                    Ok(Err(err)) => {
                                        // TODO: Send the error to the runtime.
                                        error!("Error handling connection: {:?}", err);
                                    }
                                    Err(err) => {
                                        // TODO: Send the error to the runtime.
                                        error!("Error handling connection: {:?}", err);
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Failed to accept client connection: {:?}", err);
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
    pub async fn next(&mut self) -> GaladrielResult<LothlorienEvents> {
        self.pipeline_receiver.recv().await.ok_or_else(|| {
            error!("Failed to receive Lothlórien pipeline event: Channel closed unexpectedly or an IO error occurred");

            Box::<dyn std::error::Error>::from("Error while receiving response from Lothlórien pipeline sender: No response received.")
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
        let mut file = fs::File::create(&systems_temp_file)?;

        info!(
            "Registering server port {} in temporary file: {:?}",
            port, systems_temp_file
        );

        write!(file, "{}", port)?;

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
            fs::remove_file(systems_temp_file)?;
        } else {
            warn!(
                "Server port registration file does not exist: {:?}",
                systems_temp_file
            );
        }

        Ok(())
    }

    /// Handles the synchronization of streams for each connected client.
    /// It reads data from the client and sends events to the pipeline and runtime as needed.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream to handle.
    /// * `pipeline_sender` - The sender for sending pipeline events.
    /// * `runtime_sender` - The sender for sending client events to the runtime.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    async fn stream_sync(
        stream: tokio::net::TcpStream,
        pipeline_sender: UnboundedSender<LothlorienEvents>,
        runtime_sender: broadcast::Sender<ConnectedClientEvents>,
    ) -> GaladrielFuture<()> {
        let (mut stream_sender, mut stream_receiver) = accept_async(stream).await?.split();
        let mut runtime_receiver = runtime_sender.subscribe();

        let message = Message::Text("Galadriel CSS server is ready! 🚀\n\nYou can start styling your project with Galadriel CSS and see instant updates as changes are made.\n\nHappy coding, and may your styles be ever beautiful!".to_string());
        info!("Sending initial connection message to client.");
        stream_sender.send(message).await?;

        loop {
            tokio::select! {
                // End the loop if the Lothlórien receiver is closed.
                _ = pipeline_sender.closed() => {
                    warn!("Pipeline sender closed. Terminating stream synchronization.");
                    break;
                }
                _runtime_res = runtime_receiver.recv() => {}
                // Receives events from the connected user.
                client_res = stream_receiver.next() => {
                    match client_res {
                        None => {
                            info!("Client disconnected. Ending synchronization.");
                            break;
                        }
                        Some(Err(err)) => {
                            error!("Error receiving event from client: {:?}", err);
                            // TODO: Send the error to the runtime.

                            break;
                        }
                        Some(Ok(event)) => {
                            info!("Received event from client: {:?}", event);
                            // TODO: Process the received event.
                            // TODO: Send the received event to the main runtime via sender if necessary.
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
}
