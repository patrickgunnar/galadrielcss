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

#[derive(Clone, PartialEq, Debug)]
pub enum BreakpointType {
    MobileFirst,
    DesktopFirst,
}

impl Crealion {
    /// Processes and applies breakpoint definitions (mobile-first and desktop-first) to the
    /// `breakpoints` section of the STYLITRON AST.
    ///
    /// # Arguments
    /// - `mobile_data`: An optional `IndexMap` containing mobile-first breakpoint definitions.
    /// - `desktop_data`: An optional `IndexMap` containing desktop-first breakpoint definitions.
    ///
    /// # Returns
    /// - A `JoinHandle` representing the spawned task that performs the processing.
    pub fn process_breakpoints(
        &self,
        mobile_data: Option<IndexMap<String, String>>,
        desktop_data: Option<IndexMap<String, String>>,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();

        // Spawn a blocking task to process the variables.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting the process to apply breakpoints to the STYLITRON AST.");

            // Attempt to retrieve the "variables" section from the STYLITRON AST.
            let mut stylitron_data = match STYLITRON.get_mut("breakpoints") {
                Some(data) => {
                    tracing::debug!(
                        "Successfully accessed the `breakpoints` section in STYLITRON AST."
                    );
                    data
                }
                None => {
                    tracing::error!("Failed to access the `breakpoints` section in STYLITRON AST.");

                    // If the "breakpoints" section is not accessible, create a critical error.
                    let error = GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToStylitronAST,
                        "Failed to access the breakpoints section in STYLITRON AST",
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

            // Process the provided mobile-first and desktop-first breakpoint data.
            let mobile_first_data =
                Self::process_breakpoint(mobile_data, BreakpointType::MobileFirst);
            let desktop_first_data =
                Self::process_breakpoint(desktop_data, BreakpointType::DesktopFirst);

            // Match the `stylitron_data` to ensure it's of the expected type.
            match *stylitron_data {
                // If it's a `Stylitron::Breakpoints` variant, insert or update the context variables.
                Stylitron::Breakpoints(ref mut breakpoints_definitions) => {
                    tracing::info!(
                        "Found `Breakpoints` section in STYLITRON AST. Applying updates..."
                    );

                    // Transform the provided breakpoints data into the expected format for STYLITRON.
                    let breakpoints = IndexMap::from([
                        ("mobile-first".to_string(), mobile_first_data),
                        ("desktop-first".to_string(), desktop_first_data),
                    ]);

                    // Overwrite the existing breakpoints definitions with the new data.
                    *breakpoints_definitions = breakpoints;

                    tracing::info!(
                        "Successfully updated the `breakpoints` section in STYLITRON AST."
                    );
                }
                _ => {}
            }

            tracing::info!("Completed the process of applying breakpoints to the STYLITRON AST.");
        })
    }

    /// Processes a set of breakpoints and formats them according to the specified schema type.
    ///
    /// # Arguments
    /// - `breakpoint_data`: An optional `IndexMap` containing the breakpoint definitions.
    /// - `breakpoint_type`: The type of breakpoint schema (`MobileFirst` or `DesktopFirst`).
    ///
    /// # Returns
    /// - An `IndexMap` with formatted breakpoint definitions.
    fn process_breakpoint(
        breakpoint_data: Option<IndexMap<String, String>>,
        breakpoint_type: BreakpointType,
    ) -> IndexMap<String, String> {
        // Determine the schema type (`min-width` or `max-width`) based on the breakpoint type.
        let schema_type = match breakpoint_type {
            BreakpointType::MobileFirst => "min-width",
            BreakpointType::DesktopFirst => "max-width",
        };

        // Process the breakpoint data, formatting each entry with the schema type.
        breakpoint_data
            .unwrap_or_default()
            .into_iter()
            .map(|(identifier, value)| {
                // Format each breakpoint definition as `<schema_type>:<value>`.
                (identifier, format!("{}:{}", schema_type, value))
            })
            .collect()
    }
}
