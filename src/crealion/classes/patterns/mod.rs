use chrono::Local;
use indexmap::IndexMap;

use crate::{
    crealion::{
        processors::{
            breakpoint::BreakpointProcessor, nickname::NicknameProcessor,
            variables::VariablesProcessor,
        },
        utils::{generate_utility_class_name::generate_utility_class_name, pascalify::pascalify},
        Crealion,
    },
    shellscape::alerts::ShellscapeAlerts,
};

use super::types::UtilityClass;

impl Crealion {
    /// Matches style patterns to generate utility classes based on the provided context and patterns.
    ///
    /// This function processes style patterns and properties, applies context-specific variables,
    /// and generates utility classes. Alerts are raised for errors or warnings encountered during the process.
    ///
    /// # Arguments
    /// - `inherited_contexts`: A vector of inherited context names used to resolve variables, animations and aliases.
    /// - `breakpoint`: An optional breakpoint value that influences the generated class name.
    /// - `class_name`: The name of the Nenyr class being processed.
    /// - `is_important`: A boolean indicating whether the styles should be marked as `!important`.
    /// - `alerts`: A mutable vector for storing alerts generated during processing.
    /// - `classes`: A mutable vector for storing the resulting utility classes.
    /// - `utility_names`: A mutable vector for storing the utility class names.
    /// - `patterns`: A map of style patterns and their associated properties and values.
    #[tracing::instrument(
        skip(inherited_contexts, alerts, classes, utility_names, patterns),
        fields(class_name = %class_name, breakpoint = ?breakpoint, is_important = is_important)
    )]
    pub async fn match_style_patterns(
        inherited_contexts: &Vec<String>,
        breakpoint: Option<String>,
        class_name: &str,
        is_important: bool,
        alerts: &mut Vec<ShellscapeAlerts>,
        classes: &mut Vec<UtilityClass>,
        utility_names: &mut Vec<String>,
        patterns: IndexMap<String, IndexMap<String, String>>,
    ) {
        tracing::info!("Starting match_style_patterns for class {}", class_name);

        // Create a variables processor for resolving variables within the context.
        let variables_processor = match VariablesProcessor::new(inherited_contexts.to_vec()) {
            Ok(processor) => {
                tracing::debug!("Variables processor created successfully.");
                processor
            }
            Err(err) => {
                tracing::error!("Failed to create variables processor: {:?}", err);

                // Log an error alert if the processor creation fails.
                alerts.insert(
                    0,
                    ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                );

                return;
            }
        };

        // Resolve the breakpoint value to a CSS-compatible format, if provided.
        let breakpoint_value = breakpoint
            .as_ref()
            .map(|b| BreakpointProcessor::new(b).process())
            .unwrap_or(None);

        tracing::debug!(?breakpoint_value, "Breakpoint resolved.");

        // Create a nickname processor for resolving style aliases.
        let nickname_processor = NicknameProcessor::new(inherited_contexts.to_vec());

        tracing::debug!("Nickname processor initialized.");

        // Iterate through each pattern and its associated styles.
        for (pattern, style) in patterns {
            tracing::debug!(?pattern, "Processing pattern.");

            for (property, value) in style {
                tracing::trace!(?property, ?value, "Processing style property.");

                // Process the style property and resolve its alias.
                match Self::process_style_property(
                    &property,
                    class_name,
                    &pattern,
                    &breakpoint,
                    alerts,
                    &nickname_processor,
                ) {
                    // Process variables and generate a utility class for valid styles.
                    Ok(new_property) => match variables_processor
                        .process(
                            &value,
                            &property,
                            class_name,
                            &pattern,
                            &breakpoint,
                            alerts,
                            false,
                        )
                        .await
                    {
                        Ok(Some(new_value)) => {
                            tracing::debug!(
                                new_pattern = pascalify(&pattern),
                                new_property = %new_property,
                                new_value = %new_value,
                                "Generated utility class."
                            );

                            // Trim unnecessary suffixes from the pattern name.
                            let new_pattern = pattern.trim_end_matches("stylesheet");
                            // Generate the utility class name.
                            let utility_cls_name = generate_utility_class_name(
                                &breakpoint,
                                is_important,
                                new_pattern,
                                &new_property,
                                &new_value,
                            );

                            // Create and store the utility class.
                            classes.push(UtilityClass::create_class(
                                &breakpoint_value,
                                new_pattern,
                                &utility_cls_name,
                                is_important,
                                &new_property,
                                &new_value,
                            ));

                            // Save the utility class name into the the utility names vector.
                            utility_names.push(utility_cls_name);
                        }
                        Err(err) => {
                            tracing::error!(
                                "Failed to process variables for property `{}`: {:?}",
                                property,
                                err
                            );

                            // Log an error alert for failed variable processing.
                            alerts.insert(
                                0,
                                ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                            );
                        }
                        Ok(None) => {
                            tracing::warn!(
                                "Variable processor returned None for property `{}`.",
                                property
                            );
                        } // Skip processing if no value is returned.
                    },
                    Err(_) => {
                        tracing::warn!(
                            "Skipping unresolved property alias `{}` in pattern `{}`.",
                            property,
                            pascalify(&pattern)
                        );
                    } // Skip processing if the property alias cannot be resolved.
                }
            }
        }

        tracing::info!("Completed match_style_patterns for class {}", class_name);
    }

    /// Processes a style property alias and resolves it to a recognized property name.
    ///
    /// If the alias starts with `"nickname;"`, it is processed as a style alias using the nickname processor.
    /// Alerts are raised if the alias cannot be resolved, and an error is returned.
    ///
    /// # Arguments
    /// - `alias`: The property alias to process.
    /// - `class_name`: The name of the Nenyr class being processed.
    /// - `pattern`: The pattern in which the alias is defined.
    /// - `breakpoint`: An optional breakpoint value.
    /// - `alerts`: A mutable vector for storing alerts.
    /// - `nickname_processor`: The nickname processor for resolving aliases.
    ///
    /// # Returns
    /// - `Ok(String)`: The resolved property name.
    /// - `Err(())`: An error if the alias cannot be resolved.
    #[tracing::instrument(
        skip(alerts, nickname_processor),
        fields(alias = %alias, class_name = %class_name, pattern = %pattern, breakpoint = ?breakpoint)
    )]
    fn process_style_property(
        alias: &str,
        class_name: &str,
        pattern: &str,
        breakpoint: &Option<String>,
        alerts: &mut Vec<ShellscapeAlerts>,
        nickname_processor: &NicknameProcessor,
    ) -> Result<String, ()> {
        // Check if the alias is a nickname alias.
        if alias.starts_with("nickname;") {
            let alias_value = alias.trim_start_matches("nickname;");
            tracing::debug!("Processing nickname alias: {}", alias_value);

            // Attempt to process the alias using the nickname processor.
            match nickname_processor.process(alias_value) {
                Some(processed_alias) => {
                    tracing::debug!(
                        "Resolved nickname alias `{}` to `{}`.",
                        alias_value,
                        processed_alias
                    );

                    return Ok(processed_alias);
                }
                None => {
                    // Generate a formatted warning message for unresolved aliases.
                    let formatted_pattern = pascalify(&pattern);
                    let panoramic_message = breakpoint
                        .as_ref()
                        .map(|name| {
                            format!(
                            " This occurred in the `{}` breakpoint of the `PanoramicViewer` method.",
                            name
                        )
                        })
                        .unwrap_or_default();

                    let message = format!(
                        "The alias `{}` in the `{}` class for the `{}` pattern was not recognized. The style could not be created for this value. Please review and update the alias to ensure the style is generated correctly.{}",
                        alias_value, class_name, formatted_pattern, panoramic_message
                    );

                    tracing::warn!(
                        "Unresolved nickname alias: `{}`. Alert: {}",
                        alias_value,
                        message
                    );

                    // Log a warning alert for the unresolved alias.
                    alerts.insert(0, ShellscapeAlerts::create_warning(Local::now(), &message));
                    return Err(());
                }
            }
        }

        // Return the alias as-is if it does not require special processing.
        Ok(alias.to_string())
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
    async fn test_match_style_patterns_valid_from_variables() {
        // Arrange: Setup mock data
        mock_variable_node(); // Initializes mock variables
        mock_themes_node(); // Initializes mock themes

        let inherited_contexts = vec!["myGlacialContext".to_string()];
        let breakpoint = Some("myMob01".to_string());
        let class_name = "myClass";
        let is_important = false;

        let mut alerts = Vec::new();
        let mut classes = Vec::new();
        let mut utility_names = Vec::new();

        let patterns = IndexMap::from([(
            "_stylesheet".to_string(),
            IndexMap::from([("color".to_string(), "${primaryColor}".to_string())]),
        )]);

        // Act: Call the function
        Crealion::match_style_patterns(
            &inherited_contexts,
            breakpoint,
            class_name,
            is_important,
            &mut alerts,
            &mut classes,
            &mut utility_names,
            patterns,
        )
        .await;

        // Assert: Check if utility class is generated and alerts are empty
        assert_eq!(classes.len(), 1); // Expect 1 class
        assert!(alerts.is_empty()); // No errors should occur
        assert_eq!(utility_names.len(), 1); // 1 utility class name should be generated
    }

    #[tokio::test]
    async fn test_match_style_patterns_valid_from_themes() {
        // Arrange: Setup mock data
        mock_variable_node(); // Initializes mock variables
        mock_themes_node(); // Initializes mock themes

        let inherited_contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let breakpoint = Some("myMob01".to_string());
        let class_name = "myClass";
        let is_important = false;

        let mut alerts = Vec::new();
        let mut classes = Vec::new();
        let mut utility_names = Vec::new();

        let patterns = IndexMap::from([(
            "_stylesheet".to_string(),
            IndexMap::from([(
                "color".to_string(),
                "${galaxyForegroundColorThemed}".to_string(),
            )]),
        )]);

        // Act: Call the function
        Crealion::match_style_patterns(
            &inherited_contexts,
            breakpoint,
            class_name,
            is_important,
            &mut alerts,
            &mut classes,
            &mut utility_names,
            patterns,
        )
        .await;

        // Assert: Check if utility class is generated and alerts are empty
        assert_eq!(classes.len(), 1); // Expect 1 class
        assert!(alerts.is_empty()); // No errors should occur
        assert_eq!(utility_names.len(), 1); // 1 utility class name should be generated
    }

    #[tokio::test]
    async fn test_match_style_patterns_valid_from_animations() {
        // Arrange: Setup mock data
        mock_variable_node(); // Initializes mock variables
        mock_themes_node(); // Initializes mock themes
        mock_animations_node();

        let inherited_contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let breakpoint = Some("myMob01".to_string());
        let class_name = "myClass";
        let is_important = false;

        let mut alerts = Vec::new();
        let mut classes = Vec::new();
        let mut utility_names = Vec::new();

        let patterns = IndexMap::from([(
            "_stylesheet".to_string(),
            IndexMap::from([(
                "animation-name".to_string(),
                "${myUniqueAnimation}".to_string(),
            )]),
        )]);

        // Act: Call the function
        Crealion::match_style_patterns(
            &inherited_contexts,
            breakpoint,
            class_name,
            is_important,
            &mut alerts,
            &mut classes,
            &mut utility_names,
            patterns,
        )
        .await;

        // Assert: Check if utility class is generated and alerts are empty
        assert_eq!(classes.len(), 1); // Expect 1 class
        assert!(alerts.is_empty()); // No errors should occur
        assert_eq!(utility_names.len(), 1); // 1 utility class name should be generated
    }

    // Test for handling unresolved alias
    #[tokio::test]
    async fn test_match_style_patterns_unresolved_alias() {
        // Arrange: Setup mock data
        mock_variable_node();
        mock_themes_node();

        let inherited_contexts = vec!["myGlacialContext".to_string()];
        let breakpoint = Some("myDesk01".to_string());
        let class_name = "myClass";
        let is_important = false;

        let mut alerts = Vec::new();
        let mut classes = Vec::new();
        let mut utility_names = Vec::new();

        let patterns = IndexMap::from([(
            ":hover".to_string(),
            IndexMap::from([("nickname;unknownAlias".to_string(), "#123456".to_string())]),
        )]);

        // Act: Call the function
        Crealion::match_style_patterns(
            &inherited_contexts,
            breakpoint,
            class_name,
            is_important,
            &mut alerts,
            &mut classes,
            &mut utility_names,
            patterns,
        )
        .await;

        // Assert: Check if error alert was generated and no utility class was created
        assert!(alerts.len() > 0); // An alert should have been generated for the alias issue
        assert_eq!(classes.len(), 0); // No classes should be created for unresolved alias
        assert_eq!(utility_names.len(), 0); // No utility names should be generated
    }

    // Test for edge case with empty patterns
    #[tokio::test]
    async fn test_match_style_patterns_empty_patterns() {
        // Arrange: Setup mock data
        mock_variable_node();
        mock_themes_node();

        let inherited_contexts = vec!["myGlacialContext".to_string()];
        let breakpoint = None;
        let class_name = "emptyClass";
        let is_important = true;

        let mut alerts = Vec::new();
        let mut classes = Vec::new();
        let mut utility_names = Vec::new();

        let patterns = IndexMap::new(); // Empty patterns

        // Act: Call the function
        Crealion::match_style_patterns(
            &inherited_contexts,
            breakpoint,
            class_name,
            is_important,
            &mut alerts,
            &mut classes,
            &mut utility_names,
            patterns,
        )
        .await;

        // Assert: Ensure that no utility class is generated for empty patterns
        assert_eq!(classes.len(), 0); // No utility classes should be created
        assert!(alerts.is_empty()); // No alerts should be raised
        assert_eq!(utility_names.len(), 0); // No utility names should be generated
    }

    // Test for error in variable processing
    #[tokio::test]
    async fn test_match_style_patterns_variable_error() {
        // Arrange: Setup mock data with invalid variable references
        mock_variable_node(); // Initialize valid variables
        let inherited_contexts = vec!["myGlacialContext".to_string()];
        let breakpoint = Some("myMob02".to_string());
        let class_name = "errorClass";
        let is_important = false;

        let mut alerts = Vec::new();
        let mut classes = Vec::new();
        let mut utility_names = Vec::new();

        let patterns = IndexMap::from([(
            "_stylesheet".to_string(),
            IndexMap::from([("color".to_string(), "${nonExistentVariable}".to_string())]),
        )]);

        // Act: Call the function
        Crealion::match_style_patterns(
            &inherited_contexts,
            breakpoint,
            class_name,
            is_important,
            &mut alerts,
            &mut classes,
            &mut utility_names,
            patterns,
        )
        .await;

        // Assert: Check if error alert is raised for missing variable
        assert!(alerts.len() > 0); // An alert should be raised due to the unresolved variable
        assert_eq!(classes.len(), 0); // No classes should be created due to error
        assert_eq!(utility_names.len(), 0); // No utility names should be generated
    }
}
