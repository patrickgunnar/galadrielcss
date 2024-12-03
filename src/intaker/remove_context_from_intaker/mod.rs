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

#[cfg(test)]
mod tests {
    use crate::{asts::INTAKER, intaker::remove_context_from_intaker::remove_context_from_intaker};

    #[test]
    fn context_did_remove() {
        INTAKER.insert(
            "path/to/be/removed".to_string(),
            "myRemovedContext".to_string(),
        );

        let ctx = INTAKER
            .get("path/to/be/removed")
            .map(|entry| entry.value().to_owned());

        assert!(ctx.is_some());
        assert_eq!(ctx.unwrap(), "myRemovedContext".to_string());

        remove_context_from_intaker("path/to/be/removed");

        let ctx = INTAKER
            .get("path/to/be/removed")
            .map(|entry| entry.value().to_owned());

        assert!(ctx.is_none());
    }
}
