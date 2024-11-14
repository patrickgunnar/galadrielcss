use crate::{error::GaladrielError, shellscape::alerts::ShellscapeAlerts};

#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielEvents {
    Notify(ShellscapeAlerts),
    Error(GaladrielError),
    ReloadGaladrielConfigs,
}
