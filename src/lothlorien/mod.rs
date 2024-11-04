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
use tracing::error;
use tungstenite::Message;

use crate::{GaladrielFuture, GaladrielResult};

#[derive(Clone, PartialEq, Debug)]
pub enum LothlorienEvents {}

#[derive(Clone, PartialEq, Debug)]
pub enum ConnectedClientEvents {}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LothlorienPipeline {
    pipeline_sender: UnboundedSender<LothlorienEvents>,
    pipeline_receiver: UnboundedReceiver<LothlorienEvents>,

    runtime_sender: broadcast::Sender<ConnectedClientEvents>,
    runtime_receiver: broadcast::Receiver<ConnectedClientEvents>,

    socket_addr: String,
    systems_temp_folder: PathBuf,
}

impl LothlorienPipeline {
    pub fn new(port: String) -> Self {
        let (pipeline_sender, pipeline_receiver) = mpsc::unbounded_channel();
        let (runtime_sender, runtime_receiver) = broadcast::channel(2);

        Self {
            pipeline_sender,
            pipeline_receiver,
            runtime_sender,
            runtime_receiver,
            socket_addr: format!("127.0.0.1:{}", port),
            systems_temp_folder: env::temp_dir(),
        }
    }

    pub async fn create_listener(&self) -> GaladrielResult<TcpListener> {
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

    pub fn create_pipeline(&self, listener: TcpListener) -> JoinHandle<()> {
        let _sender = self.pipeline_sender.clone();
        let _runtime_sender = self.runtime_sender.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = _sender.closed() => {
                        break;
                    }
                    // Continuously accept connections from the Lothlórien pipeline listener.
                    connection = listener.accept() => {
                        match connection {
                            Ok((stream, _)) => {
                                let _pipeline_sender = _sender.clone();
                                let _runtime_sender = _runtime_sender.clone();

                                let result = tokio::spawn(async move {
                                    Self::stream_sync(stream, _pipeline_sender, _runtime_sender).await
                                }).await;

                                match result {
                                    Ok(Ok(_)) => {}
                                    Ok(Err(err)) => {
                                        // TODO: Send the error to the runtime.
                                        println!("{:?}", err);
                                    }
                                    Err(err) => {
                                        // TODO: Send the error to the runtime.
                                        println!("{:?}", err);
                                    }
                                }
                            }
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                }
            }
        })
    }

    pub async fn next(&mut self) -> GaladrielResult<LothlorienEvents> {
        self.pipeline_receiver.recv().await.ok_or_else(|| {
            error!("Failed to receive Lothlórien pipeline event: Channel closed unexpectedly or an IO error occurred");

            Box::<dyn std::error::Error>::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error while receiving response from Lothlórien pipeline sender: No response received.",
            ))
        })
    }

    pub fn get_runtime_sender(&self) -> broadcast::Sender<ConnectedClientEvents> {
        self.runtime_sender.clone()
    }

    pub fn register_server_port_in_temp(&self, port: u16) -> GaladrielResult<()> {
        use std::io::Write;

        let systems_temp_file = self
            .systems_temp_folder
            .join("galadrielcss_lothlorien_pipeline_port.txt");
        let mut file = fs::File::create(systems_temp_file)?;

        write!(file, "{}", port)?;

        Ok(())
    }

    pub fn remove_server_port_in_temp(&self) -> GaladrielResult<()> {
        let systems_temp_file = self
            .systems_temp_folder
            .join("galadrielcss_lothlorien_pipeline_port.txt");

        if systems_temp_file.exists() {
            fs::remove_file(systems_temp_file)?;
        }

        Ok(())
    }

    async fn stream_sync(
        stream: tokio::net::TcpStream,
        pipeline_sender: UnboundedSender<LothlorienEvents>,
        runtime_sender: broadcast::Sender<ConnectedClientEvents>,
    ) -> GaladrielFuture<()> {
        let (mut stream_sender, mut stream_receiver) = accept_async(stream).await?.split();
        let mut runtime_receiver = runtime_sender.subscribe();

        let message = Message::Text("Galadriel CSS server is ready! 🚀\n\nYou can start styling your project with Galadriel CSS and see instant updates as changes are made.\n\nHappy coding, and may your styles be ever beautiful!".to_string());
        stream_sender.send(message).await?;

        loop {
            tokio::select! {
                // End the loop if the Lothlórien receiver is closed.
                _ = pipeline_sender.closed() => {
                    break;
                }
                _runtime_res = runtime_receiver.recv() => {}
                // Receives events from the connected user.
                client_res = stream_receiver.next() => {
                    match client_res {
                        None => {
                            break;
                        }
                        Some(Err(err)) => {
                            // TODO: Send the error to the runtime.
                            println!("{:?}", err);

                            break;
                        }
                        Some(Ok(event)) => {
                            // TODO: Process the received event.
                            // TODO: Send the received event to the main runtime via sender if necessary.
                            println!("{:?}", event);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
