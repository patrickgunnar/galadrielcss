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
