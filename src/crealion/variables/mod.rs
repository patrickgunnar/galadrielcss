use chrono::Local;
use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    types::Stylitron,
};

use super::{
    utils::generates_variable_or_animation_name::generates_variable_or_animation_name, Crealion,
};

impl Crealion {
    /// Processes the variables associated with a given context and stores them in the STYLITRON AST.
    ///
    /// # Arguments
    /// - `context_name`: A `String` representing the name of the context for the variables.
    /// - `variables_data`: An `IndexMap` where keys are variable identifiers, and values are their corresponding values.
    ///
    /// # Returns
    /// - A `JoinHandle` representing the spawned task. The task processes the variables in a separate thread.
    pub fn process_variables(
        &self,
        context_name: String,
        variables_data: IndexMap<String, String>,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();

        // Spawn a blocking task to process the variables.
        tokio::task::spawn_blocking(move || {
            tracing::info!(
                "Starting variable processing for context: '{}'. Number of variables: {}.",
                context_name,
                variables_data.len()
            );

            // Attempt to retrieve the "variables" section from the STYLITRON AST.
            let mut stylitron_data = match STYLITRON.get_mut("variables") {
                Some(data) => {
                    tracing::debug!(
                        "Successfully accessed the `variables` section in STYLITRON AST."
                    );
                    data
                }
                None => {
                    tracing::error!("Failed to access the `variables` section in STYLITRON AST.");

                    // If the "variables" section is not accessible, create a critical error.
                    let error = GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToStylitronAST,
                        "Failed to access the variables section in STYLITRON AST",
                        ErrorAction::Restart,
                    );

                    tracing::error!("Critical error encountered: {:?}", error);

                    // Generate an error notification and attempt to send it via the sender.
                    let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

                    if let Err(err) = sender.send(notification) {
                        tracing::error!("Failed to send notification: {}", err);
                    }

                    return;
                }
            };

            // Transform the provided variable data into the expected format for STYLITRON.
            let variables = Self::process_variables_data(variables_data, &context_name);

            // Match the `stylitron_data` to ensure it's of the expected type.
            match *stylitron_data {
                // If it's a `Stylitron::Variables` variant, insert or update the context variables.
                Stylitron::Variables(ref mut variables_definitions) => {
                    tracing::info!("Inserting variables into the context: '{}'.", context_name);

                    // Retrieve the context-specific variable map or initialize a new one.
                    let context_variables = variables_definitions
                        .entry(context_name.to_owned())
                        .or_default();

                    // Update the context's variables with the processed data.
                    *context_variables = variables;

                    tracing::info!(
                        "Successfully updated variables for context: '{}'.",
                        context_name
                    );
                }
                _ => {}
            }

            tracing::info!(
                "Completed variable processing for context: '{}'.",
                context_name
            );
        })
    }

    /// Processes variable data by transforming identifiers into unique variable names
    /// and associating them with their corresponding values.
    ///
    /// # Arguments
    /// - `variables_data` - An `IndexMap` containing identifiers as keys and their
    ///   respective values as strings.
    /// - `context_name` - A string slice representing the name of the context
    ///   to ensure the uniqueness of variable names.
    ///
    /// # Returns
    /// An `IndexMap` where each key is the original identifier and the value is
    /// a `Vec<String>` containing:
    /// 1. A unique variable name based on the context and identifier.
    /// 2. The original value associated with the identifier.
    fn process_variables_data(
        variables_data: IndexMap<String, String>,
        context_name: &str,
    ) -> IndexMap<String, Vec<String>> {
        variables_data
            .into_iter()
            // Convert the map into an iterator over (key, value) pairs.
            .map(|(identifier, value)| {
                // Generate a unique variable name based on the context and identifier.
                let unique_var_name =
                    generates_variable_or_animation_name(&context_name, &identifier, true);

                tracing::trace!(
                    "Generated unique variable name '{}' for identifier '{}'.",
                    unique_var_name,
                    identifier
                );

                // Return a pair with the original identifier and a vector
                // containing the unique name and the original value.
                (identifier, vec![unique_var_name, value])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext};
    use tokio::sync::broadcast;

    use crate::{
        asts::STYLITRON,
        crealion::{
            utils::generates_variable_or_animation_name::generates_variable_or_animation_name,
            Crealion,
        },
        events::GaladrielAlerts,
        types::Stylitron,
    };

    fn mock_variables() -> IndexMap<String, String> {
        IndexMap::from([
            ("myVarOne".to_string(), "128px".to_string()),
            ("myVarTwo".to_string(), "#000000".to_string()),
            ("myVarThree".to_string(), "rgb(255, 255, 255)".to_string()),
            ("myVarFour".to_string(), "1024vw".to_string()),
        ])
    }

    fn transform_variables(
        variables: IndexMap<String, String>,
        context_name: &str,
    ) -> IndexMap<String, Vec<String>> {
        variables
            .into_iter()
            .map(|(identifier, value)| {
                let unique_name =
                    generates_variable_or_animation_name(context_name, &identifier, true);

                (identifier, vec![unique_name, value])
            })
            .collect()
    }

    #[tokio::test]
    async fn test_apply_variables_success() {
        let (sender, _) = broadcast::channel(10);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .process_variables("myContextName1".to_string(), mock_variables())
            .await;

        let result = STYLITRON
            .get("variables")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Variables(variables_definitions) => variables_definitions
                    .get("myContextName1")
                    .and_then(|context_variables| Some(context_variables.to_owned())),
                _ => None,
            });

        assert!(result.is_some());

        let variables = result.unwrap();
        let expected_variables = transform_variables(mock_variables(), "myContextName1");

        assert_eq!(variables, expected_variables);
    }

    #[tokio::test]
    async fn test_apply_variables_to_existing_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let (sender, _) = broadcast::channel(10);

        // Pre-populate the STYLITRON AST with existing data.
        let initial_data = IndexMap::from([(
            "myContextName3".to_string(),
            IndexMap::from([(
                "myFakeVar".to_string(),
                vec!["--sm4edk34d".to_string(), "#000".to_string()],
            )]),
        )]);

        STYLITRON.insert("variables".to_string(), Stylitron::Variables(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .process_variables("myContextName3".to_string(), mock_variables())
            .await;

        let result = STYLITRON
            .get("variables")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Variables(variables_definitions) => {
                    variables_definitions.get("myContextName3").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());
        let variables = result.unwrap();
        let expected_variables = transform_variables(mock_variables(), "myContextName3");

        // Verify that the context was updated correctly.
        assert_eq!(variables, expected_variables);
    }

    #[tokio::test]
    async fn test_apply_variables_to_new_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let (sender, _) = broadcast::channel(10);

        // Ensure no existing context in the STYLITRON AST.
        let initial_data = IndexMap::new();
        STYLITRON.insert("variables".to_string(), Stylitron::Variables(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .process_variables("newContextName".to_string(), mock_variables())
            .await;

        let result = STYLITRON
            .get("variables")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Variables(variables_definitions) => {
                    variables_definitions.get("newContextName").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());
        let variables = result.unwrap();
        let expected_variables = transform_variables(mock_variables(), "newContextName");

        // Verify that the new context was added with correct variables.
        assert_eq!(variables, expected_variables);
    }

    #[tokio::test]
    async fn test_apply_variables_no_variables_section() {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let (sender, mut receiver) = broadcast::channel(10);

        // Simulate an empty STYLITRON AST to trigger an error.
        STYLITRON.remove("variables");

        let crealion = Crealion::new(
            sender.clone(),
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .process_variables("noVariablesSection".to_string(), mock_variables())
            .await;

        // Verify that an error notification was sent.
        if let Ok(notification) = receiver.recv().await {
            if let GaladrielAlerts::GaladrielError {
                start_time: _,
                error,
            } = notification
            {
                assert_eq!(
                    error.get_message(),
                    "Failed to access the variables section in STYLITRON AST".to_string()
                );
            }
        } else {
            panic!("Expected an error notification, but none was received.");
        }
    }
}
