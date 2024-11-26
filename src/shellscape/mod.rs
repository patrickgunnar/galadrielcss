use app::ShellscapeApp;
use commands::ShellscapeCommands;
use events::{ShellscapeEvents, ShellscapeTerminalEvents};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{error, info};
use ui::ShellscapeInterface;

use crate::{
    configatron::Configatron,
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

pub mod alerts;
pub mod app;
mod area;
pub mod commands;
pub mod events;
mod metadata;
mod ui;
mod widgets;

/// Represents a shell interface to communicate with the Galadriel CSS runtime.
#[derive(Debug)]
pub struct Shellscape {
    // Sender for events to communicate with the Galadriel CSS runtime
    events_sender: UnboundedSender<ShellscapeTerminalEvents>,
    // Receiver for events to the Galadriel CSS runtime receives events from Shellscape
    events_receiver: UnboundedReceiver<ShellscapeTerminalEvents>,
}

impl Shellscape {
    /// Creates a new instance of `Shellscape` with an event sender and receiver.
    ///
    /// # Returns
    /// A new `Shellscape` instance with unbounded channel sender and receiver.
    pub fn new() -> Self {
        // Initialize unbounded channels for sending and receiving events
        let (events_sender, events_receiver) = mpsc::unbounded_channel();

        info!("Shellscape instance created with new event sender and receiver.");

        Self {
            events_sender,
            events_receiver,
        }
    }

    /// Attempts to create a new terminal instance using the Crossterm library.
    ///
    /// # Returns
    /// A `GaladrielResult` containing the terminal instance or an error if terminal creation fails.
    fn get_terminal(&self) -> GaladrielResult<Terminal<CrosstermBackend<Stdout>>> {
        info!("Attempting to create a new terminal instance.");

        // Initialize the backend for the terminal
        let backend = CrosstermBackend::new(io::stdout());

        // Create a terminal with the provided backend
        Terminal::new(backend).map_err(|err| {
            error!("Failed to create terminal backend: {:?}", err);

            GaladrielError::raise_general_interface_error(
                ErrorKind::TerminalInitializationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })
    }

    /// Creates and returns a Shellscape interface, initializing a new terminal instance.
    ///
    /// # Returns
    /// A `GaladrielResult` containing the created `ShellscapeInterface` instance.
    pub fn create_interface(
        &self,
    ) -> GaladrielResult<ShellscapeInterface<CrosstermBackend<Stdout>>> {
        info!("Creating Shellscape interface.");

        // Create a terminal and then return the interface
        self.get_terminal().map(ShellscapeInterface::new)
    }

    /// Creates and returns a Shellscape application instance with the provided configurations.
    ///
    /// # Arguments
    /// * `configs` - The configuration parameters used to initialize the app.
    ///
    /// # Returns
    /// A `ShellscapeApp` instance initialized with the provided configurations.
    pub fn create_app(&self, configs: Configatron) -> GaladrielResult<ShellscapeApp> {
        info!("Creating Shellscape application instance with provided configurations.");

        ShellscapeApp::new(configs, "1.0.0")
    }

    /// Creates and returns a Shellscape event handler with a specified tick rate.
    ///
    /// # Arguments
    /// * `tick_rate` - The tick rate (in milliseconds) for the event loop.
    ///
    /// # Returns
    /// A `ShellscapeEvents` instance to handle events based on the tick rate.
    pub fn create_events(&self, tick_rate: u64) -> ShellscapeEvents {
        info!(
            "Initializing Shellscape events with a tick rate of {} ms.",
            tick_rate
        );

        ShellscapeEvents::new(tick_rate, self.events_sender.clone())
    }

    /// Awaits and returns the next terminal event from the Shellscape event receiver.
    ///
    /// # Returns
    /// A `GaladrielResult` containing the next terminal event or an error if the channel is closed or an I/O error occurs.
    pub async fn next(&mut self) -> GaladrielResult<ShellscapeTerminalEvents> {
        //info!("Waiting for the next Shellscape terminal event.");

        // Wait for the next event from the receiver
        self.events_receiver.recv().await.ok_or_else(|| {
            error!("Failed to receive Shellscape terminal event: Channel closed unexpectedly or an IO error occurred");

            GaladrielError::raise_general_interface_error(
                ErrorKind::TerminalEventReceiveFailed,
                "Error while receiving response from Shellscape sender: No response received.",
                ErrorAction::Notify
            )
        })
    }

    /// Processes the given terminal event and returns a corresponding command.
    ///
    /// # Arguments
    /// * `event` - The terminal event that needs to be processed.
    ///
    /// # Returns
    /// A corresponding `ShellscapeCommands` variant based on the event type.
    pub fn match_shellscape_event(
        &mut self,
        event: ShellscapeTerminalEvents,
    ) -> ShellscapeCommands {
        match event {
            // If the event is a key event, convert it into a command
            ShellscapeTerminalEvents::Key(key) => {
                info!("Processing key event: {:?}", key);
                ShellscapeCommands::from_key_event(key)
            }
            ShellscapeTerminalEvents::Mouse(event) => {
                //info!("Processing mouse event: {:?}", event);
                ShellscapeCommands::from_mouse_event(event)
            }
            // If the event is unrecognized, log a warning and return None
            _ => {
                // warn!("Received an unrecognized Shellscape event: {:?}", event);
                ShellscapeCommands::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{Configatron, Shellscape, ShellscapeCommands, ShellscapeTerminalEvents};

    #[tokio::test]
    async fn test_shellscape_new() {
        let mut shellscape = Shellscape::new();
        // Send a test event through events_sender and verify it is received by events_receiver
        let _ = shellscape
            .events_sender
            .send(ShellscapeTerminalEvents::Tick);
        let received_event = shellscape.next().await;

        assert_eq!(format!("{:?}", received_event), "Ok(Tick)".to_string());
    }

    #[test]
    fn test_create_app() {
        let shellscape = Shellscape::new();
        let configs = Configatron::new(
            vec![],
            true,
            true,
            true,
            "8080".to_string(),
            "1.0.0".to_string(),
        );
        let app = shellscape.create_app(configs.clone()).unwrap();

        assert_eq!(app.configs, configs);
    }

    #[tokio::test]
    async fn test_next_event_reception() {
        let mut shellscape = Shellscape::new();

        // Send an event to the sender side
        shellscape
            .events_sender
            .send(ShellscapeTerminalEvents::Tick)
            .unwrap();

        // Call next and check for correct event
        let result = shellscape.next().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShellscapeTerminalEvents::Tick);
    }

    #[tokio::test]
    async fn test_get_terminal_failure() {
        let shellscape = Shellscape::new();

        let result = shellscape.get_terminal();
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_shellscape_event_key() {
        let mut shellscape = Shellscape::new();
        let event =
            ShellscapeTerminalEvents::Key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE));

        // This test assumes `ShellscapeCommands::from_key_event` would process "A" correctly.
        let command = shellscape.match_shellscape_event(event);
        assert_eq!(
            command,
            ShellscapeCommands::from_key_event(KeyEvent::new(
                KeyCode::Char('A'),
                KeyModifiers::NONE
            ))
        );
    }

    #[test]
    fn test_create_interface_error_handling() {
        let shellscape = Shellscape::new();

        let interface = shellscape.create_interface();
        assert!(interface.is_ok());
    }
}
