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

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext};
    use tokio::sync::mpsc;

    use crate::{
        asts::STYLITRON, crealion::Crealion, shellscape::alerts::ShellscapeAlerts, types::Stylitron,
    };

    fn mock_imports() -> IndexMap<String, ()> {
        IndexMap::from([
            ("myImportOne".to_string(), ()),
            ("myImportTwo".to_string(), ()),
            ("myImportThree".to_string(), ()),
            ("myImportFour".to_string(), ()),
        ])
    }

    #[tokio::test]
    async fn test_apply_imports_success() {
        let (sender, _) = mpsc::unbounded_channel();

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion.apply_imports_to_stylitron(mock_imports()).await;

        let result = STYLITRON
            .get("imports")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Imports(imports_definitions) => Some(imports_definitions.to_owned()),
                _ => None,
            });

        assert!(result.is_some());

        let imports = result.unwrap();

        assert_eq!(imports, mock_imports());
    }

    #[tokio::test]
    async fn test_apply_imports_to_existing_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let (sender, _) = mpsc::unbounded_channel();

        // Pre-populate the STYLITRON AST with existing data.
        let initial_data = IndexMap::from([("animeName".to_string(), ())]);

        STYLITRON.insert("imports".to_string(), Stylitron::Imports(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion.apply_imports_to_stylitron(mock_imports()).await;

        let result = STYLITRON
            .get("imports")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Imports(imports_definitions) => Some(imports_definitions.clone()),
                _ => None,
            });

        assert!(result.is_some());
        let imports = result.unwrap();

        // Verify that the context was updated correctly.
        assert_eq!(imports, mock_imports());
    }

    #[tokio::test]
    async fn test_apply_imports_to_new_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let (sender, _) = mpsc::unbounded_channel();

        // Ensure no existing context in the STYLITRON AST.
        let initial_data = IndexMap::new();
        STYLITRON.insert("imports".to_string(), Stylitron::Imports(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion.apply_imports_to_stylitron(mock_imports()).await;

        let result = STYLITRON
            .get("imports")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Imports(imports_definitions) => Some(imports_definitions.clone()),
                _ => None,
            });

        assert!(result.is_some());
        let imports = result.unwrap();

        // Verify that the new context was added with correct imports.
        assert_eq!(imports, mock_imports());
    }

    #[tokio::test]
    async fn test_apply_imports_with_empty_imports_data() {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let (sender, _) = mpsc::unbounded_channel();

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let empty_imports: IndexMap<String, ()> = IndexMap::new();
        let _ = crealion
            .apply_imports_to_stylitron(empty_imports.clone())
            .await;

        let result = STYLITRON
            .get("imports")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Imports(imports_definitions) => Some(imports_definitions.clone()),
                _ => None,
            });

        assert!(result.is_some());
        let imports = result.unwrap();

        // Verify that the context was added but remains empty.
        assert_eq!(imports, empty_imports);
    }

    #[tokio::test]
    async fn test_apply_imports_no_imports_section() {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let (sender, mut receiver) = mpsc::unbounded_channel();

        // Simulate an empty STYLITRON AST to trigger an error.
        STYLITRON.remove("imports");

        let crealion = Crealion::new(
            sender.clone(),
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion.apply_imports_to_stylitron(mock_imports()).await;

        // Verify that an error notification was sent.
        if let Some(notification) = receiver.recv().await {
            if let ShellscapeAlerts::GaladrielError {
                start_time: _,
                error,
            } = notification
            {
                assert_eq!(
                    error.get_message(),
                    "Failed to access the imports section in STYLITRON AST".to_string()
                );
            }
        } else {
            panic!("Expected an error notification, but none was received.");
        }
    }
}
