use std::path::PathBuf;

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

/// Replaces the contents of a file with the provided text.
///
/// This asynchronous function opens a file at the given `file_path`, truncates its content,
/// and writes the `replacement_context` to it. If any error occurs during file access or
/// writing, it raises a `GaladrielError` with the specified `ErrorKind` and `ErrorAction`.
///
/// # Arguments
///
/// - `file_path` - A `PathBuf` specifying the path to the file to be modified.
/// - `replacement_context` - A string slice containing the new content to replace the existing content.
/// - `error_open_kind` - The kind of error to raise if the file cannot be opened.
/// - `error_open_action` - The action to suggest if the file opening fails.
/// - `error_write_kind` - The kind of error to raise if the write operation fails.
/// - `error_write_action` - The action to suggest if the write operation fails.
///
/// # Returns
///
/// This function returns a `GaladrielResult<()>`. On success, it returns `Ok(())`.
/// If an error occurs, it returns a `GaladrielError` wrapped in the `Err` variant.
///
/// # Errors
///
/// - If the file cannot be opened, an error is raised using `error_open_kind` and `error_open_action`.
/// - If the file cannot be written to, an error is raised using `error_write_kind` and `error_write_action`.
///
/// - This function uses asynchronous I/O to handle potentially long-running operations
///   like file access and writing efficiently.
/// - The file must already exist; it does not create a new file if the specified `file_path` does not exist.
pub async fn replace_file(
    file_path: PathBuf,
    replacement_context: &str,
    error_open_kind: ErrorKind,
    error_open_action: ErrorAction,
    error_write_kind: ErrorKind,
    error_write_action: ErrorAction,
) -> GaladrielResult<()> {
    // Attempt to open the file for writing and truncating its content.
    let mut file = OpenOptions::new()
        .write(true) // Open file with write permissions.
        .truncate(true) // Truncate the file content upon opening.
        .open(file_path) // Path of the file to open.
        .await // Asynchronous file operation.
        .map_err(|err| {
            // Map any error encountered during file opening to a GaladrielError.
            GaladrielError::raise_general_runtime_error(
                error_open_kind,
                &err.to_string(),
                error_open_action,
            )
        })?;

    // Write the replacement content into the opened file.
    file.write_all(replacement_context.as_bytes())
        .await
        .map_err(|err| {
            // Map any error encountered during writing to a GaladrielError.
            GaladrielError::raise_general_runtime_error(
                error_write_kind,
                &err.to_string(),
                error_write_action,
            )
        })?;

    file.sync_all().await.map_err(|err| {
        GaladrielError::raise_general_other_error(
            ErrorKind::Other,
            &err.to_string(),
            ErrorAction::Notify,
        )
    })?;

    Ok(())
}
