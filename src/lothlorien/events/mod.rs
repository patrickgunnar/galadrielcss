use crate::{error::GaladrielError, shellscape::notifications::ShellscapeNotifications};

#[derive(Clone, PartialEq, Debug)]
pub enum LothlorienEvents {
    Header(String),
    Notify(ShellscapeNotifications),
    Error(GaladrielError),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ClientResponse {
    Text(String),
    Continue,
    Break,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConnectedClientEvents {}
