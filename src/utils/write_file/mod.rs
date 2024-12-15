use std::path::PathBuf;

use tokio::{fs, io::AsyncWriteExt};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

/// Writes content to a specified file, creating the necessary directories and handling errors gracefully.
///
/// # Arguments
/// - `folder_path` (`PathBuf`): The path to the folder where the file will be created.
///   If the folder doesn't exist, it will be created.
/// - `file_path` (`PathBuf`): The path to the file to be written.
/// - `write_context` (`String`): The content to be written to the file.
/// - `error_action` (`ErrorAction`): Specifies the action to take if an error occurs.
///
/// # Returns
/// - `GaladrielResult<()>`: Returns `Ok(())` if the operation succeeds, or an error wrapped in `GaladrielResult` otherwise.
///
/// # Errors
/// This function can return the following error types:
/// - `FileDirCreationError`: If the directory cannot be created.
/// - `FileCreationError`: If the file cannot be created.
/// - `FileWriteError`: If writing to the file fails.
/// - `Other`: If syncing the file fails.
pub async fn write_file(
    folder_path: PathBuf,
    file_path: PathBuf,
    write_context: String,
    error_action: ErrorAction,
    error_file_creation_kind: ErrorKind,
    error_file_write_kind: ErrorKind,
) -> GaladrielResult<()> {
    tracing::info!("Starting the write operation for file: {:?}", file_path);

    // Ensure the folder path exists or create it.
    fs::create_dir_all(folder_path).await.map_err(|err| {
        GaladrielError::raise_critical_other_error(
            ErrorKind::FileDirCreationError,
            &err.to_string(),
            error_action.to_owned(),
        )
    })?;

    tracing::debug!("Attempting to create file: {:?}", file_path);

    // Attempt to create the file at the specified path.
    let mut file = fs::File::create(file_path).await.map_err(|err| {
        GaladrielError::raise_critical_other_error(
            error_file_creation_kind,
            &err.to_string(),
            error_action.to_owned(),
        )
    })?;

    tracing::debug!(
        "Writing content to file. Content size: {} bytes.",
        write_context.len()
    );

    // Write the provided content to the file as bytes.
    file.write_all(write_context.as_bytes())
        .await
        .map_err(|err| {
            GaladrielError::raise_critical_other_error(
                error_file_write_kind,
                &err.to_string(),
                error_action.to_owned(),
            )
        })?;

    tracing::debug!("Content successfully written to file.");

    // Ensure all written content is flushed and synced to the storage.
    file.sync_all().await.map_err(|err| {
        GaladrielError::raise_critical_other_error(ErrorKind::Other, &err.to_string(), error_action)
    })?;

    tracing::info!("Write operation completed successfully for file.");

    Ok(())
}
