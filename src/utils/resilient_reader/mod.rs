use std::path::PathBuf;

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

pub async fn resilient_reader(path: &PathBuf) -> GaladrielResult<String> {
    let mut attempts = 0;
    let retries = 20;

    while attempts <= retries {
        let raw_content = tokio::fs::read_to_string(path).await.map_err(|err| {
            GaladrielError::raise_general_other_error(
                ErrorKind::FileReadFailed,
                &err.to_string(),
                ErrorAction::Notify,
            )
        })?;

        if !raw_content.is_empty() {
            return Ok(raw_content);
        }

        attempts += 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }

    Ok(String::new())
}
