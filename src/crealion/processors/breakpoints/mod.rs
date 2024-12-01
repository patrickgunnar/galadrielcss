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

#[cfg(test)]
mod breakpoints_test {
    use indexmap::IndexMap;

    use crate::{
        asts::STYLITRON, crealion::processors::breakpoints::resolve_breakpoint_identifier,
        types::Stylitron,
    };

    fn mock_breakpoints() {
        let map = IndexMap::from([
            (
                "mobile-first".to_string(),
                IndexMap::from([
                    ("mobSm".to_string(), "min-width:320px".to_string()),
                    ("mobMd".to_string(), "min-width:740px".to_string()),
                ]),
            ),
            (
                "desktop-first".to_string(),
                IndexMap::from([
                    ("deskSm".to_string(), "max-width:320px".to_string()),
                    ("deskMd".to_string(), "max-width:740px".to_string()),
                ]),
            ),
        ]);

        STYLITRON.insert("breakpoints".to_string(), Stylitron::Breakpoints(map));
    }

    #[test]
    fn sm_breakpoint_exists_in_mobile_first_node() {
        mock_breakpoints();

        let input = "mobSm";

        let resolved_input = resolve_breakpoint_identifier(input);
        let expected_result = "min-width:320px".to_string();

        assert!(resolved_input.is_some());
        assert_eq!(resolved_input, Some(expected_result));
    }

    #[test]
    fn md_breakpoint_exists_in_mobile_first_node() {
        mock_breakpoints();

        let input = "mobMd";

        let resolved_input = resolve_breakpoint_identifier(input);
        let expected_result = "min-width:740px".to_string();

        assert!(resolved_input.is_some());
        assert_eq!(resolved_input, Some(expected_result));
    }

    #[test]
    fn sm_breakpoint_exists_in_desktop_first_node() {
        mock_breakpoints();

        let input = "deskSm";

        let resolved_input = resolve_breakpoint_identifier(input);
        let expected_result = "max-width:320px".to_string();

        assert!(resolved_input.is_some());
        assert_eq!(resolved_input, Some(expected_result));
    }

    #[test]
    fn md_breakpoint_exists_in_desktop_first_node() {
        mock_breakpoints();

        let input = "deskMd";

        let resolved_input = resolve_breakpoint_identifier(input);
        let expected_result = "max-width:740px".to_string();

        assert!(resolved_input.is_some());
        assert_eq!(resolved_input, Some(expected_result));
    }
}
