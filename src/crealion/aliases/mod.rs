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
                    let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

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

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext};
    use tokio::sync::broadcast;

    use crate::{asts::STYLITRON, crealion::Crealion, events::GaladrielAlerts, types::Stylitron};

    fn mock_aliases() -> IndexMap<String, String> {
        IndexMap::from([
            ("bgd".to_string(), "background-color".to_string()),
            ("dsp".to_string(), "display".to_string()),
            ("br".to_string(), "border".to_string()),
            ("wd".to_string(), "width".to_string()),
        ])
    }

    #[tokio::test]
    async fn test_apply_aliases_success() {
        let (sender, _) = broadcast::channel(0);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_aliases_to_stylitron("myContextName1".to_string(), mock_aliases())
            .await;

        let result = STYLITRON
            .get("aliases")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Aliases(aliases_definitions) => aliases_definitions
                    .get("myContextName1")
                    .and_then(|context_aliases| Some(context_aliases.to_owned())),
                _ => None,
            });

        assert!(result.is_some());

        let aliases = result.unwrap();

        assert_eq!(aliases, mock_aliases());
    }

    #[tokio::test]
    async fn test_apply_aliases_to_existing_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let (sender, _) = broadcast::channel(0);

        // Pre-populate the STYLITRON AST with existing data.
        let initial_data = IndexMap::from([(
            "myContextName3".to_string(),
            IndexMap::from([("animeName".to_string(), "animation-name".to_string())]),
        )]);

        STYLITRON.insert("aliases".to_string(), Stylitron::Aliases(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_aliases_to_stylitron("myContextName3".to_string(), mock_aliases())
            .await;

        let result = STYLITRON
            .get("aliases")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Aliases(aliases_definitions) => {
                    aliases_definitions.get("myContextName3").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());
        let aliases = result.unwrap();

        // Verify that the context was updated correctly.
        assert_eq!(aliases, mock_aliases());
    }

    #[tokio::test]
    async fn test_apply_aliases_to_new_context() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let (sender, _) = broadcast::channel(0);

        // Ensure no existing context in the STYLITRON AST.
        let initial_data = IndexMap::new();
        STYLITRON.insert("aliases".to_string(), Stylitron::Aliases(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_aliases_to_stylitron("newContextName".to_string(), mock_aliases())
            .await;

        let result = STYLITRON
            .get("aliases")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Aliases(aliases_definitions) => {
                    aliases_definitions.get("newContextName").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());
        let aliases = result.unwrap();

        // Verify that the new context was added with correct aliases.
        assert_eq!(aliases, mock_aliases());
    }

    #[tokio::test]
    async fn test_apply_aliases_with_empty_aliases_data() {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let (sender, _) = broadcast::channel(0);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let empty_aliases: IndexMap<String, String> = IndexMap::new();
        let _ = crealion
            .apply_aliases_to_stylitron("emptyAliasesContext".to_string(), empty_aliases.clone())
            .await;

        let result = STYLITRON
            .get("aliases")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Aliases(aliases_definitions) => {
                    aliases_definitions.get("emptyAliasesContext").cloned()
                }
                _ => None,
            });

        assert!(result.is_some());
        let aliases = result.unwrap();

        // Verify that the context was added but remains empty.
        assert_eq!(aliases, empty_aliases);
    }

    #[tokio::test]
    async fn test_apply_aliases_no_aliases_section() {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let (sender, mut receiver) = broadcast::channel(0);

        // Simulate an empty STYLITRON AST to trigger an error.
        STYLITRON.remove("aliases");

        let crealion = Crealion::new(
            sender.clone(),
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .apply_aliases_to_stylitron("noAliasSection".to_string(), mock_aliases())
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
                    "Failed to access the aliases section in STYLITRON AST".to_string()
                );
            }
        } else {
            panic!("Expected an error notification, but none was received.");
        }
    }
}
