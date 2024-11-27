use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use nenyr::types::breakpoints::NenyrBreakpoints;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
    GaladrielResult,
};

use super::Crealion;

impl Crealion {
    /// Processes the given breakpoints and returns a handle to a task that resolves to a list of alerts.
    ///
    /// # Arguments
    /// * `breakpoints` - Optional NenyrBreakpoints structure containing breakpoint definitions.
    ///
    /// # Returns
    /// * `JoinHandle<Vec<ShellscapeAlerts>>` - A handle to an asynchronous task that produces alerts.
    pub fn process_breakpoints(
        &self,
        breakpoints: Option<NenyrBreakpoints>,
    ) -> JoinHandle<Vec<ShellscapeAlerts>> {
        tokio::task::spawn(async move {
            tracing::trace!("Starting process_breakpoints");

            // Vector to accumulate alerts.
            let mut alerts: Vec<ShellscapeAlerts> = vec![];
            // Map to store processed breakpoints.
            let mut breakpoints_map: IndexMap<String, IndexMap<String, String>> = IndexMap::new();

            // Concurrently process all breakpoint sets.
            let breakpoint_futures = join_all(
                breakpoints
                    .map(|breakpoint_entry| {
                        tracing::trace!("Processing breakpoint entry");

                        // Process both mobile-first and desktop-first breakpoint sets.
                        vec![
                            Self::process_breakpoint_set(true, breakpoint_entry.mobile_first),
                            Self::process_breakpoint_set(false, breakpoint_entry.desktop_first),
                        ]
                    })
                    .unwrap_or_default(), // Handle the case where no breakpoints are provided.
            )
            .await;

            tracing::trace!("Breakpoint futures processing completed");

            // Process the results of each breakpoint future.
            breakpoint_futures
                .iter()
                .for_each(|future_result| match future_result {
                    Ok(Some(updated_map)) => {
                        tracing::debug!("Successfully processed a breakpoint set");
                        breakpoints_map.extend(updated_map.to_vec());
                    }
                    Ok(None) => {}
                    Err(err) => {
                        tracing::error!("Error while processing a breakpoint set: {}", err);

                        let error = GaladrielError::raise_general_other_error(
                            ErrorKind::TaskFailure,
                            &err.to_string(),
                            ErrorAction::Notify,
                        );

                        alerts.insert(
                            0,
                            ShellscapeAlerts::create_galadriel_error(Local::now(), error),
                        );
                    }
                });

            tracing::trace!("Applying breakpoints to Stylitron");

            // Apply processed breakpoints to Stylitron.
            match Self::apply_breakpoints_to_stylitron(breakpoints_map) {
                Ok(()) => {
                    tracing::debug!("Breakpoints successfully applied to Stylitron");
                }
                Err(err) => {
                    tracing::error!("Error applying breakpoints to Stylitron: {}", err);

                    alerts.insert(
                        0,
                        ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                    );
                }
            }

            tracing::trace!("process_breakpoints completed");

            alerts
        })
    }

    /// Applies the given breakpoints to the Stylitron AST.
    ///
    /// # Arguments
    /// * `breakpoints` - A map containing breakpoint definitions.
    ///
    /// # Returns
    /// * `GaladrielResult<()>` - A result indicating success or an error.
    fn apply_breakpoints_to_stylitron(
        breakpoints: IndexMap<String, IndexMap<String, String>>,
    ) -> GaladrielResult<()> {
        tracing::trace!("Entering apply_breakpoints_to_stylitron");

        // Attempt to retrieve and update the "breakpoints" entry in the Stylitron AST.
        match STYLITRON.get_mut("breakpoints") {
            Some(mut breakpoints_definitions) => {
                tracing::debug!("Updating breakpoints in Stylitron");
                *breakpoints_definitions = Stylitron::Breakpoints(breakpoints);

                Ok(())
            }
            None => {
                tracing::error!("Failed to access Stylitron AST");

                Err(GaladrielError::raise_critical_other_error(
                    ErrorKind::AccessDeniedToStylitronAST,
                    "",
                    ErrorAction::Restart,
                ))
            }
        }
    }

