use crate::{asts::STYLITRON, types::Stylitron};

const SCHEMA_TYPES: &[&str] = &["mobile-first", "desktop-first"];

/// Resolves a breakpoint identifier based on the given `identifier` string.
///
/// This function searches for a breakpoint associated with the provided `identifier` in the
/// `STYLITRON` data structure under the "breakpoints" section. It checks both "mobile-first"
/// and "desktop-first" schemas and attempts to find the corresponding breakpoint entry. If a
/// matching breakpoint is found, it returns the resolved breakpoint as a `String`. If no
/// matching breakpoint is found, `None` is returned.
///
/// # Parameters
/// - `identifier`: A string slice representing the identifier of the breakpoint to resolve.
///
/// # Returns
/// - `Option<String>`: The resolved breakpoint if found, otherwise `None`.
pub fn resolve_breakpoint_identifier(identifier: &str) -> Option<String> {
    tracing::info!(identifier, "Resolving breakpoint identifier");

    // Attempt to retrieve the "breakpoints" data from the STYLITRON structure.
    STYLITRON
        .get("breakpoints")
        .and_then(|stylitron_data| match &*stylitron_data {
            Stylitron::Breakpoints(ref breakpoints_definitions) => {
                SCHEMA_TYPES.iter().find_map(|schema_type| {
                    breakpoints_definitions
                        .get(schema_type.to_owned())
                        .and_then(|schema_breakpoints| {
                            schema_breakpoints
                                .get(identifier)
                                .and_then(|breakpoint_entry| {
                                    tracing::info!(identifier, resolved_breakpoint = %breakpoint_entry, "Breakpoint resolved");

                                    Some(breakpoint_entry.to_owned())
                            })
                        })
                })
            }
            _ => None,
        })
}
