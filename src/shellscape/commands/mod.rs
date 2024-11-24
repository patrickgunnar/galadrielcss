use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use tracing::info;

/// Enum representing the various shellscape commands.
#[derive(Clone, PartialEq, Debug)]
pub enum ShellscapeCommands {
    /// Command to terminate the shellscape.
    Terminate,
    /// No operation or command.
    None,
    ScrollNotificationsUp,
    ScrollNotificationsDown,
    ScrollDockUp,
    ScrollDockDown,
    ScrollUp {
        column: u16,
        row: u16,
    },
    ScrollDown {
        column: u16,
        row: u16,
    },
    ClearAlertsTable,
    VewShortcuts,
    ViewLicense,
    MakeDonation,
    ContributeAsDev,
    AboutAuthor,
    ToggleResetStyles,
    ToggleMinifiedStyles,
    ToggleAutoNaming,
    ModifyVersion,
    AdjustExclude,
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
            KeyCode::Char('r') | KeyCode::Char('R') if event.modifiers == KeyModifiers::SHIFT => {
                info!("Toggling reset styles...");
                ShellscapeCommands::ToggleResetStyles
            }
            KeyCode::Char('m') | KeyCode::Char('M') if event.modifiers == KeyModifiers::SHIFT => {
                info!("Toggling minified styles...");
                ShellscapeCommands::ToggleMinifiedStyles
            }
            KeyCode::Char('n') | KeyCode::Char('N') if event.modifiers == KeyModifiers::SHIFT => {
                info!("Toggling auto-naming feature...");
                ShellscapeCommands::ToggleAutoNaming
            }
            KeyCode::Char('v') | KeyCode::Char('V') if event.modifiers == KeyModifiers::SHIFT => {
                info!("Modifying version configuration...");
                ShellscapeCommands::ModifyVersion
            }
            KeyCode::Char('e') | KeyCode::Char('E') if event.modifiers == KeyModifiers::SHIFT => {
                info!("Adjusting exclusion settings...");
                ShellscapeCommands::AdjustExclude
            }
            KeyCode::Char('k') | KeyCode::Char('K') if event.modifiers == KeyModifiers::SHIFT => {
                info!("Clearing all alerts...");
                ShellscapeCommands::ClearAlertsTable
            }
            KeyCode::Char('s') | KeyCode::Char('S') if event.modifiers == KeyModifiers::CONTROL => {
                info!("Displaying shortcut guide...");
                ShellscapeCommands::VewShortcuts
            }
            KeyCode::Char('l') | KeyCode::Char('L') if event.modifiers == KeyModifiers::CONTROL => {
                info!("Opening license information...");
                ShellscapeCommands::ViewLicense
            }
            KeyCode::Char('d') | KeyCode::Char('D') if event.modifiers == KeyModifiers::CONTROL => {
                info!("Displaying donation guide...");
                ShellscapeCommands::MakeDonation
            }
            KeyCode::Char('t') | KeyCode::Char('T') if event.modifiers == KeyModifiers::CONTROL => {
                info!("Opening contribution information for developers...");
                ShellscapeCommands::ContributeAsDev
            }
            KeyCode::Char('a') | KeyCode::Char('A') if event.modifiers == KeyModifiers::CONTROL => {
                info!("Displaying author information...");
                ShellscapeCommands::AboutAuthor
            }
            KeyCode::Up if event.modifiers == KeyModifiers::CONTROL => {
                ShellscapeCommands::ScrollNotificationsUp
            }
            KeyCode::Down if event.modifiers == KeyModifiers::CONTROL => {
                ShellscapeCommands::ScrollNotificationsDown
            }
            KeyCode::Up if event.modifiers == KeyModifiers::SHIFT => {
                ShellscapeCommands::ScrollDockUp
            }
            KeyCode::Down if event.modifiers == KeyModifiers::SHIFT => {
                ShellscapeCommands::ScrollDockDown
            }
            _ => ShellscapeCommands::None,
        }
    }

    pub fn from_mouse_event(event: MouseEvent) -> ShellscapeCommands {
        match event.kind {
            MouseEventKind::ScrollDown => ShellscapeCommands::ScrollDown {
                column: event.column,
                row: event.row,
            },
            MouseEventKind::ScrollUp => ShellscapeCommands::ScrollUp {
                column: event.column,
                row: event.row,
            },
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
        assert_eq!(result, ShellscapeCommands::AboutAuthor);
    }

    #[test]
    fn test_from_key_event_none_with_non_control() {
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let result = ShellscapeCommands::from_key_event(event);
        assert_eq!(result, ShellscapeCommands::None);
    }
}
