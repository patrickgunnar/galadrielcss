use chrono::{DateTime, Local, TimeDelta};
use nenyr::error::NenyrError;

use crate::error::GaladrielError;

#[derive(Clone, PartialEq, Debug)]
pub enum ShellscapeNotifications {
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
}

impl ShellscapeNotifications {
    pub fn create_success(
        start_time: DateTime<Local>,
        ending_time: DateTime<Local>,
        duration: TimeDelta,
        message: &str,
    ) -> Self {
        ShellscapeNotifications::Success {
            duration,
            start_time,
            ending_time,
            message: message.to_string(),
        }
    }

    pub fn create_information(start_time: DateTime<Local>, message: &str) -> Self {
        ShellscapeNotifications::Information {
            start_time,
            message: message.to_string(),
        }
    }

    pub fn create_warning(start_time: DateTime<Local>, message: &str) -> Self {
        ShellscapeNotifications::Warning {
            start_time,
            message: message.to_string(),
        }
    }

    pub fn create_nenyr_error(start_time: DateTime<Local>, error: NenyrError) -> Self {
        ShellscapeNotifications::NenyrError { start_time, error }
    }

    pub fn create_galadriel_error(start_time: DateTime<Local>, error: GaladrielError) -> Self {
        ShellscapeNotifications::GaladrielError { start_time, error }
    }
}
