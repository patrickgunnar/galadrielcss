use std::{path::PathBuf, time::SystemTime};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

pub fn set_file_times(current_path: &PathBuf) -> GaladrielResult<()> {
    filetime::set_file_times(
        current_path,
        SystemTime::now().into(),
        SystemTime::now().into(),
    )
    .map_err(|err| {
        GaladrielError::raise_general_other_error(
            ErrorKind::Other,
            &err.to_string(),
            ErrorAction::Notify,
        )
    })
}
