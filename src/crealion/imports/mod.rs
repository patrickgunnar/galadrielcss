use chrono::Local;
use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
};

use super::Crealion;

impl Crealion {
    /// Applies the provided imports data to the `imports` section of the STYLITRON AST.
    ///
    /// # Arguments
    /// - `imports_data`: An `IndexMap` where the keys are identifiers for the imports, and the values are empty tuples.
    ///
    /// # Returns
    /// - A `JoinHandle` representing the spawned task. The task updates the STYLITRON AST in a separate thread.
    pub fn apply_imports_to_stylitron(&self, imports_data: IndexMap<String, ()>) -> JoinHandle<()> {
        let sender = self.sender.clone();

        // Spawn a blocking task to apply the imports to the STYLITRON AST.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting the process to apply imports to the STYLITRON AST.");

            // Attempt to access the `imports` section of the STYLITRON AST.
            let mut stylitron_data = match STYLITRON.get_mut("imports") {
                Some(data) => {
                    tracing::debug!(
                        "Successfully accessed the `imports` section in STYLITRON AST."
                    );
                    data
                }
                None => {
                    tracing::error!("Failed to access the `imports` section in STYLITRON AST.");

                    // If the `imports` section is not accessible, raise a critical error.
                    let error = GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToStylitronAST,
                        "Failed to access the imports section in STYLITRON AST",
                        ErrorAction::Restart,
                    );

                    tracing::error!("Critical error encountered: {:?}", error);

                    // Generate an error notification to inform the system about the issue.
                    let notification =
                        ShellscapeAlerts::create_galadriel_error(Local::now(), error);

                    // Attempt to send the notification using the sender.
                    if let Err(err) = sender.send(notification) {
                        tracing::error!("Failed to send notification: {}", err);
                    }

                    return;
                }
            };

            // Match the retrieved `stylitron_data` to ensure it is the expected `Imports` variant.
            match *stylitron_data {
                // If it is `Stylitron::Imports`, update it with the provided imports data.
                Stylitron::Imports(ref mut imports_definitions) => {
                    tracing::info!(
                        "Found `Imports` section in STYLITRON AST. Proceeding to apply updates."
                    );

                    // Overwrite the existing imports definitions with the new data.
                    *imports_definitions = imports_data;

                    tracing::info!("Successfully updated the `imports` section in STYLITRON AST.");
                }
                _ => {}
            }

            tracing::info!("Completed the process of applying imports to the STYLITRON AST.");
        })
    }
}
