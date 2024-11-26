use crate::{asts::STYLITRON, types::Stylitron};

/// A constant array defining the schemas used for breakpoint processing.
///
/// The schemas specify the ordering of breakpoints, either "mobile-first" (small to large)
/// or "desktop-first" (large to small).
const SCHEMAS: &[&str] = &["mobile-first", "desktop-first"];

/// A processor for handling and resolving breakpoints.
///
/// The `BreakpointProcessor` resolves a breakpoint value to its corresponding
/// representation (e.g., `min-width` or `max-width`) based on the specified schema.
#[derive(Clone, PartialEq, Debug)]
pub struct BreakpointProcessor {
    /// The breakpoint name to process (e.g., "myBreakpoint").
    breakpoint: String,
}

impl BreakpointProcessor {
    /// Creates a new `BreakpointProcessor` instance.
    ///
    /// # Arguments
    /// - `breakpoint`: A string slice representing the breakpoint value to process.
    ///
    /// # Returns
    /// - A new instance of `BreakpointProcessor`.
    pub fn new(breakpoint: &str) -> Self {
        tracing::info!(
            "Initializing BreakpointProcessor with breakpoint: '{}'",
            breakpoint
        );

        Self {
            breakpoint: breakpoint.to_string(),
        }
    }

    /// Processes the breakpoint and resolves it to a CSS-compatible format.
    ///
    /// This function looks up the breakpoint value in the `STYLITRON` data under the
    /// "breakpoints" node. It iterates over the defined schemas (`SCHEMAS`) to find
    /// a matching entry and formats it accordingly.
    ///
    /// # Returns
    /// - `Some(String)`: The formatted breakpoint value if a match is found.
    /// - `None`: If the breakpoint could not be resolved in any schema.
    pub fn process(&self) -> Option<String> {
        tracing::info!(
            "Processing breakpoint: '{}'. Attempting to resolve from schemas.",
            self.breakpoint
        );

        // Access the "breakpoints" node in the STYLITRON data structure.
        STYLITRON
            .get("breakpoints")
            .and_then(|stylitron_data| match &*stylitron_data {
                // If the data is of type `Breakpoints`, proceed with processing.
                Stylitron::Breakpoints(breakpoints_definitions) => {
                    tracing::debug!(
                        "Breakpoints definitions found. Searching schemas for breakpoint '{}'.",
                        self.breakpoint
                    );

                    // Iterate over the schemas to find a matching breakpoint.
                    SCHEMAS.iter().find_map(|schema| {
                        tracing::debug!("Checking schema: '{}'", schema);

                        // Retrieve the schema-specific breakpoints definitions.
                        breakpoints_definitions.get(&schema.to_string()).and_then(
                            |schema_breakpoints| {
                                tracing::debug!(
                                    "Schema '{}' contains breakpoint definitions. Checking for match with '{}'.",
                                    schema,
                                    self.breakpoint
                                );

                                // Attempt to resolve and format the breakpoint value.
                                schema_breakpoints
                                    .get(&self.breakpoint)
                                    .and_then(|breakpoint_entry| {
                                        tracing::info!(
                                            "Breakpoint '{}' resolved in schema '{}': '{}'",
                                            self.breakpoint,
                                            schema,
                                            breakpoint_entry
                                        );

                                        Some(breakpoint_entry.to_owned())
                                    })
                            },
                        )
                    })
                }
                _ => None, // If the node is not of type `Breakpoints`, return `None`.
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::crealion::{
        mocks::test_helpers::mock_breakpoints_node, processors::breakpoint::BreakpointProcessor,
    };

    #[test]
    fn test_breakpoint_processor_mobile_first() {
        mock_breakpoints_node(); // Setup mock breakpoints

        let processor = BreakpointProcessor::new("myMob02");

        // Process with "mobile-first" schema, expecting "min-width:720px"
        let result = processor.process();
        assert_eq!(result, Some("min-width:720px".to_string()));
    }

    #[test]
    fn test_breakpoint_processor_desktop_first() {
        mock_breakpoints_node(); // Setup mock breakpoints

        let processor = BreakpointProcessor::new("myDesk02");

        // Process with "desktop-first" schema, expecting "max-width:720px"
        let result = processor.process();
        assert_eq!(result, Some("max-width:720px".to_string()));
    }

    // Test that a non-existent breakpoint returns None
    #[test]
    fn test_breakpoint_processor_non_existent() {
        mock_breakpoints_node(); // Setup mock breakpoints

        let processor = BreakpointProcessor::new("nonExistentBreakpoint");

        // Process a non-existent breakpoint, expecting None
        let result = processor.process();
        assert_eq!(result, None);
    }
}
