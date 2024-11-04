use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::Backend, Terminal};

use crate::GaladrielResult;

use super::{app::ShellscapeApp, widgets::ShellscapeWidgets};

#[derive(Debug)]
pub struct ShellscapeInterface<B: Backend> {
    terminal: Terminal<B>,
}

impl<B: Backend> ShellscapeInterface<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Self { terminal }
    }

    pub fn invoke(&mut self) -> GaladrielResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let panic_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("Fail to reset the terminal.");

            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        Ok(())
    }

    pub fn render(&mut self, shellscape_app: &mut ShellscapeApp) -> GaladrielResult<()> {
        self.terminal
            .draw(|frame| ShellscapeWidgets::paint(frame, shellscape_app))?;

        Ok(())
    }

    pub fn reset() -> GaladrielResult<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }

    pub fn abort(&mut self) -> GaladrielResult<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}
