use std::path::PathBuf;

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum ContextProcessingStatus {
    Awaiting,
    Processing,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum BaraddurEventProcessorKind {
    Modify,
    Remove,
    None,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum BaraddurEventProcessor {
    ProcessEvent {
        kind: BaraddurEventProcessorKind,
        path: PathBuf,
    },
    ReloadGaladrielConfigs,
}

#[derive(Clone, PartialEq, Debug)]
pub enum BaraddurRenameEventState {
    Rename,
    None,
}
