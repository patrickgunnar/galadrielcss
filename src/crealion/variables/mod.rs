use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use nenyr::types::variables::NenyrVariables;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
    GaladrielResult,
};

use super::{
    utils::generates_variable_or_animation_name::generates_variable_or_animation_name, Crealion,
};

impl Crealion {
    /// Processes a set of variables asynchronously and applies them to the Stylitron AST.
    ///
    /// # Parameters
    /// - `context`: A `String` representing the context in which the variables are processed.
    /// - `variables`: An optional `NenyrVariables` struct containing the variables to be processed.
    ///
    /// # Returns
    /// Returns a `JoinHandle<Vec<ShellscapeAlerts>>`, which is the handle for the asynchronous task
    /// that processes the variables and collects any alerts that may be generated during the process.
    pub fn process_variables(
        &self,
        context: String,
        variables: Option<NenyrVariables>,
    ) -> JoinHandle<Vec<ShellscapeAlerts>> {
        tokio::task::spawn(async move {
            let mut alerts: Vec<ShellscapeAlerts> = vec![];

            tracing::info!(context = %context, "Starting process_variables");

            // If variables are provided, proceed to process them
            let variables_map = if let Some(variables_definitions) = variables {
                tracing::info!(
                    variables_count = variables_definitions.values.len(),
                    "Processing {} variables",
                    variables_definitions.values.len()
                );

                // Spawn tasks to process each variable concurrently
                let variables_futures = join_all(variables_definitions.values.iter().map(
                    |(identifier, value)| {
                        Self::process_variable(
                            context.clone(),
                            identifier.to_owned(),
                            value.to_owned(),
                        )
                    },
                ))
                .await;

                // Collect results from the futures and handle errors
                Self::extract_futures(&mut alerts, variables_futures)
            } else {
                IndexMap::new()
            };

            // Apply the successfully processed variables to the Stylitron AST
            if let Err(err) = Self::apply_variables_to_stylitron(context, variables_map) {
                tracing::error!(%err, "Failed to apply variables to Stylitron");

                alerts.insert(
                    0,
                    ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                );
            } else {
                tracing::info!("Successfully applied variables to Stylitron");
            }

            tracing::info!(
                alerts_count = alerts.len(),
                "process_variables finished with {} alerts",
                alerts.len()
            );

            alerts
        })
    }

    /// Applies a set of variables to the Stylitron AST under the given context.
    ///
    /// # Parameters
    /// - `context`: The context in which the variables should be applied.
    /// - `variables`: A map of variable names and their corresponding values.
    ///
    /// # Returns
    /// - `Ok(())` if the variables were successfully applied to the Stylitron AST.
    /// - `Err(GaladrielError)` if there was an error while accessing or modifying the Stylitron AST.
    fn apply_variables_to_stylitron(
        context: String,
        variables: IndexMap<String, IndexMap<String, String>>,
    ) -> GaladrielResult<()> {
        tracing::debug!(context = %context, variables_count = variables.len(), "Applying variables to Stylitron");

        // Access the "variables" entry in the Stylitron AST
        match STYLITRON.get_mut("variables") {
            Some(mut stylitron_data) => {
                match *stylitron_data {
                    // If the entry is found and contains variables, apply the new variables
                    Stylitron::Variables(ref mut variables_definitions) => {
                        let context_map =
                            variables_definitions.entry(context.to_owned()).or_default();

                        // Update the context's variables map
                        *context_map = variables;
                        tracing::info!(context = %context, "Variables applied to Stylitron AST");
                    }
                    _ => {}
                }

                Ok(())
            }
            None => {
                tracing::error!("Failed to access the 'variables' entry in the Stylitron AST");

                Err(GaladrielError::raise_critical_other_error(
                    ErrorKind::AccessDeniedToStylitronAST,
                    "Failed to access the 'variables' entry in the Stylitron AST.",
                    ErrorAction::Restart,
                ))
            }
        }
    }

    /// Processes a single variable asynchronously and generates a unique name for it.
    ///
    /// # Parameters
    /// - `context`: The context in which the variable is being processed.
    /// - `identifier`: The name of the variable.
    /// - `value`: The value associated with the variable.
    ///
    /// # Returns
    /// Returns a `JoinHandle` for the asynchronous task that processes the variable and generates a unique name for it.
    fn process_variable(
        context: String,
        identifier: String,
        value: String,
    ) -> JoinHandle<(String, IndexMap<String, String>)> {
        tokio::task::spawn_blocking(move || {
            tracing::info!(context = %context, identifier = %identifier, "Processing variable");

            // Generate a unique name for the variable based on the context and identifier
            let variable_unique_name =
                generates_variable_or_animation_name(&context, &identifier, true);

            tracing::debug!(variable_unique_name = %variable_unique_name, "Generated unique variable name");

            // Return the identifier along with a map containing the unique variable name and its value
            (identifier, IndexMap::from([(variable_unique_name, value)]))
        })
    }

