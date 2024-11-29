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
