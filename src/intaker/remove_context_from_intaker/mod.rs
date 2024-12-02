use crate::asts::INTAKER;

/// Removes the context associated with a given file path from the `INTAKER` registry.
///
/// This function attempts to remove an entry from the global `INTAKER` map using the provided
/// `file_path` as the key. If the file path exists in the registry, its associated context is
/// removed. If the file path is not found, no operation is performed.
///
/// # Arguments
/// * `file_path` - The file path whose associated context should be removed from the registry.
///
/// This will remove the context associated with the specified file path, if it exists.
pub fn remove_context_from_intaker(file_path: &str) {
    tracing::info!(
        "Attempting to remove context for file path: '{}'",
        file_path
    );

    // Remove the entry from the global INTAKER registry using the file path as the key.
    INTAKER.remove(file_path);
}
