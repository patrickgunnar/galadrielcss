use chrono::{DateTime, Local, TimeDelta};
use nenyr::error::NenyrError;

use crate::error::GaladrielError;

#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielEvents {
    Error(GaladrielError),
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum AlertTextType {
    Donation,
    License,
    Creator,
    ContributeAsDev,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielAlerts {
    Success {
        start_time: DateTime<Local>,
        ending_time: DateTime<Local>,
        duration: TimeDelta,
        message: String,
    },
    Information {
        start_time: DateTime<Local>,
        message: String,
    },
    Warning {
        start_time: DateTime<Local>,
        message: String,
    },
    NenyrError {
        start_time: DateTime<Local>,
        error: NenyrError,
    },
    GaladrielError {
        start_time: DateTime<Local>,
        error: GaladrielError,
    },
    Shortcuts {
        start_time: DateTime<Local>,
        shortcuts: Vec<(String, String)>,
    },
    Text {
        start_time: DateTime<Local>,
        title: String,
        content: Vec<String>,
        kind: AlertTextType,
    },
}

#[allow(dead_code)]
impl GaladrielAlerts {
    pub fn create_success(
        start_time: DateTime<Local>,
        ending_time: DateTime<Local>,
        duration: TimeDelta,
        message: &str,
    ) -> Self {
        GaladrielAlerts::Success {
            duration,
            start_time,
            ending_time,
            message: message.to_string(),
        }
    }

    pub fn create_information(start_time: DateTime<Local>, message: &str) -> Self {
        GaladrielAlerts::Information {
            start_time,
            message: message.to_string(),
        }
    }

    pub fn create_warning(start_time: DateTime<Local>, message: &str) -> Self {
        GaladrielAlerts::Warning {
            start_time,
            message: message.to_string(),
        }
    }

    pub fn create_nenyr_error(start_time: DateTime<Local>, error: NenyrError) -> Self {
        GaladrielAlerts::NenyrError { start_time, error }
    }

    pub fn create_galadriel_error(start_time: DateTime<Local>, error: GaladrielError) -> Self {
        GaladrielAlerts::GaladrielError { start_time, error }
    }

    pub fn create_text(
        kind: AlertTextType,
        start_time: DateTime<Local>,
        title: &str,
        content: Vec<String>,
    ) -> Self {
        GaladrielAlerts::Text {
            start_time,
            title: title.to_string(),
            content,
            kind,
        }
    }

    pub fn create_shortcuts(start_time: DateTime<Local>, shortcuts: Vec<(String, String)>) -> Self {
        GaladrielAlerts::Shortcuts {
            start_time,
            shortcuts,
        }
    }
}
