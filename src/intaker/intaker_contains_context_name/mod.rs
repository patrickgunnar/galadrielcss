use crate::asts::INTAKER;

/// Checks if the given context name exists in the `INTAKER` registry.
///
/// This function iterates through the entries in the global `INTAKER` registry to check if any
/// entry has a value that matches the provided `context_name`. It returns `true` if a matching
/// entry is found, otherwise `false`.
///
/// # Arguments
/// * `context_name` - The context name to search for in the `INTAKER` registry.
///
/// # Returns
/// Returns a `bool` indicating whether the context name exists in the registry. `true` if the
/// context name is found, and `false` otherwise.
///
/// This will check if the context name "myContextName" exists in the `INTAKER` registry.
///
/// # Efficiency
///
/// This function performs an iteration over the entire `INTAKER` registry, so its performance
/// may degrade if the registry contains a large number of entries.
pub fn intaker_contains_context_name(context_name: &str) -> bool {
    tracing::debug!(
        "Iterating over INTAKER registry to find context name: '{}'",
        context_name
    );

    // Iterate over each entry in the INTAKER registry and check if the value matches the given context_name.
    INTAKER.iter().any(|entry| entry.value() == context_name)
}
