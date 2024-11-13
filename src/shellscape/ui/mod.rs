use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::Backend, Terminal};
use tracing::{error, info, warn};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

use super::{app::ShellscapeApp, widgets::ShellscapeWidgets};

/// A struct representing a Shellscape interface with a terminal backend.
#[derive(Debug)]
pub struct ShellscapeInterface<B: Backend> {
    /// The terminal instance associated with the interface.
    terminal: Terminal<B>,
}

impl<B: Backend> ShellscapeInterface<B> {
    /// Creates a new `ShellscapeInterface` instance with the provided terminal backend.
    ///
    /// # Arguments
    ///
    /// * `terminal` - The terminal backend to use for the interface.
    ///
    /// # Returns
    ///
    /// Returns a `ShellscapeInterface` initialized with the provided terminal.
    pub fn new(terminal: Terminal<B>) -> Self {
        info!("Shellscape interface instance created with a terminal backend.");

        Self { terminal }
    }

    /// Initializes the Shellscape interface, enabling raw mode and configuring terminal settings.
    ///
    /// This function enables raw mode, enters an alternate screen, enables mouse capture,
    /// sets a custom panic hook, hides the cursor, and clears the terminal.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if any terminal operations fail.
    pub fn invoke(&mut self) -> GaladrielResult<()> {
        info!("Initializing Shellscape interface and enabling raw mode.");

        // Enable raw mode for terminal input
        terminal::enable_raw_mode().map_err(|err| {
            error!("Failed to enable raw mode: {:?}", err);

            GaladrielError::raise_general_interface_error(
                ErrorKind::TerminalRawModeActivationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        // Enter alternate screen and enable mouse capture
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).map_err(
            |err| {
                error!(
                    "Failed to enter alternate screen or enable mouse capture: {:?}",
                    err
                );

                GaladrielError::raise_general_interface_error(
                    ErrorKind::EnterTerminalAltScreenMouseCaptureFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            },
        )?;

        // Capture the original panic hook to restore after terminal reset
        let panic_hook = std::panic::take_hook();

        // Set a custom panic hook to reset the terminal in case of a panic
        std::panic::set_hook(Box::new(move |panic| {
            warn!("Panic occurred: invoking terminal reset.");

            if let Err(err) = Self::reset() {
                error!("Failed to reset the terminal after panic: {:?}", err);
            }

            panic_hook(panic);
        }));

        // Hide the cursor in the terminal
        self.terminal.hide_cursor().map_err(|err| {
            error!("Failed to hide cursor: {:?}", err);

            GaladrielError::raise_general_interface_error(
                ErrorKind::TerminalCursorHideFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        // Clear the terminal screen
        self.terminal.clear().map_err(|err| {
            error!("Failed to clear the terminal: {:?}", err);

            GaladrielError::raise_general_interface_error(
                ErrorKind::TerminalClearScreenFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        info!("Shellscape interface successfully invoked.");

        Ok(())
    }

    /// Renders the Shellscape interface, drawing the `ShellscapeApp` widgets onto the terminal.
    ///
    /// # Arguments
    ///
    /// * `shellscape_app` - The `ShellscapeApp` instance to render.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if rendering was successful, or an error if the terminal drawing failed.
    pub fn render(&mut self, shellscape_app: &mut ShellscapeApp) -> GaladrielResult<()> {
        //info!("Rendering Shellscape interface.");

        // Render the application widgets to the terminal
        self.terminal
            .draw(|frame| ShellscapeWidgets::paint(frame, shellscape_app))
            .map_err(|err| {
                error!("Failed to render interface: {:?}", err);

                GaladrielError::raise_critical_interface_error(
                    ErrorKind::TerminalWidgetRenderingError,
                    &err.to_string(),
                    ErrorAction::Restart,
                )
            })?;

        //info!("Shellscape interface rendered successfully.");

        Ok(())
    }

    /// Resets the terminal state to its default configuration.
    ///
    /// This function disables raw mode, leaves the alternate screen, and disables mouse capture.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the terminal reset was successful, or an error if any operation failed.
    pub fn reset() -> GaladrielResult<()> {
        info!("Resetting terminal state to default.");

        // Disable raw mode
        terminal::disable_raw_mode().map_err(|err| {
            error!("Failed to disable raw mode: {:?}", err);

            GaladrielError::raise_general_interface_error(
                ErrorKind::TerminalRawModeDeactivationFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        // Leave the alternate screen and disable mouse capture
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).map_err(
            |err| {
                error!(
                    "Failed to leave alternate screen or disable mouse capture: {:?}",
                    err
                );

                GaladrielError::raise_general_interface_error(
                    ErrorKind::LeaveTerminalAltScreenMouseCaptureFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            },
        )?;

        info!("Terminal state reset successfully.");

        Ok(())
    }

    /// Aborts the Shellscape interface, resetting the terminal and showing the cursor.
    ///
    /// This function calls `reset()` to restore the terminal state and shows the cursor after aborting.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the abortion process was successful, or an error if any operation failed.
    pub fn abort(&mut self) -> GaladrielResult<()> {
        warn!("Aborting Shellscape interface.");

        Self::reset()?;
        self.terminal.show_cursor().map_err(|err| {
            error!("Failed to show cursor during abort: {:?}", err);

            GaladrielError::raise_general_interface_error(
                ErrorKind::TerminalCursorUnhideFailed,
                &err.to_string(),
                ErrorAction::Exit,
            )
        })?;

        info!("Shellscape interface aborted and reset successfully.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        configatron::Configatron,
        shellscape::{app::ShellscapeApp, ui::ShellscapeInterface, Shellscape},
    };

    #[ignore]
    #[test]
    fn test_invoke() {
        let shellscape = Shellscape::new();
        let terminal = shellscape.get_terminal().unwrap();
        let mut interface = ShellscapeInterface::new(terminal);

        // Call the invoke method
        let result = interface.invoke();

        // Check if raw mode was enabled and screen mode was changed
        assert!(result.is_ok());
        assert!(matches!(result, Ok(_)));

        let _ = interface.abort();
    }

    #[ignore]
    #[test]
    fn test_render() {
        let shellscape = Shellscape::new();
        let terminal = shellscape.get_terminal().unwrap();
        let mut interface = ShellscapeInterface::new(terminal);
        let mut app = ShellscapeApp::new(
            Configatron::new(
                vec![],
                true,
                true,
                true,
                "8080".to_string(),
                "1.0.0".to_string(),
            ),
            "1.0.0",
        )
        .unwrap();

        // Call render and check if it's successful
        let result = interface.render(&mut app);

        // Assert that rendering was successful
        assert!(result.is_ok());

        let _ = interface.abort();
    }

    #[test]
    fn test_abort() {
        let shellscape = Shellscape::new();
        let terminal = shellscape.get_terminal().unwrap();
        let mut interface = ShellscapeInterface::new(terminal);

        // Call abort and check if it resets properly
        let result = interface.abort();

        // Assert that abort was successful and reset happened
        assert!(result.is_ok());

        let _ = interface.abort();
    }
}
