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
    /// Updates the `aliases` section of the STYLITRON AST with the provided aliases data
    /// for a specific context.
    ///
    /// # Arguments
    /// - `context_name`: The name of the context for which the aliases are being applied.
    /// - `aliases_data`: An `IndexMap` where the keys represent alias identifiers, and the
    ///   values represent their corresponding property definitions.
    ///
    /// # Returns
    /// - A `JoinHandle` representing the spawned blocking task.
    pub fn apply_aliases_to_stylitron(
        &self,
        context_name: String,
        aliases_data: IndexMap<String, String>,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();

        // Spawn a blocking task to safely update the STYLITRON AST.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting to apply aliases for context: {}", context_name);

            // Attempt to retrieve the `aliases` section of the STYLITRON AST.
            let mut stylitron_data = match STYLITRON.get_mut("aliases") {
                Some(data) => {
                    tracing::debug!("Successfully accessed the aliases section in STYLITRON AST.");
                    data
                }
                None => {
                    tracing::error!(
                        "Failed to access the aliases section in STYLITRON AST for context: {}",
                        context_name
                    );

                    // If the `aliases` section is not found, raise a critical error.
                    let error = GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToStylitronAST,
                        "Failed to access the aliases section in STYLITRON AST",
                        ErrorAction::Restart,
                    );

                    tracing::error!("Critical error raised: {:?}", error);

                    // Create a notification to report the error.
                    let notification =
                        ShellscapeAlerts::create_galadriel_error(Local::now(), error);

                    // Attempt to send the notification and log any failures.
                    if let Err(err) = sender.send(notification) {
                        tracing::error!("Failed to send notification: {}", err);
                    }

                    return;
                }
            };

            // Check if the retrieved data matches the `Aliases` variant.
            match *stylitron_data {
                // If it matches `Stylitron::Aliases`, update its content with the provided data.
                Stylitron::Aliases(ref mut aliases_definitions) => {
                    tracing::info!(
                        "Found `Aliases` section in STYLITRON AST for context: {}",
                        context_name
                    );

                    // Find or create the aliases for the specified context.
                    let context_aliases = aliases_definitions
                        .entry(context_name.to_owned())
                        .or_default();

                    // Update the aliases for the context with the provided data.
                    *context_aliases = aliases_data;

                    tracing::debug!(
                        "Aliases for context '{}' updated successfully. New aliases: {:?}",
                        context_name,
                        context_aliases
                    );
                }
                _ => {}
            }

            tracing::info!("Completed alias application for context: {}", context_name);
        })
    }
}
