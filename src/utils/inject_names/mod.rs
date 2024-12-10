use std::path::PathBuf;

use crate::{
    error::{ErrorAction, ErrorKind},
    injectron::Injectron,
    GaladrielResult,
};

use super::{replace_file::replace_file, resilient_reader::resilient_reader};

/// Asynchronously injects context, class, and animation names into a file if applicable.
///
/// # Parameters
/// - `file_path`: The path to the file where names should be injected.
///
/// # Returns
/// - `GaladrielResult<bool>`:
///     - `Ok(true)` if the injection was successful and the file was updated.
///     - `Ok(false)` if no injection was performed.
///     - `Err` if any error occurs during reading, injection, or file replacement.
pub async fn inject_names(file_path: PathBuf) -> GaladrielResult<bool> {
    // Reads the raw content of the file in a resilient manner, handling potential errors.
    let raw_content = resilient_reader(&file_path).await?;

    // Creates a new `Injectron` instance with the file content and attempts to inject names.
    if let Some(injected_content) = Injectron::new(&raw_content).inject()? {
        // If injection is successful, replaces the file content with the modified content.
        replace_file(
            file_path,
            &injected_content,
            ErrorKind::OpenFileError,
            ErrorAction::Notify,
            ErrorKind::FileWriteError,
            ErrorAction::Notify,
        )
        .await?;

        // Returns `true` indicating that the injection and file replacement were successful.
        return Ok(true);
    }

    // Returns `false` if no injection was performed (e.g., no content to modify).
    Ok(false)
}
