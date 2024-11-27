use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{crealion::Crealion, shellscape::alerts::ShellscapeAlerts};

use super::types::UtilityClass;

impl Crealion {
    /// Spawns an asynchronous task to process responsive styles.
    ///
    /// This function handles style patterns associated with specific breakpoints and generates
    /// utility classes for those styles. It returns a `JoinHandle` for the task, which resolves
    /// to a tuple containing the generated utility classes, any alerts, and the names of the
    /// utility classes.
    ///
    /// # Parameters
    /// - `inherited_contexts`: A vector of strings representing inherited style contexts.
    /// - `class_name`: The name of the Nenyr class for which styles are being processed.
    /// - `is_important`: A boolean indicating if the generated styles should have `!important`.
    /// - `responsive_patterns`: An optional nested map of responsive style patterns, where:
    ///   - The outer key is the breakpoint.
    ///   - The inner map key is the pattern.
    ///   - The innermost map contains property-value pairs.
    ///
    /// # Returns
    /// A `JoinHandle` that resolves to a tuple containing:
    /// - A vector of `UtilityClass` objects for the generated styles.
    /// - A vector of `ShellscapeAlerts` containing any alerts or warnings.
    /// - A vector of strings with the names of the generated utility classes.
    #[tracing::instrument(level = "info", skip(responsive_patterns))]
    pub fn process_responsive_styles(
        inherited_contexts: Vec<String>,
        class_name: String,
        is_important: bool,
        responsive_patterns: Option<IndexMap<String, IndexMap<String, IndexMap<String, String>>>>,
    ) -> JoinHandle<(Vec<UtilityClass>, Vec<ShellscapeAlerts>, Vec<String>)> {
        // Spawn an asynchronous task using `tokio::spawn`.
        tokio::spawn(async move {
            tracing::info!(
                "Starting process_responsive_styles for class: {}",
                class_name
            );

            // Initialize a mutable vector to collect alerts.
            let mut alerts: Vec<ShellscapeAlerts> = vec![];
            // Initialize a mutable vector to collect utility classes.
            let mut classes: Vec<UtilityClass> = vec![];
            // Initialize a mutable vector to collect the names of utility classes.
            let mut utility_names: Vec<String> = vec![];

            // Check if responsive patterns are provided.
            match responsive_patterns {
                Some(patterns) => {
                    tracing::debug!(
                        "Processing {} responsive patterns for class: {}",
                        patterns.len(),
                        class_name
                    );

                    // Process each breakpoint and its associated style patterns.
                    Self::match_style_breakpoint(
                        &inherited_contexts, // Pass inherited contexts as reference.
                        &class_name,         // Pass the class name as reference.
                        is_important,        // Propagate the `!important` flag.
                        &mut alerts,         // Collect alerts during processing.
                        &mut classes,        // Collect generated utility classes.
                        &mut utility_names,  // Collect utility class names.
                        patterns,            // Provide the responsive style patterns to match.
                    )
                    .await; // Await the asynchronous processing.
                }
                _ => {
                    tracing::warn!(
                        "No responsive patterns provided for class: {}. Skipping processing.",
                        class_name
                    );
                }
            }

            tracing::info!(
                "Finished processing styles for class: {}. Generated {} utility classes, {} alerts, and {} utility names.",
                class_name,
                classes.len(),
                alerts.len(),
                utility_names.len()
            );

            // Return the collected utility classes, alerts, and utility class names as a tuple.
            (classes, alerts, utility_names)
        })
    }

