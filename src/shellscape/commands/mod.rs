use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tracing::info;

#[derive(Clone, PartialEq, Debug)]
pub enum ShellscapeCommands {
    Terminate,
    None,
}

impl ShellscapeCommands {
    pub fn from_key_event(event: KeyEvent) -> ShellscapeCommands {
        info!("Handling key event: {:?}", event);

        match event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                info!("Received termination event from Esc or 'q'");
                ShellscapeCommands::Terminate
            }
            KeyCode::Char('c') | KeyCode::Char('C') if event.modifiers == KeyModifiers::CONTROL => {
                info!("Received termination command via `Ctrl+C`");
                ShellscapeCommands::Terminate
            }
            _ => ShellscapeCommands::None,
        }
    }
}
