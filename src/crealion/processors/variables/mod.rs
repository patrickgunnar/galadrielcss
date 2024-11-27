use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use regex::Regex;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    crealion::utils::{camelify::camelify, pascalify::pascalify},
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
    GaladrielResult,
};

/// Defines the available schemas for styling.
const SCHEMAS: &[&str] = &["light", "dark"];

/// A struct to process variables within styles.
/// This processor resolves variables and replaces them with their respective values.
#[derive(Clone, Debug)]
pub struct VariablesProcessor {
    /// Regular expression used to identify variable patterns in strings.
    re: Regex,
    /// List of inherited contexts used for variable resolution.
    inherited_contexts: Vec<String>,
}

impl VariablesProcessor {
    /// Constructs a new `VariablesProcessor`.
    ///
    /// # Parameters
    /// - `inherited_contexts`: A vector of contexts that influence variable resolution.
    ///
    /// # Returns
    /// A `GaladrielResult` containing the new `VariablesProcessor` or an error if the regex compilation fails.
    pub fn new(inherited_contexts: Vec<String>) -> GaladrielResult<Self> {
        tracing::info!(
            "Initializing VariablesProcessor with contexts: {:?}",
            inherited_contexts
        );

        // Initialize the regex for capturing variables, raising an error if the pattern is invalid.
        let re = Regex::new(r"\$\{(.*?)\}").map_err(|err| {
            tracing::error!("Failed to compile regex for variable resolution: {}", err);

            GaladrielError::raise_general_other_error(
                ErrorKind::Other,
                &err.to_string(),
                ErrorAction::Notify,
            )
        })?;

        tracing::debug!("VariablesProcessor successfully initialized.");

        Ok(Self {
            inherited_contexts,
            re,
        })
    }

    /// Resolves variables within a given value string.
    ///
    /// # Parameters
    /// - `value`: The string containing variables to resolve.
    /// - `property`: The property name related to the value.
    /// - `class_name`: The class name using the value.
    /// - `pattern`: The pattern name associated with the value.
    /// - `breakpoint`: Optional breakpoint context for the style.
    /// - `alerts`: A mutable reference to a vector for collecting alerts or warnings.
    ///
    /// # Returns
    /// A `GaladrielResult` containing the resolved string or `None` if the resolution fails.
    pub async fn process(
        &self,
        value: &str,
        property: &str,
        class_name: &str,
        pattern: &str,
        breakpoint: &Option<String>,
        alerts: &mut Vec<ShellscapeAlerts>,
    ) -> GaladrielResult<Option<String>> {
        tracing::info!(
            "Starting variable resolution for value: `{}` in class `{}`, property `{}`.",
            value,
            class_name,
            property
        );

        // Start with the original value and an index offset for replacements.
        let mut resolved_value = value.to_string();
        let mut offset_index = 0;

        // Iterate through all variable captures in the value string.
        for capture in self.re.captures_iter(value) {
            // Extract the variable name from the capture group.
            let relative_name = &capture[1].to_string();
            tracing::debug!("Processing variable: `{}`.", relative_name);

            // Concurrently attempt to resolve the variable from multiple nodes.
            let results = join_all(vec![
                self.process_from_variables_node(relative_name.to_owned()),
                self.process_from_themes_node(relative_name.to_owned()),
                self.process_from_animations_node(relative_name.to_owned()),
            ])
            .await;

            // Find the first successful resolution result or report an error.
            let resolved_result = results.iter().find_map(|result| match result {
                Ok(Some(v)) => Some(Ok(format!("var({})", v))),
                Err(err) => Some(Err(err.to_string())),
                Ok(None) => None,
            });

            match resolved_result {
                Some(Ok(replacement_value)) => {
                    tracing::info!(
                        "Variable `{}` resolved to `{}`.",
                        relative_name,
                        replacement_value
                    );

                    // Compute the positions of the variable within the string.
                    let (start_pos, end_pos) = self.get_positions(capture)?;

                    // Replace the variable with its resolved value.
                    resolved_value.replace_range(
                        start_pos.saturating_add(offset_index)
                            ..end_pos.saturating_add(offset_index),
                        &replacement_value,
                    );

                    // Adjust the offset index based on the replacement length.
                    let adjustment = if end_pos <= start_pos {
                        end_pos.saturating_sub(start_pos)
                    } else {
                        0
                    };

                    offset_index += replacement_value.len().saturating_sub(adjustment);
                }
                Some(Err(err)) => {
                    tracing::error!("Failed to resolve variable `{}`: {}.", relative_name, err);

                    // Raise an error if variable resolution fails.
                    return Err(GaladrielError::raise_general_other_error(
                        ErrorKind::TaskFailure,
                        &err,
                        ErrorAction::Notify,
                    ));
                }
                None => {
                    tracing::warn!(
                        "Variable `{}` could not be resolved in contexts: {:?}.",
                        relative_name,
                        self.inherited_contexts
                    );

                    // If no resolution is found, log a warning and return `None`.
                    let formatted_property = camelify(&property);
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
                        "The `{}` property in the `{}` class for the `{}` pattern references an unrecognized `{}` variable, preventing the style from being created. Please review and update the variable to ensure the style is generated correctly.{}",
                        formatted_property, class_name, formatted_pattern, relative_name, panoramic_message
                    );

                    // Add the warning to the alerts vector.
                    alerts.insert(0, ShellscapeAlerts::create_warning(Local::now(), &message));
                    return Ok(None);
                }
            }
        }

