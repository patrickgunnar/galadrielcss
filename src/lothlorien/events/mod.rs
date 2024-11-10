use crate::error::GaladrielError;

#[derive(Clone, PartialEq, Debug)]
pub enum LothlorienEvents {
    Notify(String),
    Error(GaladrielError),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConnectedClientEvents {}
