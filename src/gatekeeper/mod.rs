use crate::asts::GATEKEEPER;

/// Removes a specific file path from the GATEKEEPER registry.
///
/// This function iterates over all entries in the `GATEKEEPER` registry and removes the
/// specified `file_path` from any module path lists that contain it. If a module path matches
/// the `file_path`, it is removed from the list associated with the corresponding layout.
///
/// # Arguments
///
/// * `file_path` - The file path to remove from the GATEKEEPER registry's module path lists.
///
/// # Behavior
///
/// - This function will modify the GATEKEEPER registry by removing the specified file path
///   from any layout's list of module paths.
pub fn remove_path_from_gatekeeper(file_path: &str) {
    tracing::info!(
        "Initiating removal of file path '{}' from the GATEKEEPER registry.",
        file_path
    );

    // Iterate over all entries in the GATEKEEPER registry.
    GATEKEEPER.iter_mut().for_each(|mut entry| {
        // Access and mutate the value (list of module paths) associated with the current entry.
        // Retain only those module paths that are not equal to the `file_path`.
        entry
            .value_mut()
            .retain(|module_path| module_path != file_path);
    });
}