    /// Processes a specific breakpoint set for a given schema.
    ///
    /// # Arguments
    /// * `breakpoint_schema` - The schema type, such as "mobile-first" or "desktop-first".
    /// * `breakpoint_definitions` - Optional map of breakpoint definitions.
    ///
    /// # Returns
    /// * `JoinHandle<Option<Vec<(String, IndexMap<String, String>)>>>` - A handle to a task that returns processed breakpoints.
    fn process_breakpoint_set(
        is_mobile_first: bool,
        breakpoint_definitions: Option<IndexMap<String, String>>,
    ) -> JoinHandle<Option<Vec<(String, IndexMap<String, String>)>>> {
        tokio::task::spawn_blocking(move || {
            let breakpoint_schema = if is_mobile_first {
                "mobile-first"
            } else {
                "desktop-first"
            };

            tracing::trace!(
                "Starting process_breakpoint_set for schema: {}",
                breakpoint_schema
            );

            // Handle the breakpoint definitions if provided.
            match breakpoint_definitions {
                Some(breakpoints) => {
                    tracing::debug!(
                        "Processing breakpoint definitions for schema: {}",
                        breakpoint_schema
                    );

                    // Retrieve the schema type for formatting.
                    let schema_type = if is_mobile_first {
                        "min-width".to_string()
                    } else {
                        "max-width".to_string()
                    };

                    // Process and update the map with formatted breakpoints.
                    let mut updated_map: IndexMap<String, String> = IndexMap::new();

                    for (identifier, value) in breakpoints {
                        let updated_value = format!("{}:{}", schema_type, value);

                        updated_map.insert(identifier, updated_value);
                    }

                    tracing::debug!(
                        "Completed processing breakpoint definitions for schema: {}",
                        breakpoint_schema
                    );

                    Some(vec![(breakpoint_schema.to_string(), updated_map)])
                }
                None => {
                    tracing::info!(
                        "No breakpoint definitions provided for schema: {}",
                        breakpoint_schema
                    );

                    None
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, breakpoints::NenyrBreakpoints, central::CentralContext};

    use crate::{asts::STYLITRON, crealion::Crealion, types::Stylitron};

    #[tokio::test]
    async fn test_process_breakpoints_with_valid_input() {
        let crealion = Crealion::new(
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        ); // Assuming a constructor

        let breakpoints = Some(NenyrBreakpoints {
            mobile_first: Some(IndexMap::from([("sm".to_string(), "640px".to_string())])),
            desktop_first: None,
        });

        // Call the method and wait for the result
        let result = crealion.process_breakpoints(breakpoints).await.unwrap();

        // Assert expected results
        assert!(result.is_empty()); // Expect no alerts on success
    }

    #[test]
    fn test_apply_breakpoints_to_stylitron_success() {
        let breakpoints = IndexMap::from([(
            "mobile-first".to_string(),
            IndexMap::from([("sm".to_string(), "640px".to_string())]),
        )]);

        // Mock STYLITRON behavior
        STYLITRON.insert(
            "breakpoints".to_string(),
            Stylitron::Breakpoints(Default::default()),
        );

        // Call the method
        let result = Crealion::apply_breakpoints_to_stylitron(breakpoints);

        // Assert success
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_breakpoints_to_stylitron_is_ok() {
        let breakpoints = IndexMap::from([(
            "mobile-first".to_string(),
            IndexMap::from([("sm".to_string(), "640px".to_string())]),
        )]);

        // Call the method
        let result = Crealion::apply_breakpoints_to_stylitron(breakpoints);

        // Assert error
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_breakpoint_set_with_valid_data() {
        let breakpoints = Some(IndexMap::from([("sm".to_string(), "640px".to_string())]));

        // Call the method
        let result = Crealion::process_breakpoint_set(true, breakpoints)
            .await
            .unwrap();

        // Assert the expected structure
        assert!(result.is_some());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].0, "mobile-first");
        assert_eq!(processed[0].1["sm"], "min-width:640px");
    }

    #[tokio::test]
    async fn test_process_breakpoint_set_with_none() {
        // Call the method with None
        let result = Crealion::process_breakpoint_set(true, None).await.unwrap();

        // Assert that None is returned
        assert!(result.is_none());
    }
}
