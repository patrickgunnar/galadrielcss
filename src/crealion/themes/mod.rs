use chrono::Local;
use indexmap::IndexMap;
use nenyr::types::variables::NenyrVariables;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
};

use super::{
    utils::generates_variable_or_animation_name::generates_variable_or_animation_name, Crealion,
};

impl Crealion {
    /// Processes and applies themes to the STYLITRON AST for a specific context.
    ///
    /// # Arguments
    /// - `context_name` - The name of the context to which the themes belong.
    /// - `light_data` - Optional light theme variables.
    /// - `dark_data` - Optional dark theme variables.
    ///
    /// # Returns
    /// A `JoinHandle` that executes the theme processing task asynchronously.
    pub fn process_themes(
        &self,
        context_name: String,
        light_data: Option<NenyrVariables>,
        dark_data: Option<NenyrVariables>,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();

        // Spawn a blocking task to process the themes.
        tokio::spawn(async move {
            tracing::info!(
                "Starting themes processing for context: '{}'.",
                context_name
            );

            // Attempt to retrieve the "themes" section from the STYLITRON AST.
            let mut stylitron_data = match STYLITRON.get_mut("themes") {
                Some(data) => {
                    tracing::debug!("Successfully accessed the `themes` section in STYLITRON AST.");
                    data
                }
                None => {
                    tracing::error!("Failed to access the `themes` section in STYLITRON AST.");

                    // If the "themes" section is not accessible, create a critical error.
                    let error = GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToStylitronAST,
                        "Failed to access the themes section in STYLITRON AST",
                        ErrorAction::Restart,
                    );

                    tracing::error!("Critical error encountered: {:?}", error);

                    // Generate an error notification and attempt to send it via the sender.
                    let notification =
                        ShellscapeAlerts::create_galadriel_error(Local::now(), error);

                    if let Err(err) = sender.send(notification) {
                        tracing::error!("Failed to send notification: {}", err);
                    }

                    return;
                }
            };

            // Process the provided light and dark theme data.
            let light_schema_data = Self::process_theme(light_data, &context_name);
            let dark_schema_data = Self::process_theme(dark_data, &context_name);

            // Match the `stylitron_data` to ensure it's of the expected type.
            match *stylitron_data {
                // If it's a `Stylitron::Themes` variant, insert or update the context themes.
                Stylitron::Themes(ref mut themes_definitions) => {
                    tracing::info!("Inserting themes into the context: '{}'.", context_name);

                    // Retrieve the context-specific themes map or initialize a new one.
                    let context_themes = themes_definitions
                        .entry(context_name.to_owned())
                        .or_default();

                    // Transform the provided themes data into the expected format for STYLITRON.
                    let themes = IndexMap::from([
                        ("light".to_string(), light_schema_data),
                        ("dark".to_string(), dark_schema_data),
                    ]);

                    // Update the context's themes with the processed data.
                    *context_themes = themes;

                    tracing::info!(
                        "Successfully updated themes for context: '{}'.",
                        context_name
                    );
                }
                _ => {}
            }
        })
    }

    /// Transforms and processes theme data into the required format for the STYLITRON AST.
    ///
    /// # Arguments
    /// - `theme_data` - Optional theme variables to be processed.
    /// - `context_name` - The name of the context to which the theme belongs.
    ///
    /// # Returns
    /// A map of processed theme variables with unique variable names.
    fn process_theme(
        theme_data: Option<NenyrVariables>,
        context_name: &str,
    ) -> IndexMap<String, Vec<String>> {
        theme_data
            // Use an empty map if no theme data is provided.
            .map_or_else(IndexMap::new, |v| v.values)
            .into_iter()
            .map(|(identifier, value)| {
                // Generate a unique variable name based on the context and identifier.
                let unique_var_name =
                    generates_variable_or_animation_name(context_name, &identifier, true);

                // Return a pair with the original identifier and a vector
                // containing the unique name and the original value.
                (identifier, vec![unique_var_name, value.to_owned()])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{
        ast::NenyrAst, central::CentralContext, themes::NenyrThemes, variables::NenyrVariables,
    };
    use tokio::sync::mpsc;

    use crate::{
        asts::STYLITRON,
        crealion::{
            utils::generates_variable_or_animation_name::generates_variable_or_animation_name,
            Crealion,
        },
        shellscape::alerts::ShellscapeAlerts,
        types::Stylitron,
    };

    fn mock_light_variables() -> IndexMap<String, String> {
        IndexMap::from([
            ("myVarOne".to_string(), "128px".to_string()),
            ("myVarTwo".to_string(), "#000000".to_string()),
            ("myVarThree".to_string(), "rgb(255, 255, 255)".to_string()),
            ("myVarFour".to_string(), "1024vw".to_string()),
        ])
    }

    fn mock_dark_variables() -> IndexMap<String, String> {
        IndexMap::from([
            ("myVarOne".to_string(), "320px".to_string()),
            ("myVarTwo".to_string(), "#FFFFFF".to_string()),
            ("myVarThree".to_string(), "rgb(0, 0, 0)".to_string()),
            ("myVarFour".to_string(), "128vw".to_string()),
        ])
    }

    fn transform_variables(
        light_variables: IndexMap<String, String>,
        dark_variables: IndexMap<String, String>,
        context_name: &str,
    ) -> IndexMap<String, IndexMap<String, Vec<String>>> {
        let mut light_map = IndexMap::new();
        let mut dark_map = IndexMap::new();

        light_variables.into_iter().for_each(|(identifier, value)| {
            let unique_name = generates_variable_or_animation_name(context_name, &identifier, true);

            light_map.insert(identifier, vec![unique_name, value]);
        });

        dark_variables.into_iter().for_each(|(identifier, value)| {
            let unique_name = generates_variable_or_animation_name(context_name, &identifier, true);

            dark_map.insert(identifier, vec![unique_name, value]);
        });

        IndexMap::from([
            ("light".to_string(), light_map),
            ("dark".to_string(), dark_map),
        ])
    }

    #[tokio::test]
    async fn test_apply_themes_success() {
        let (sender, _) = mpsc::unbounded_channel();

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let nenyr_themes = NenyrThemes {
            light_schema: Some(NenyrVariables {
                values: mock_light_variables(),
            }),
            dark_schema: Some(NenyrVariables {
                values: mock_dark_variables(),
            }),
        };

        let _ = crealion
            .process_themes(
                "myContextName1".to_string(),
                nenyr_themes.light_schema,
                nenyr_themes.dark_schema,
            )
            .await;

        let result = STYLITRON
            .get("themes")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Themes(themes_definitions) => themes_definitions
                    .get("myContextName1")
                    .and_then(|context_themes| Some(context_themes.to_owned())),
                _ => None,
            });

        assert!(result.is_some());

        let themes = result.unwrap();
        let expected_themes = transform_variables(
            mock_light_variables(),
            mock_dark_variables(),
            "myContextName1",
        );

        assert_eq!(themes, expected_themes);
    }

    #[tokio::test]
    async fn test_apply_themes_to_existing_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let (sender, _) = mpsc::unbounded_channel();

        // Pre-populate the STYLITRON AST with existing data.
        let initial_data = IndexMap::from([(
            "myContextName3".to_string(),
            IndexMap::from([(
                "dark".to_string(),
                IndexMap::from([(
                    "myFakeVar".to_string(),
                    vec!["--sm4edk34d".to_string(), "#000".to_string()],
                )]),
            )]),
        )]);

        STYLITRON.insert("themes".to_string(), Stylitron::Themes(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let nenyr_themes = NenyrThemes {
            light_schema: Some(NenyrVariables {
                values: mock_light_variables(),
            }),
            dark_schema: Some(NenyrVariables {
                values: mock_dark_variables(),
            }),
        };

        let _ = crealion
            .process_themes(
                "myContextName3".to_string(),
                nenyr_themes.light_schema,
                nenyr_themes.dark_schema,
            )
            .await;

        let result = STYLITRON
            .get("themes")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Themes(themes_definitions) => {
                    themes_definitions.get("myContextName3").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());
        let themes = result.unwrap();
        let expected_themes = transform_variables(
            mock_light_variables(),
            mock_dark_variables(),
            "myContextName3",
        );

        // Verify that the context was updated correctly.
        assert_eq!(themes, expected_themes);
    }

    #[tokio::test]
    async fn test_apply_themes_to_new_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let (sender, _) = mpsc::unbounded_channel();

        // Ensure no existing context in the STYLITRON AST.
        let initial_data = IndexMap::new();
        STYLITRON.insert("themes".to_string(), Stylitron::Themes(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let nenyr_themes = NenyrThemes {
            light_schema: Some(NenyrVariables {
                values: mock_light_variables(),
            }),
            dark_schema: Some(NenyrVariables {
                values: mock_dark_variables(),
            }),
        };

        let _ = crealion
            .process_themes(
                "newContextName".to_string(),
                nenyr_themes.light_schema,
                nenyr_themes.dark_schema,
            )
            .await;

        let result = STYLITRON
            .get("themes")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Themes(themes_definitions) => {
                    themes_definitions.get("newContextName").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());
        let themes = result.unwrap();
        let expected_themes = transform_variables(
            mock_light_variables(),
            mock_dark_variables(),
            "newContextName",
        );

        // Verify that the new context was added with correct themes.
        assert_eq!(themes, expected_themes);
    }

    #[tokio::test]
    async fn test_apply_themes_with_empty_themes_data() {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let (sender, _) = mpsc::unbounded_channel();

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let nenyr_themes = NenyrThemes {
            light_schema: Some(NenyrVariables {
                values: IndexMap::new(),
            }),
            dark_schema: Some(NenyrVariables {
                values: IndexMap::new(),
            }),
        };

        let _ = crealion
            .process_themes(
                "emptyThemesContext".to_string(),
                nenyr_themes.light_schema,
                nenyr_themes.dark_schema,
            )
            .await;

        let result = STYLITRON
            .get("themes")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Themes(themes_definitions) => {
                    themes_definitions.get("emptyThemesContext").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());

        let themes = result.unwrap();
        let empty_themes: IndexMap<String, IndexMap<String, Vec<String>>> = IndexMap::from([
            ("light".to_string(), IndexMap::new()),
            ("dark".to_string(), IndexMap::new()),
        ]);

        // Verify that the context was added but remains empty.
        assert_eq!(themes, empty_themes);
    }

    #[tokio::test]
    async fn test_apply_themes_no_themes_section() {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let (sender, mut receiver) = mpsc::unbounded_channel();

        // Simulate an empty STYLITRON AST to trigger an error.
        STYLITRON.remove("themes");

        let crealion = Crealion::new(
            sender.clone(),
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let nenyr_themes = NenyrThemes {
            light_schema: Some(NenyrVariables {
                values: mock_light_variables(),
            }),
            dark_schema: Some(NenyrVariables {
                values: mock_dark_variables(),
            }),
        };

        let _ = crealion
            .process_themes(
                "noAliasSection".to_string(),
                nenyr_themes.light_schema,
                nenyr_themes.dark_schema,
            )
            .await;

        // Verify that an error notification was sent.
        if let Some(notification) = receiver.recv().await {
            if let ShellscapeAlerts::GaladrielError {
                start_time: _,
                error,
            } = notification
            {
                assert_eq!(
                    error.get_message(),
                    "Failed to access the themes section in STYLITRON AST".to_string()
                );
            }
        } else {
            panic!("Expected an error notification, but none was received.");
        }
    }
}
