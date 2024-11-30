use crate::{asts::STYLITRON, types::Stylitron};

/// Resolves an alias identifier based on a given `identifier` string and inherited contexts.
///
/// If the `identifier` starts with the prefix `"nickname;"`, it attempts to find the corresponding
/// alias from the provided `inherited_contexts`. It looks up the alias in the `STYLITRON` data structure
/// and returns the resolved alias if found. If the identifier does not have the `"nickname;"` prefix, it
/// simply returns the original `identifier` as a string.
///
/// # Parameters
/// - `identifier`: A string slice that represents the identifier to be resolved.
/// - `inherited_contexts`: A vector of strings representing the contexts from which the alias should be resolved.
///
/// # Returns
/// - `Option<String>`: The resolved alias if found, or the original identifier if no alias is found.
pub fn resolve_alias_identifier(
    identifier: &str,
    inherited_contexts: &Vec<String>,
) -> Option<String> {
    tracing::info!(identifier, "Resolving alias identifier");

    // Check if the identifier starts with the "nickname;" prefix.
    if let Some(alias) = identifier.strip_prefix("nickname;") {
        tracing::debug!(
            alias,
            "Found 'nickname;' prefix, attempting alias resolution"
        );

        return inherited_contexts.iter().find_map(|context_name| {
            STYLITRON
                .get("aliases")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Aliases(ref aliases_definitions) => aliases_definitions
                        .get(context_name)
                        .and_then(|context_aliases| {
                            context_aliases
                                .get(alias)
                                .and_then(|alias_entry| Some(alias_entry.to_owned()))
                        }),
                    _ => None,
                })
        });
    }

    tracing::trace!(
        identifier,
        "Identifier does not have 'nickname;' prefix, returning original"
    );

    // If the identifier doesn't have the "nickname;" prefix, return the original identifier as is.
    Some(identifier.to_string())
}
