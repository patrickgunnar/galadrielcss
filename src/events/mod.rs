use crate::{error::GaladrielError, shellscape::notifications::ShellscapeNotifications};

#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielEvents {
    Notify(ShellscapeNotifications),
    Error(GaladrielError),
    ReloadGaladrielConfigs,
}
