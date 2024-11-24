use crate::{asts::STYLITRON, types::Stylitron};

/// A processor for handling nickname alias resolution.
///
/// The `NicknameProcessor` is responsible for resolving nicknames to their corresponding
/// aliases by looking them up in the `STYLITRON` data structure. It searches through the
/// specified inherited contexts to find a match.
#[derive(Clone, PartialEq, Debug)]
pub struct NicknameProcessor {
    /// A list of contexts inherited by this processor, used to search for alias definitions.
    inherited_contexts: Vec<String>,
}

impl NicknameProcessor {
    /// Creates a new `NicknameProcessor` instance.
    ///
    /// # Arguments
    /// - `inherited_contexts`: A vector of context names that the processor will use to resolve aliases.
    ///
    /// # Returns
    /// - A new instance of `NicknameProcessor`.
    pub fn new(inherited_contexts: Vec<String>) -> Self {
        Self { inherited_contexts }
    }

    /// Processes an alias and attempts to resolve it to its property.
    ///
    /// This function looks up the given alias/nickname in the `STYLITRON` data under the "aliases" node.
    /// It iterates over the inherited contexts and retrieves the corresponding property if found.
    ///
    /// # Arguments
    /// - `nickname`: The alias to resolve.
    ///
    /// # Returns
    /// - `Some(String)`: The resolved property if found.
    /// - `None`: If the alias could not be resolved in any context.
    pub fn process(&self, nickname: &str) -> Option<String> {
        // Access the "aliases" node in the STYLITRON data structure.
        STYLITRON
            .get("aliases")
            .and_then(|stylitron_data| match &*stylitron_data {
                // If the data is of type `Aliases`, proceed with processing.
                Stylitron::Aliases(aliases_definitions) => {
                    // Iterate over the inherited contexts to find a matching alias.
                    self.inherited_contexts.iter().find_map(|context_name| {
                        // Retrieve the context-specific alias definitions.
                        aliases_definitions
                            .get(context_name)
                            .and_then(|context_aliases| {
                                // Attempt to resolve the property for the provided alias.
                                context_aliases
                                    .get(nickname)
                                    .and_then(|alias_entry| Some(alias_entry.to_owned()))
                            })
                    })
                }
                _ => None, // If the node is not of type `Aliases`, return `None`.
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::crealion::{
        mocks::test_helpers::mock_aliases_node, processors::nickname::NicknameProcessor,
    };

    #[test]
    fn test_resolve_alias_from_first_context() {
        // Arrange: mock the aliases node
        mock_aliases_node();

        // Create the processor and try to resolve an alias from the first context
        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = NicknameProcessor::new(contexts);

        // Act: Process a valid alias
        let result = processor.process("bgdColor");

        // Assert: Check that the alias was resolved correctly
        assert_eq!(result, Some("background-color".to_string()));
    }

    #[test]
    fn test_resolve_alias_from_second_context() {
        // Arrange: mock the aliases node
        mock_aliases_node();

        // Create the processor and try to resolve an alias from the second context
        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = NicknameProcessor::new(contexts);

        // Act: Process a valid alias
        let result = processor.process("br");

        // Assert: Check that the alias was resolved correctly
        assert_eq!(result, Some("border-radius".to_string()));
    }

    #[test]
    fn test_resolve_alias_from_multiple_contexts() {
        // Arrange: mock the aliases node
        mock_aliases_node();

        // Create the processor and try to resolve an alias from multiple contexts
        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = NicknameProcessor::new(contexts);

        // Act: Process a valid alias
        let result = processor.process("pdg");

        // Assert: Check that the alias was resolved correctly
        assert_eq!(result, Some("padding".to_string()));
    }

    #[test]
    fn test_alias_not_found() {
        // Arrange: mock the aliases node
        mock_aliases_node();

        // Create the processor and try to resolve an alias that doesn't exist
        let contexts = vec!["myGlacialContext".to_string(), "galaxyContext".to_string()];
        let processor = NicknameProcessor::new(contexts);

        // Act: Process an invalid alias
        let result = processor.process("nonExistentAlias");

        // Assert: Check that no alias was found
        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_inherited_contexts() {
        // Arrange: mock the aliases node
        mock_aliases_node();

        // Create the processor with no inherited contexts
        let processor = NicknameProcessor::new(vec![]);

        // Act: Process an alias
        let result = processor.process("bgdColor");

        // Assert: Check that no alias was found due to no contexts being inherited
        assert_eq!(result, None);
    }
}
