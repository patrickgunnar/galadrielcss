use crate::asts::CASCADEX;

/// Retrieves the latest updated CSS content from the CASCADEX cache.
///
/// # Returns
/// - A `String` containing the latest CSS content if available.
/// - An empty `String` if no CSS content is found in the cache.
pub fn get_updated_css() -> String {
    // Attempts to retrieve the "cascading_sheet" entry from the CASCADEX cache.
    if let Some(latest_css) = CASCADEX.get("cascading_sheet") {
        // If the entry exists, return its value as an owned string.
        return latest_css.value().to_owned();
    }

    // Returns an empty string if the "cascading_sheet" entry is not found in the cache.
    String::new()
}
