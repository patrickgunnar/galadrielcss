use ratatui::{prelude::Backend, Terminal};

use super::events::ShellscapeEvents;

#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeInterface<B: Backend> {
    terminal: Terminal<B>,
    pub events: ShellscapeEvents,
}

impl<B: Backend> ShellscapeInterface<B> {}
