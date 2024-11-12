#[derive(Clone, PartialEq, Debug)]
pub enum ClientResponse {
    Text(String),
    Continue,
    Break,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConnectedClientEvents {}
