use chrono::Local;
use indexmap::IndexMap;
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    types::Stylitron,
    utils::generates_node_styles::generates_node_styles,
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
                    // Handle error if the "breakpoints" section is not found in the STYLITRON AST.
                    Self::handle_access_error("breakpoints", sender);

                    return;
                }
            };

            // Process the provided mobile-first and desktop-first breakpoint data.
            let mobile_definitions =
                Self::process_breakpoint(mobile_data, BreakpointType::MobileFirst);
            let desktop_definitions =
                Self::process_breakpoint(desktop_data, BreakpointType::DesktopFirst);

            // Apply the processed breakpoint definitions to the responsive node in STYLITRON.
            if let Err(_) = Self::apply_definitions_to_responsive_node(
                mobile_definitions.to_owned(),
                desktop_definitions.to_owned(),
                sender.clone(),
            ) {
                return;
            }

            // Match the `stylitron_data` to ensure it's of the expected type.
            match *stylitron_data {
                // If it's a `Stylitron::Breakpoints` variant, insert or update the context variables.
                Stylitron::Breakpoints(ref mut breakpoints_definitions) => {
                    tracing::info!(
                        "Found `Breakpoints` section in STYLITRON AST. Applying updates..."
                    );

                    // Transform the provided breakpoints data into the expected format for STYLITRON.
                    let breakpoints = IndexMap::from([
                        ("mobile-first".to_string(), mobile_definitions),
                        ("desktop-first".to_string(), desktop_definitions),
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

    /// Applies the provided mobile and desktop breakpoints definitions to the responsive node in the STYLITRON AST.
    ///
    /// This function accesses the `responsive` section in the STYLITRON Abstract Syntax Tree (AST),
    /// and processes the given `mobile_definitions` and `desktop_definitions` to integrate them
    /// into the existing responsive styles.
    ///
    /// # Arguments
    /// - `mobile_definitions` (`IndexMap<String, String>`): A collection of mobile-first breakpoint definitions.
    /// - `desktop_definitions` (`IndexMap<String, String>`): A collection of desktop-first breakpoints definitions.
    /// - `sender` (`broadcast::Sender<GaladrielAlerts>`): The sender used to send alerts, such as error notifications.
    ///
    /// # Returns
    /// - `Ok(())`: If the definitions are successfully applied.
    /// - `Err(())`: If there was an error accessing the `responsive` section in the STYLITRON AST.
    fn apply_definitions_to_responsive_node(
        mobile_definitions: IndexMap<String, String>,
        desktop_definitions: IndexMap<String, String>,
        sender: broadcast::Sender<GaladrielAlerts>,
    ) -> Result<(), ()> {
        // Attempt to access the "responsive" section in the STYLITRON AST.
        let mut stylitron_data = match STYLITRON.get_mut("responsive") {
            Some(data) => {
                tracing::debug!("Successfully accessed the `responsive` section in STYLITRON AST.");

                data
            }
            None => {
                // Handle error if the "responsive" section is not found in the STYLITRON AST.
                Self::handle_access_error("responsive", sender);

                return Err(());
            }
        };

        // Match on the data to ensure it is of the correct type.
        match *stylitron_data {
            // If the data is of type `Stylitron::ResponsiveStyles`, process the definitions.
            Stylitron::ResponsiveStyles(ref mut responsive_definitions) => {
                // Process mobile and desktop first definitions.
                Self::process_definitions_creation(mobile_definitions, responsive_definitions);
                Self::process_definitions_creation(desktop_definitions, responsive_definitions);

                return Ok(());
            }
            _ => return Err(()),
        }
    }

    /// Processes the creation of breakpoints and adds them to the `responsive_definitions` (STYLITRON responsive node).
    ///
    /// This function iterates over the provided `definitions` (breakpoints: "<schema type>:<breakpoint value>") and
    /// ensures each definition is inserted into the `responsive_definitions` (responsive node) under the appropriate entry.
    ///
    /// # Arguments
    /// - `definitions` (`IndexMap<String, String>`): A collection of breakpoints to be processed.
    /// - `responsive_definitions` (`&mut IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>>`):
    ///   A mutable reference to the map where the processed definitions will be inserted (responsive node).
    fn process_definitions_creation(
        definitions: IndexMap<String, String>,
        responsive_definitions: &mut IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>>,
        >,
    ) {
        // Iterate over each breakpoint and insert it into the responsive definitions.
        definitions.into_iter().for_each(|(_, definition)| {
            // Insert the breakpoint into the responsive definitions map.
            responsive_definitions
                .entry(definition)
                .or_insert_with(generates_node_styles);
        });
    }

    /// Handles the error when access to a section in the STYLITRON AST fails.
    ///
    /// This function logs the error and generates a critical error notification,
    /// which is then sent through the provided `sender`.
    ///
    /// # Arguments
    /// - `node_name` (`&str`): The name of the section in the STYLITRON AST that could not be accessed.
    /// - `sender` (`broadcast::Sender<GaladrielAlerts>`): The sender used to send error alerts.
    fn handle_access_error(node_name: &str, sender: broadcast::Sender<GaladrielAlerts>) {
        tracing::error!("Failed to access the `{node_name}` section in STYLITRON AST.");

        // If the "node_name" section is not accessible, create a critical error.
        let error = GaladrielError::raise_critical_other_error(
            ErrorKind::AccessDeniedToStylitronAST,
            &format!("Failed to access the {node_name} section in STYLITRON AST"),
            ErrorAction::Restart,
        );

        tracing::error!("Critical error encountered: {:?}", error);

        // Generate an error notification and attempt to send it via the sender.
        let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

        if let Err(err) = sender.send(notification) {
            tracing::error!("Failed to send notification: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext};
    use tokio::sync::broadcast;

    use crate::{asts::STYLITRON, crealion::Crealion, events::GaladrielAlerts, types::Stylitron};

    use super::BreakpointType;

    fn mock_breakpoints() -> (IndexMap<String, String>, IndexMap<String, String>) {
        let mobile_data = IndexMap::from([
            ("sm".to_string(), "320px".to_string()),
            ("md".to_string(), "640px".to_string()),
            ("xl".to_string(), "1280px".to_string()),
            ("xx".to_string(), "2560px".to_string()),
        ]);

        let desktop_data = IndexMap::from([
            ("sm".to_string(), "320px".to_string()),
            ("md".to_string(), "640px".to_string()),
            ("xl".to_string(), "1280px".to_string()),
            ("xx".to_string(), "2560px".to_string()),
        ]);

        (mobile_data, desktop_data)
    }

    fn format_breakpoints(
        breakpoints: IndexMap<String, String>,
        breakpoint_type: BreakpointType,
    ) -> IndexMap<String, String> {
        let schema_type = match breakpoint_type {
            BreakpointType::MobileFirst => "min-width",
            BreakpointType::DesktopFirst => "max-width",
        };

        breakpoints
            .into_iter()
            .map(|(identifier, value)| (identifier, format!("{}:{}", schema_type, value)))
            .collect()
    }

    #[tokio::test]
    async fn test_apply_breakpoints_success() {
        let (sender, _) = broadcast::channel(10);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let (mobile_data, desktop_data) = mock_breakpoints();
        let _ = crealion
            .process_breakpoints(Some(mobile_data.clone()), Some(desktop_data.clone()))
            .await;

        let result =
            STYLITRON
                .get("breakpoints")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Breakpoints(breakpoints_definitions) => {
                        Some(breakpoints_definitions.to_owned())
                    }
                    _ => None,
                });

        assert!(result.is_some());

        let breakpoints = result.unwrap();
        let expected_result = IndexMap::from([
            (
                "mobile-first".to_string(),
                format_breakpoints(mobile_data, BreakpointType::MobileFirst),
            ),
            (
                "desktop-first".to_string(),
                format_breakpoints(desktop_data, BreakpointType::DesktopFirst),
            ),
        ]);

        assert_eq!(breakpoints, expected_result);
    }

    #[tokio::test]
    async fn test_apply_breakpoints_to_existing_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let (sender, _) = broadcast::channel(10);

        // Pre-populate the STYLITRON AST with existing data.
        let initial_data = IndexMap::from([(
            "mobile-first".to_string(),
            IndexMap::from([("myFakeBreakpoint".to_string(), "1024px".to_string())]),
        )]);

        STYLITRON.insert(
            "breakpoints".to_string(),
            Stylitron::Breakpoints(initial_data),
        );

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let (mobile_data, desktop_data) = mock_breakpoints();
        let _ = crealion
            .process_breakpoints(Some(mobile_data.clone()), Some(desktop_data.clone()))
            .await;

        let result =
            STYLITRON
                .get("breakpoints")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Breakpoints(breakpoints_definitions) => {
                        Some(breakpoints_definitions.clone())
                    }
                    _ => None,
                });

        assert!(result.is_some());
        let breakpoints = result.unwrap();
        let expected_result = IndexMap::from([
            (
                "mobile-first".to_string(),
                format_breakpoints(mobile_data, BreakpointType::MobileFirst),
            ),
            (
                "desktop-first".to_string(),
                format_breakpoints(desktop_data, BreakpointType::DesktopFirst),
            ),
        ]);

        // Verify that the context was updated correctly.
        assert_eq!(breakpoints, expected_result);
    }

    #[tokio::test]
    async fn test_apply_breakpoints_to_new_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let (sender, _) = broadcast::channel(10);

        // Ensure no existing context in the STYLITRON AST.
        let initial_data = IndexMap::new();
        STYLITRON.insert(
            "breakpoints".to_string(),
            Stylitron::Breakpoints(initial_data),
        );

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let (mobile_data, desktop_data) = mock_breakpoints();
        let _ = crealion
            .process_breakpoints(Some(mobile_data.clone()), Some(desktop_data.clone()))
            .await;

        let result =
            STYLITRON
                .get("breakpoints")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Breakpoints(breakpoints_definitions) => {
                        Some(breakpoints_definitions.clone())
                    }
                    _ => None,
                });

        assert!(result.is_some());
        let breakpoints = result.unwrap();
        let expected_result = IndexMap::from([
            (
                "mobile-first".to_string(),
                format_breakpoints(mobile_data, BreakpointType::MobileFirst),
            ),
            (
                "desktop-first".to_string(),
                format_breakpoints(desktop_data, BreakpointType::DesktopFirst),
            ),
        ]);

        // Verify that the new context was added with correct breakpoints.
        assert_eq!(breakpoints, expected_result);
    }

    #[tokio::test]
    async fn test_apply_breakpoints_with_empty_breakpoints_data() {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let (sender, _) = broadcast::channel(10);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let empty_breakpoints: IndexMap<String, String> = IndexMap::new();
        let _ = crealion
            .process_breakpoints(
                Some(empty_breakpoints.clone()),
                Some(empty_breakpoints.clone()),
            )
            .await;

        let result =
            STYLITRON
                .get("breakpoints")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Breakpoints(breakpoints_definitions) => {
                        Some(breakpoints_definitions.clone())
                    }
                    _ => None,
                });

        assert!(result.is_some());
        let breakpoints = result.unwrap();
        let empty_breakpoints: IndexMap<String, IndexMap<String, String>> = IndexMap::from([
            ("mobile-first".to_string(), IndexMap::new()),
            ("desktop-first".to_string(), IndexMap::new()),
        ]);

        // Verify that the context was added but remains empty.
        assert_eq!(breakpoints, empty_breakpoints);
    }

    #[tokio::test]
    async fn test_apply_breakpoints_no_breakpoints_section() {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let (sender, mut receiver) = broadcast::channel(10);

        // Simulate an empty STYLITRON AST to trigger an error.
        STYLITRON.remove("breakpoints");

        let crealion = Crealion::new(
            sender.clone(),
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let (mobile_data, desktop_data) = mock_breakpoints();
        let _ = crealion
            .process_breakpoints(Some(mobile_data), Some(desktop_data))
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
                    "Failed to access the breakpoints section in STYLITRON AST".to_string()
                );
            }
        } else {
            panic!("Expected an error notification, but none was received.");
        }
    }
}
