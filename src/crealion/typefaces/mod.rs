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
    /// Updates the `typefaces` section of the STYLITRON AST with the provided typefaces data.
    ///
    /// # Arguments
    /// - `typefaces_data`: An `IndexMap` where the keys represent typeface identifiers,
    ///   and the values represent the corresponding typeface definitions.
    ///
    /// # Returns
    /// - A `JoinHandle` representing the spawned blocking task.
    pub fn apply_typefaces_to_stylitron(
        &self,
        typefaces_data: IndexMap<String, String>,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();

        // Spawn a blocking task to safely update the STYLITRON AST.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting the process to apply typefaces to the STYLITRON AST.");

            // Attempt to access the `typefaces` section within the STYLITRON AST.
            let mut stylitron_data = match STYLITRON.get_mut("typefaces") {
                Some(data) => {
                    tracing::debug!(
                        "Successfully accessed the `typefaces` section in STYLITRON AST."
                    );
                    data
                }
                None => {
                    tracing::error!("Failed to access the `typefaces` section in STYLITRON AST.");

                    // If the `typefaces` section is not found, raise a critical error.
                    let error = GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToStylitronAST,
                        "Failed to access the typefaces section in STYLITRON AST",
                        ErrorAction::Restart,
                    );

                    tracing::error!("Critical error encountered: {:?}", error);

                    // Generate an error notification to be sent to the appropriate handler.
                    let notification =
                        ShellscapeAlerts::create_galadriel_error(Local::now(), error);

                    // Attempt to send the error notification. Log any failures.
                    if let Err(err) = sender.send(notification) {
                        tracing::error!("Failed to send notification: {}", err);
                    }

                    return;
                }
            };

            // Check if the `stylitron_data` matches the expected `Typefaces` variant.
            match *stylitron_data {
                // If it matches `Stylitron::Typefaces`, update its content with the provided data.
                Stylitron::Typefaces(ref mut typefaces_definitions) => {
                    tracing::info!(
                        "Found `Typefaces` section in STYLITRON AST. Applying updates..."
                    );

                    // Overwrite with new typefaces data.
                    *typefaces_definitions = typefaces_data;

                    tracing::info!("Successfully updated `typefaces` section with new data.");
                }
                _ => {}
            }

            tracing::info!("Completed the process of applying typefaces to the STYLITRON AST.");
        })
    }
}