    /// Matches style patterns to a specific breakpoint and processes them.
    ///
    /// This function iterates over the responsive style patterns for each breakpoint and
    /// invokes the `match_style_patterns` method to process the styles for that breakpoint.
    ///
    /// # Parameters
    /// - `inherited_contexts`: A reference to a vector of inherited style contexts.
    /// - `class_name`: A reference to the Nenyr class name for which styles are processed.
    /// - `is_important`: Indicates if styles should be marked as `!important`.
    /// - `alerts`: A mutable vector to store any generated alerts or warnings.
    /// - `classes`: A mutable vector to store the generated utility classes.
    /// - `utility_names`: A mutable vector to store the names of generated utility classes.
    /// - `responsive_patterns`: A nested map of responsive style patterns, where:
    ///   - The outer key is the breakpoint.
    ///   - The inner map key is the pattern.
    ///   - The innermost map contains property-value pairs.
    #[tracing::instrument(
        level = "debug",
        skip(alerts, classes, utility_names, responsive_patterns)
    )]
    async fn match_style_breakpoint(
        inherited_contexts: &Vec<String>,
        class_name: &String,
        is_important: bool,
        alerts: &mut Vec<ShellscapeAlerts>,
        classes: &mut Vec<UtilityClass>,
        utility_names: &mut Vec<String>,
        responsive_patterns: IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
    ) {
        tracing::debug!(
            "Starting match_style_breakpoint for class: {} with {} breakpoints.",
            class_name,
            responsive_patterns.len()
        );

        // Iterate over each breakpoint and its associated patterns.
        for (breakpoint, patterns) in responsive_patterns {
            tracing::trace!(
                "Processing breakpoint: {} with {} style patterns.",
                breakpoint,
                patterns.len()
            );

            // Match and process styles for the current breakpoint.
            Self::match_style_patterns(
                inherited_contexts,          // Pass inherited contexts as reference.
                Some(breakpoint.to_owned()), // Specify the current breakpoint.
                class_name,                  // Pass the class name as reference.
                is_important,                // Propagate the `!important` flag.
                alerts,                      // Collect alerts during processing.
                classes,                     // Collect generated utility classes.
                utility_names,               // Collect utility class names.
                patterns,                    // Provide the style patterns for the breakpoint.
            )
            .await; // Await the asynchronous matching process.

            tracing::trace!(
                "Finished processing breakpoint: {} for class: {}.",
                breakpoint,
                class_name
            );
        }

        tracing::debug!(
            "Completed match_style_breakpoint for class: {}.",
            class_name
        );
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::crealion::{
        mocks::test_helpers::{
            mock_animations_node, mock_breakpoints_node, mock_themes_node, mock_variable_node,
        },
        Crealion,
    };

    #[tokio::test]
    async fn test_process_responsive_styles_with_patterns() {
        // Mock data required for the test
        mock_variable_node();
        mock_themes_node();
        mock_animations_node();
        mock_breakpoints_node();

        let inherited_contexts = vec!["galaxyContext".to_string()];
        let class_name = "myClass".to_string();
        let is_important = false;

        let style_patterns = Some(IndexMap::from([(
            "myMob01".to_string(),
            IndexMap::from([
                (
                    "_stylesheet".to_string(),
                    IndexMap::from([
                        (
                            "background-color".to_string(),
                            "${primaryColor}".to_string(),
                        ),
                        (
                            "animation-name".to_string(),
                            "${mySecondaryAnimation}".to_string(),
                        ),
                        (
                            "color".to_string(),
                            "${galaxyForegroundColorThemed}".to_string(),
                        ),
                    ]),
                ),
                (
                    ":hover".to_string(),
                    IndexMap::from([
                        (
                            "background-color".to_string(),
                            "${primaryColor}".to_string(),
                        ),
                        (
                            "animation-name".to_string(),
                            "${mySecondaryAnimation}".to_string(),
                        ),
                        (
                            "color".to_string(),
                            "${galaxyForegroundColorThemed}".to_string(),
                        ),
                    ]),
                ),
            ]),
        )]));

        // Call the function within the runtime
        let result = Crealion::process_responsive_styles(
            inherited_contexts,
            class_name,
            is_important,
            style_patterns,
        )
        .await;

        let (classes, alerts, utility_names) = result.unwrap();

        // Assert that the function generates utility classes and alerts as expected
        assert!(!classes.is_empty());
        assert!(alerts.is_empty()); // Depending on the patterns, you may want to assert on alerts
        assert!(!utility_names.is_empty());
    }
}
