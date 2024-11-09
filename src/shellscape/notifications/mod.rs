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
        &self,
        start_time: DateTime<Local>,
        ending_time: DateTime<Local>,
        duration: TimeDelta,
        message: &str,
    ) -> ShellscapeNotifications {
        ShellscapeNotifications::Success {
            duration,
            start_time,
            ending_time,
            message: message.to_string(),
        }
    }

    pub fn create_information(
        &self,
        start_time: DateTime<Local>,
        message: &str,
    ) -> ShellscapeNotifications {
        ShellscapeNotifications::Information {
            start_time,
            message: message.to_string(),
        }
    }

    pub fn create_warning(
        &self,
        start_time: DateTime<Local>,
        message: &str,
    ) -> ShellscapeNotifications {
        ShellscapeNotifications::Warning {
            start_time,
            message: message.to_string(),
        }
    }

    pub fn create_nenyr_error(
        &self,
        start_time: DateTime<Local>,
        error: NenyrError,
    ) -> ShellscapeNotifications {
        ShellscapeNotifications::NenyrError { start_time, error }
    }

    pub fn create_galadriel_error(
        &self,
        start_time: DateTime<Local>,
        error: GaladrielError,
    ) -> ShellscapeNotifications {
        ShellscapeNotifications::GaladrielError { start_time, error }
    }
}
