use chrono::Local;
use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
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
                    let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

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

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext};
    use tokio::sync::broadcast;

    use crate::{asts::STYLITRON, crealion::Crealion, events::GaladrielAlerts, types::Stylitron};

    fn mock_typefaces() -> IndexMap<String, String> {
        IndexMap::from([
            (
                "myTypefaceOne".to_string(),
                "../typefaces/showa-source-curry.regular-webfont.eot".to_string(),
            ),
            (
                "myTypefaceTwo".to_string(),
                "../typefaces/showa-source-curry.regular-webfont.svg".to_string(),
            ),
            (
                "myTypefaceThree".to_string(),
                "../typefaces/showa-source-curry.regular-webfont.ttf".to_string(),
            ),
            (
                "myTypefaceFour".to_string(),
                "../typefaces/showa-source-curry.regular-webfont.woff".to_string(),
            ),
        ])
    }

    #[tokio::test]
    async fn test_apply_typefaces_success() {
        let (sender, _) = broadcast::channel(0);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_typefaces_to_stylitron(mock_typefaces())
            .await;

        let result = STYLITRON
            .get("typefaces")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Typefaces(typefaces_definitions) => {
                    Some(typefaces_definitions.to_owned())
                }
                _ => None,
            });

        assert!(result.is_some());

        let typefaces = result.unwrap();

        assert_eq!(typefaces, mock_typefaces());
    }

    #[tokio::test]
    async fn test_apply_typefaces_to_existing_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let (sender, _) = broadcast::channel(0);

        // Pre-populate the STYLITRON AST with existing data.
        let initial_data =
            IndexMap::from([("animeName".to_string(), "animation-name".to_string())]);

        STYLITRON.insert("typefaces".to_string(), Stylitron::Typefaces(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_typefaces_to_stylitron(mock_typefaces())
            .await;

        let result = STYLITRON
            .get("typefaces")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Typefaces(typefaces_definitions) => Some(typefaces_definitions.clone()),
                _ => None,
            });

        assert!(result.is_some());
        let typefaces = result.unwrap();

        // Verify that the context was updated correctly.
        assert_eq!(typefaces, mock_typefaces());
    }

    #[tokio::test]
    async fn test_apply_typefaces_to_new_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let (sender, _) = broadcast::channel(0);

        // Ensure no existing context in the STYLITRON AST.
        let initial_data = IndexMap::new();
        STYLITRON.insert("typefaces".to_string(), Stylitron::Typefaces(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_typefaces_to_stylitron(mock_typefaces())
            .await;

        let result = STYLITRON
            .get("typefaces")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Typefaces(typefaces_definitions) => Some(typefaces_definitions.clone()),
                _ => None,
            });

        assert!(result.is_some());
        let typefaces = result.unwrap();

        // Verify that the new context was added with correct typefaces.
        assert_eq!(typefaces, mock_typefaces());
    }

    #[tokio::test]
    async fn test_apply_typefaces_with_empty_typefaces_data() {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let (sender, _) = broadcast::channel(0);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let empty_typefaces: IndexMap<String, String> = IndexMap::new();
        let _ = crealion
            .apply_typefaces_to_stylitron(empty_typefaces.clone())
            .await;

        let result = STYLITRON
            .get("typefaces")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Typefaces(typefaces_definitions) => Some(typefaces_definitions.clone()),
                _ => None,
            });

        println!("{:?}", result);
        assert!(result.is_some());
        let typefaces = result.unwrap();

        // Verify that the context was added but remains empty.
        assert_eq!(typefaces, empty_typefaces);
    }

    #[tokio::test]
    async fn test_apply_typefaces_no_typefaces_section() {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let (sender, mut receiver) = broadcast::channel(0);

        // Simulate an empty STYLITRON AST to trigger an error.
        STYLITRON.remove("typefaces");

        let crealion = Crealion::new(
            sender.clone(),
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_typefaces_to_stylitron(mock_typefaces())
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
                    "Failed to access the typefaces section in STYLITRON AST".to_string()
                );
            }
        } else {
            panic!("Expected an error notification, but none was received.");
        }
    }
}
