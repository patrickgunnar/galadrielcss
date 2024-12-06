use crate::{
    asts::INTAKER,
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

use super::Crealion;

impl Crealion {
    /// Validates the uniqueness of a context name across different file paths.
    ///
    /// This function checks if the provided `context_name` is already in use for another file path.
    /// If a conflict is found, it returns an error with details about the conflicting context and file paths.
    /// If no conflict is found, the context name is inserted into a global registry (`INTAKER`).
    ///
    /// # Arguments
    /// * `context_name` - The name of the context to validate.
    /// * `file_path` - The file path where the context name is being used.
    ///
    /// # Returns
    /// * `Ok(())` if the context name is unique and successfully inserted.
    /// * An error of type `GaladrielError` if a conflict is found.
    ///
    /// # Errors
    /// This function raises a `GaladrielError` if the context name is found to be in use in another file path.
    pub fn validates_context_name(
        &self,
        context_name: String,
        file_path: String,
    ) -> GaladrielResult<()> {
        tracing::info!(
            "Validating context name '{}' for file path '{}'",
            context_name,
            file_path
        );

        // Attempt to find a conflicting context name already associated with a different file path
        let conflicting_context_entry = INTAKER
            .iter()
            .find(|entry| entry.key() != &file_path && entry.value() == &context_name);

        match conflicting_context_entry {
            // If a conflict is found, return an error with details about the conflict
            Some(conflict_entry) => {
                // Extract the conflicting file path and context name
                let conflicting_path = conflict_entry.key();
                let conflicting_name = conflict_entry.value();

                tracing::error!(
                    "Context name conflict detected! The context name '{}' in file '{}' conflicts with '{}' in '{}'.",
                    conflicting_name, conflicting_path, context_name, file_path
                );

                // Return an error, indicating a common context name conflict with a description
                return Err(GaladrielError::raise_general_other_error(
                    ErrorKind::ContextNameConflict,
                    &format!(
                        "The context name `{}` cannot be used in `{}` because it is already in use in `{}`. Each context name must be unique across all paths.",
                        conflicting_name, file_path, conflicting_path
                    ),
                    ErrorAction::Notify,
                ));
            }
            None => {
                // If no conflict is found, insert the context name and file path into the registry
                INTAKER.insert(file_path.to_owned(), context_name.to_owned());

                tracing::info!(
                    "Context name '{}' successfully validated and added for '{}'",
                    context_name,
                    file_path
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use nenyr::types::{ast::NenyrAst, central::CentralContext};
    use tokio::sync::broadcast;

    use crate::{asts::INTAKER, crealion::Crealion};

    #[test]
    fn context_name_is_valid() {
        let (sender, _) = broadcast::channel(0);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let result = crealion.validates_context_name(
            "noExistingContextName".to_string(),
            "path/to/context.nyr".to_string(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn context_name_is_not_valid() {
        INTAKER.insert(
            "path/to/context_1.nyr".to_string(),
            "newContextName".to_string(),
        );

        let (sender, _) = broadcast::channel(0);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let result = crealion.validates_context_name(
            "newContextName".to_string(),
            "path/to/context_2.nyr".to_string(),
        );

        assert!(result.is_err());
    }
}