    /// Extracts and processes futures of variables, filtering out errors and collecting valid results.
    ///
    /// This method takes a vector of `Result` values representing the outcomes of processing
    /// variables, and returns a map of successfully processed variables. If any variable processing
    /// fails, an error is logged, and a corresponding alert is added to the `alerts` vector.
    ///
    /// # Arguments
    ///
    /// * `alerts`: A mutable reference to a vector of `ShellscapeAlerts` where errors will be inserted
    ///   if any of the variable futures fail.
    /// * `variables_futures`: A vector of `Result` values, each representing the result of processing
    ///   a variable. The `Ok` variant contains a tuple of the variable identifier and its associated value,
    ///   while the `Err` variant contains a `JoinError` indicating the failure.
    ///
    /// # Returns
    ///
    /// A `IndexMap` where the keys are variable identifiers (as `String`), and the values are maps of
    /// variable names to their corresponding values (both as `IndexMap<String, String>`).
    fn extract_futures(
        alerts: &mut Vec<ShellscapeAlerts>,
        variables_futures: Vec<Result<(String, IndexMap<String, String>), tokio::task::JoinError>>,
    ) -> IndexMap<String, IndexMap<String, String>> {
        // Iterate over each result of the variable futures
        variables_futures
            .iter()
            .filter_map(|future_result| match future_result {
                // In case of a successful result, log the processing and include it in the output
                Ok(updated_map) => {
                    tracing::info!(identifier = %updated_map.0, "Processed variable successfully");

                    Some(updated_map.to_owned())
                }
                // In case of an error, log the error and create an alert
                Err(err) => {
                    tracing::error!(%err, "Error processing variable");

                    // Create an error and insert it into the alert list
                    let error = GaladrielError::raise_general_other_error(
                        ErrorKind::TaskFailure,
                        &err.to_string(),
                        ErrorAction::Notify,
                    );

                    alerts.insert(
                        0,
                        ShellscapeAlerts::create_galadriel_error(Local::now(), error),
                    );

                    None
                }
            })
            .collect::<IndexMap<_, _>>()
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext, variables::NenyrVariables};

    use crate::crealion::Crealion;

    #[tokio::test]
    async fn test_process_variables_with_variables() {
        // Mock necessary dependencies (e.g., Stylitron, tracing)
        let crealion = Crealion::new(
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        ); // assuming you have a constructor for this

        // Mock or create a fake `NenyrVariables` struct
        let variables = NenyrVariables {
            values: IndexMap::from([("var1".to_string(), "value1".to_string())]),
        };

        // Call the method
        let join_handle = crealion.process_variables("context1".to_string(), Some(variables));

        // Await the join_handle and check results
        let alerts = join_handle.await.unwrap();
        assert_eq!(alerts.len(), 0); // Assuming no errors, adjust as needed

        // Verify logging and any external interactions here
    }

    #[tokio::test]
    async fn test_process_variables_without_variables() {
        let crealion = Crealion::new(
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        // Call the method with None (no variables)
        let join_handle = crealion.process_variables("context1".to_string(), None);

        let alerts = join_handle.await.unwrap();
        assert_eq!(alerts.len(), 0); // Ensure no alerts or errors are generated
    }

    #[tokio::test]
    async fn test_apply_variables_to_stylitron_success() {
        let context = "context1".to_string();
        let variables = IndexMap::from([(
            "var1".to_string(),
            IndexMap::from([("unique-name".to_string(), "value1".to_string())]),
        )]);

        // Test success scenario
        let result = Crealion::apply_variables_to_stylitron(context, variables);
        assert!(result.is_ok()); // Assuming success
    }

    #[tokio::test]
    async fn test_process_variable() {
        let context = "context1".to_string();
        let identifier = "var1".to_string();
        let value = "value1".to_string();

        // Call the method
        let join_handle = Crealion::process_variable(context, identifier, value);
        let result = join_handle.await.unwrap();

        // Check the result
        assert_eq!(result.0, "var1");
        assert!(result.1.get_index(0).is_some());
    }

    #[tokio::test]
    async fn test_extract_futures_success() {
        let mut alerts = Vec::new();
        let futures = vec![Ok((
            "var1".to_string(),
            IndexMap::from([("name".to_string(), "value".to_string())]),
        ))];

        let result = Crealion::extract_futures(&mut alerts, futures);
        assert_eq!(result.len(), 1); // Check that we processed one variable
        assert!(alerts.is_empty()); // No errors, so no alerts
    }
}
