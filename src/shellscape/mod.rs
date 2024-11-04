use app::ShellscapeApp;
use commands::ShellscapeCommands;
use events::{ShellscapeEvents, ShellscapeTerminalEvents};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::error;
use ui::ShellscapeInterface;

use crate::{configatron::Configatron, GaladrielResult};

mod app;
pub mod commands;
mod events;
mod ui;
mod widgets;

#[derive(Debug)]
pub struct Shellscape {
    // This meant to send data to the Galadriel CSS runtime
    events_sender: UnboundedSender<ShellscapeTerminalEvents>,
    events_receiver: UnboundedReceiver<ShellscapeTerminalEvents>,
}

impl Shellscape {
    pub fn new() -> Self {
        // This meant to send data to the Galadriel CSS runtime
        let (events_sender, events_receiver) = mpsc::unbounded_channel();

        Self {
            events_sender,
            events_receiver,
        }
    }

    fn get_terminal(&self) -> GaladrielResult<Terminal<CrosstermBackend<Stdout>>> {
        let backend = CrosstermBackend::new(io::stdout());

        Terminal::new(backend).map_err(|err| {
            error!("Failed to create terminal backend: {:?}", err);

            Box::<dyn std::error::Error>::from(err)
        })
    }

    pub fn create_interface(
        &self,
    ) -> GaladrielResult<ShellscapeInterface<CrosstermBackend<Stdout>>> {
        self.get_terminal().map(ShellscapeInterface::new)
    }

    pub fn create_app(&self, configs: Configatron) -> ShellscapeApp {
        ShellscapeApp::new(
            configs,
            "1.0.0",
            "Galadriel CSS and Nenyr License Agreement",
            "Patrick Gunnar",
            "© 2024 Galadriel CSS. Crafting modular, efficient, and scalable styles with precision. Built with Rust.",
            "Galadriel CSS",
        )
    }

    pub fn create_events(&self, tick_rate: u64) -> ShellscapeEvents {
        ShellscapeEvents::new(tick_rate, self.events_sender.clone())
    }

    pub async fn next(&mut self) -> GaladrielResult<ShellscapeTerminalEvents> {
        self.events_receiver.recv().await.ok_or_else(|| {
            error!("Failed to receive Shellscape terminal event: Channel closed unexpectedly or an IO error occurred");

            Box::<dyn std::error::Error>::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error while receiving response from Shellscape sender: No response received.",
            ))
        })
    }

    pub fn match_shellscape_event(
        &mut self,
        event: ShellscapeTerminalEvents,
    ) -> ShellscapeCommands {
        match event {
            ShellscapeTerminalEvents::Key(key) => ShellscapeCommands::from_key_event(key),
            _ => ShellscapeCommands::None,
        }
    }
}
