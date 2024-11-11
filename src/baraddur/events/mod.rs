use crate::{error::GaladrielError, shellscape::notifications::ShellscapeNotifications};

#[derive(Clone, PartialEq, Debug)]
pub enum ObserverEvents {
    Notification(ShellscapeNotifications),
    AsyncDebouncerError(GaladrielError),
    Header(String),
    ReloadGaladrielConfigs,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ProcessingState {
    Running,
    Awaiting,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DebouncedWatch {
    Continue,
    Break,
}
