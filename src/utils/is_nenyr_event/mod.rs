use std::path::PathBuf;

use ignore::overrides;

/// Checks if a given path corresponds to a Nenyr file.
///
/// # Arguments
/// - `path`: Path to the file triggering the event.
/// - `matcher`: Matcher for identifying Nenyr-specific events.
///
/// # Returns
/// - `true` if the path corresponds to a Nenyr file, otherwise `false`.
pub fn is_nenyr_event(path: &PathBuf, matcher: &overrides::Override) -> bool {
    !matcher.matched(path, false).is_ignore()
        && path.extension().map(|ext| ext == "nyr").unwrap_or(false)
}