        tracing::info!(
            "Variable resolution completed for value: `{}`. Resolved value: `{}`.",
            value,
            resolved_value
        );

        Ok(Some(resolved_value))
    }

    /// Processes a variable reference by looking it up in the "variables" node of STYLITRON.
    ///
    /// # Parameters
    /// - `relative_name`: The name of the variable to resolve.
    ///
    /// # Returns
    /// A [`JoinHandle`] for a task that resolves to an optional string containing the resolved variable value.
    fn process_from_variables_node(&self, relative_name: String) -> JoinHandle<Option<String>> {
        tracing::debug!(
            "Spawning task to resolve variable `{}` from variables node.",
            relative_name
        );

        // Clone the inherited contexts to be moved into the async block.
        let inherited_contexts = self.inherited_contexts.clone();

        // Spawns a blocking task for resolving the variable.
        tokio::task::spawn_blocking(move || {
            // Retrieve the "variables" node from STYLITRON and process it.
            STYLITRON
                .get("variables")
                .and_then(|stylitron_data| match &*stylitron_data {
                    // Search through the inherited contexts for the variable definition.
                    Stylitron::Variables(variables_definitions) => {
                        inherited_contexts.iter().find_map(|context_name| {
                            variables_definitions
                                .get(context_name)
                                .and_then(|context_variables| {
                                    // Retrieve the resolved variable name, if available.
                                    context_variables.get(&relative_name).and_then(
                                        |variable_entry| {
                                            variable_entry.get_index(0).map(|(resolved_name, _)| {
                                                tracing::debug!(
                                                    "Resolved variable `{}` to `{}`.",
                                                    relative_name,
                                                    resolved_name
                                                );

                                                resolved_name.to_owned()
                                            })
                                        },
                                    )
                                })
                        })
                    }
                    _ => None, // Return None if the "variables" node is not properly formatted.
                })
        })
    }

    /// Processes a variable reference by looking it up in the "themes" node of STYLITRON.
    ///
    /// # Parameters
    /// - `relative_name`: The name of the theme variable to resolve.
    ///
    /// # Returns
    /// A [`JoinHandle`] for a task that resolves to an optional string containing the resolved theme value.
    fn process_from_themes_node(&self, relative_name: String) -> JoinHandle<Option<String>> {
        tracing::debug!(
            "Spawning task to resolve variable `{}` from themes node.",
            relative_name
        );

        // Clone the inherited contexts to be moved into the async block.
        let inherited_contexts = self.inherited_contexts.clone();

        // Spawns a blocking task for resolving the theme variable.
        tokio::task::spawn_blocking(move || {
            // Retrieve the "themes" node from STYLITRON and process it.
            STYLITRON
                .get("themes")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Themes(themes_definitions) => {
                        // Iterate through inherited contexts to find the theme definition.
                        inherited_contexts.iter().find_map(|context_name| {
                            themes_definitions
                                .get(context_name)
                                .and_then(|context_themes| {
                                    // Check each schema (e.g., "light" or "dark") for the variable.
                                    SCHEMAS.iter().find_map(|schema| {
                                        Self::process_theme_schema(
                                            schema,
                                            &relative_name,
                                            context_themes,
                                        )
                                    })
                                })
                        })
                    }
                    _ => None, // Return None if the "themes" node is not properly formatted.
                })
        })
    }

    /// Processes a specific theme schema for a variable reference.
    ///
    /// # Parameters
    /// - `schema`: The name of the schema (e.g., "light" or "dark").
    /// - `relative_name`: The name of the theme variable to resolve.
    /// - `context_themes`: The themes context to search within.
    ///
    /// # Returns
    /// An optional string containing the resolved theme value.
    fn process_theme_schema(
        schema: &str,
        relative_name: &str,
        context_themes: &IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
    ) -> Option<String> {
        // Check if the schema exists in the context themes.
        context_themes.get(schema).and_then(|schema_variables| {
            // Retrieve the resolved variable name, if available.
            schema_variables
                .get(relative_name)
                .and_then(|variable_entry| {
                    variable_entry.get_index(0).map(|(resolved_name, _)| {
                        tracing::debug!(
                            "Resolved theme variable `{}` to `{}`.",
                            relative_name,
                            resolved_name
                        );

                        resolved_name.to_owned()
                    })
                })
        })
    }

    /// Processes a variable reference by looking it up in the "animations" node of STYLITRON.
    ///
    /// # Parameters
    /// - `relative_name`: The name of the animation variable to resolve.
    ///
    /// # Returns
    /// A [`JoinHandle`] for a task that resolves to an optional string containing the resolved animation value.
    fn process_from_animations_node(&self, relative_name: String) -> JoinHandle<Option<String>> {
        tracing::debug!(
            "Spawning task to resolve animation `{}` from animations node.",
            relative_name
        );

        // Clone the inherited contexts to be moved into the async block.
        let inherited_contexts = self.inherited_contexts.clone();

        // Spawns a blocking task for resolving the animation variable.
        tokio::task::spawn_blocking(move || {
            // Retrieve the "animations" node from STYLITRON and process it.
            STYLITRON
                .get("animations")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Animation(animations_definitions) => {
                        // Iterate through inherited contexts to find the animation definition.
                        inherited_contexts.iter().find_map(|context_name| {
                            animations_definitions.get(context_name).and_then(
                                |context_animations| {
                                    // Retrieve the resolved animation name, if available.
                                    context_animations.get(&relative_name).and_then(
                                        |animation_entry| {
                                            animation_entry.get_index(0).map(
                                                |(resolved_name, _)| {
                                                    tracing::debug!(
                                                        "Resolved animation `{}` to `{}`.",
                                                        relative_name,
                                                        resolved_name
                                                    );

                                                    resolved_name.to_owned()
                                                },
                                            )
                                        },
                                    )
                                },
                            )
                        })
                    }
                    _ => None, // Return None if the "animations" node is not properly formatted.
                })
        })
    }

    /// Retrieves the start and end positions of a regex capture group.
    ///
    /// # Parameters
    /// - `capture`: A [`regex::Captures`] object containing the matched groups.
    ///
    /// # Returns
    /// A [`GaladrielResult`] containing a tuple of `(start, end)` positions for the capture group.
    fn get_positions(&self, capture: regex::Captures<'_>) -> GaladrielResult<(usize, usize)> {
        // Retrieve the positions for the first capture group (index 0).
        capture
            .get(0)
            .and_then(|cap| Some((cap.start(), cap.end())))
            .ok_or_else(|| {
                tracing::error!(
                    "Failed to retrieve capture group positions: capture group 0 not found."
                );

                GaladrielError::raise_general_other_error(
                    ErrorKind::Other,
                    "Failed to retrieve capture group positions: capture group 0 not found.",
                    ErrorAction::Notify,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::crealion::mocks::test_helpers::{
        mock_animations_node, mock_themes_node, mock_variable_node,
    };

    use super::VariablesProcessor;

    use regex::Regex;

    #[tokio::test]
    async fn test_new_initialization() {
        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts.clone()).expect("Failed to initialize");

        // Validate regex
        assert_eq!(processor.re.as_str(), r"\$\{(.*?)\}");

        // Validate inherited contexts
        assert_eq!(processor.inherited_contexts, contexts);
    }

    #[tokio::test]
    async fn test_process_from_variable_resolution_success_on_first_position_context() {
        // Mock data
        mock_variable_node();

        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts).unwrap();

        let mut alerts = Vec::new();
        let resolved_value = processor
            .process(
                "${primaryColor}",
                "background-color",
                "myClassName",
                ":hover",
                &None,
                &mut alerts,
            )
            .await
            .expect("Failed to process")
            .unwrap();

        // Assert resolved value (mock the result from the mocked functions)
        assert_eq!(resolved_value, "var(s7sj3d)");

        // Ensure no alerts are generated
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_process_from_variable_resolution_success_on_second_position_context() {
        // Mock data
        mock_variable_node();

        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts).unwrap();

        let mut alerts = Vec::new();
        let resolved_value = processor
            .process(
                "${galaxyForegroundColor}",
                "background-color",
                "myClassName",
                ":hover",
                &None,
                &mut alerts,
            )
            .await
            .expect("Failed to process")
            .unwrap();

        // Assert resolved value (mock the result from the mocked functions)
        assert_eq!(resolved_value, "var(d8373jd79)");

        // Ensure no alerts are generated
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_process_from_animation_resolution_success_on_first_position_context() {
        // Mock data
        mock_animations_node();

        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts).unwrap();

        let mut alerts = Vec::new();
        let resolved_value = processor
            .process(
                "${mySecondaryAnimation}",
                "animation-name",
                "myClassName",
                "_stylesheet",
                &None,
                &mut alerts,
            )
            .await
            .expect("Failed to process")
            .unwrap();

        // Assert resolved value (mock the result from the mocked functions)
        assert_eq!(resolved_value, "var(ch8725sdw2cs5w)");

        // Ensure no alerts are generated
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_process_from_animation_resolution_success_on_second_position_context() {
        // Mock data
        mock_animations_node();

        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts).unwrap();

        let mut alerts = Vec::new();
        let resolved_value = processor
            .process(
                "${myUniqueAnimation}",
                "animation-name",
                "myClassName",
                "_stylesheet",
                &None,
                &mut alerts,
            )
            .await
            .expect("Failed to process")
            .unwrap();

        // Assert resolved value (mock the result from the mocked functions)
        assert_eq!(resolved_value, "var(d72jd5fkw54k5w)");

        // Ensure no alerts are generated
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_process_from_theme_resolution_success_on_first_position_context() {
        // Mock data
        mock_themes_node();

        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts).unwrap();

        let mut alerts = Vec::new();
        let resolved_value = processor
            .process(
                "${secondaryColor}",
                "animation-name",
                "myClassName",
                "_stylesheet",
                &None,
                &mut alerts,
            )
            .await
            .expect("Failed to process")
            .unwrap();

        // Assert resolved value (mock the result from the mocked functions)
        assert_eq!(resolved_value, "var(ste62jh)");

        // Ensure no alerts are generated
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_process_from_theme_resolution_success_on_second_position_context() {
        // Mock data
        mock_themes_node();

        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts).unwrap();

        let mut alerts = Vec::new();
        let resolved_value = processor
            .process(
                "${galaxyForegroundColor}",
                "animation-name",
                "myClassName",
                "_stylesheet",
                &None,
                &mut alerts,
            )
            .await
            .expect("Failed to process")
            .unwrap();

        // Assert resolved value (mock the result from the mocked functions)
        assert_eq!(resolved_value, "var(d8373jd79)");

        // Ensure no alerts are generated
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_process_variable_resolution_failure() {
        // Mock data
        mock_variable_node();

        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = VariablesProcessor::new(contexts).unwrap();

        let mut alerts = Vec::new();
        let resolved_value = processor
            .process(
                "${unknown_var}",
                "background-color",
                "myClassName",
                ":hover",
                &None,
                &mut alerts,
            )
            .await
            .unwrap();

        // Assert no value resolved
        assert!(resolved_value.is_none());

        // Assert an alert was generated
        assert_eq!(alerts.len(), 1);
    }

    #[tokio::test]
    async fn test_regex_failure() {
        let result = Regex::new("[invalid(");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("regex parse error"));
    }
}
