use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tracing::info;

/// Enum representing the various shellscape commands.
#[derive(Clone, PartialEq, Debug)]
pub enum ShellscapeCommands {
    /// Command to terminate the shellscape.
    Terminate,
    /// No operation or command.
    None,
    ScrollUp,
    ScrollDown,
}

impl ShellscapeCommands {
    /// Converts a `KeyEvent` into a `ShellscapeCommands` variant.
    ///
    /// This method maps certain key events to shellscape commands. Specifically:
    /// - If the event corresponds to the Escape key (`Esc`) or the character `'q'`, it returns `ShellscapeCommands::Terminate`.
    /// - If the event corresponds to `Ctrl+C`, it also returns `ShellscapeCommands::Terminate`.
    /// - For any other event, it returns `ShellscapeCommands::None`.
    ///
    /// # Arguments
    /// - `event`: A `KeyEvent` representing the key press event to be processed.
    ///
    /// # Returns
    /// Returns a `ShellscapeCommands` variant based on the `KeyEvent` passed.
    pub fn from_key_event(event: KeyEvent) -> ShellscapeCommands {
        info!("Handling key event: {:?}", event);

        match event.code {
            // If the key event is Escape or the character 'q', return the Terminate command.
            KeyCode::Esc | KeyCode::Char('q') => {
                info!("Received termination event from Esc or 'q'");
                ShellscapeCommands::Terminate
            }
            // If the key event is 'c' or 'C' and the modifiers are `Ctrl`, return the Terminate command.
            KeyCode::Char('c') | KeyCode::Char('C') if event.modifiers == KeyModifiers::CONTROL => {
                info!("Received termination command via `Ctrl+C`");
                ShellscapeCommands::Terminate
            }
            KeyCode::Up if event.modifiers == KeyModifiers::CONTROL => ShellscapeCommands::ScrollUp,
            KeyCode::Down if event.modifiers == KeyModifiers::CONTROL => {
                ShellscapeCommands::ScrollDown
            }
            _ => ShellscapeCommands::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::shellscape::commands::ShellscapeCommands;

    #[test]
    fn test_from_key_event_terminate_with_esc() {
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        let result = ShellscapeCommands::from_key_event(event);
        assert_eq!(result, ShellscapeCommands::Terminate);
    }

    #[test]
    fn test_from_key_event_terminate_with_q() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        let result = ShellscapeCommands::from_key_event(event);
        assert_eq!(result, ShellscapeCommands::Terminate);
    }

    #[test]
    fn test_from_key_event_terminate_with_ctrl_c() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let result = ShellscapeCommands::from_key_event(event);
        assert_eq!(result, ShellscapeCommands::Terminate);
    }

    #[test]
    fn test_from_key_event_none_with_ctrl_a() {
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let result = ShellscapeCommands::from_key_event(event);
        assert_eq!(result, ShellscapeCommands::None);
    }

    #[test]
    fn test_from_key_event_none_with_non_control() {
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let result = ShellscapeCommands::from_key_event(event);
        assert_eq!(result, ShellscapeCommands::None);
    }
}
