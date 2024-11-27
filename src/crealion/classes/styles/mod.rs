use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{crealion::Crealion, shellscape::alerts::ShellscapeAlerts};

use super::types::UtilityClass;

impl Crealion {
    /// Spawns a new asynchronous task to collect non-responsive styles.
    ///
    /// This function processes style patterns that are not tied to specific breakpoints
    /// and generates utility classes for those styles. It returns a `JoinHandle` for the
    /// spawned task, which resolves to a tuple containing the generated utility classes,
    /// any associated alerts and a vector with the generated utility class names.
    ///
    /// # Parameters
    /// - `inherited_contexts`: A vector of strings representing the inherited style contexts.
    /// - `class_name`: The name of the Nenyr class for which styles are being processed.
    /// - `is_important`: A boolean flag indicating whether the generated styles should have
    ///   `!important` applied.
    /// - `style_patterns`: An optional map containing style patterns. Each pattern is an
    ///   `IndexMap` of properties and their respective values.
    ///
    /// # Returns
    /// A `JoinHandle` for the spawned task. The task resolves to:
    /// - A vector of `UtilityClass` objects representing the generated styles.
    /// - A vector of `ShellscapeAlerts` containing any alerts or warnings generated during processing.
    /// - A vector of `String` containing all the generated utility class names.
    pub fn collect_non_responsive_styles(
        inherited_contexts: Vec<String>,
        class_name: String,
        is_important: bool,
        style_patterns: Option<IndexMap<String, IndexMap<String, String>>>,
    ) -> JoinHandle<(Vec<UtilityClass>, Vec<ShellscapeAlerts>, Vec<String>)> {
        // Spawn an asynchronous task using `tokio::spawn`.
        tokio::spawn(async move {
            tracing::info!(
                "Starting collection of non-responsive styles for class '{}'. Important: {}",
                class_name,
                is_important
            );

            // Initialize a mutable vector to collect alerts.
            let mut alerts: Vec<ShellscapeAlerts> = vec![];
            // Initialize a mutable vector to collect utility classes.
            let mut classes: Vec<UtilityClass> = vec![];
            // Initialize a mutable vector to collect utility class names.
            let mut utility_names: Vec<String> = vec![];

            // Check if there are style patterns to process.
            match style_patterns {
                Some(patterns) => {
                    tracing::debug!(
                        "Processing {} non-responsive style patterns for class '{}'.",
                        patterns.len(),
                        class_name
                    );

                    // If style patterns exist, match them to generate utility classes.
                    // The styles are processed without any breakpoint (`None`).
                    Self::match_style_patterns(
                        &inherited_contexts, // Pass inherited contexts as reference.
                        None,                // No breakpoint for non-responsive styles.
                        &class_name,         // Pass the class name as reference.
                        is_important,        // Propagate the `!important` flag.
                        &mut alerts,         // Collect alerts during processing.
                        &mut classes,        // Collect generated utility classes.
                        &mut utility_names,  // Collect generated utility class names.
                        patterns,            // Provide the style patterns to match.
                    )
                    .await; // Await the asynchronous matching process.

                    tracing::info!(
                        "Finished processing styles for class '{}'. Generated {} utility classes.",
                        class_name,
                        classes.len()
                    );
                }
                None => {
                    tracing::warn!(
                        "No style patterns provided for class '{}'. Skipping processing.",
                        class_name
                    );
                } // If no style patterns are provided, no operation is performed.
            }

            tracing::debug!(
                "Returning results for class '{}': {} utility classes, {} alerts, {} utility names.",
                class_name,
                classes.len(),
                alerts.len(),
                utility_names.len()
            );

            // Return the collected utility classes, alerts and utility class names as a tuple.
            (classes, alerts, utility_names)
        })
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::crealion::{
        mocks::test_helpers::{mock_animations_node, mock_themes_node, mock_variable_node},
        Crealion,
    };

    #[tokio::test]
    async fn test_collect_non_responsive_styles_with_patterns() {
        // Mock data required for the test
        mock_variable_node();
        mock_themes_node();
        mock_animations_node();

        let inherited_contexts = vec!["galaxyContext".to_string()];
        let class_name = "myClass".to_string();
        let is_important = false;

        let style_patterns = Some(IndexMap::from([
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
        ]));

        // Call the function within the runtime
        let result = Crealion::collect_non_responsive_styles(
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
