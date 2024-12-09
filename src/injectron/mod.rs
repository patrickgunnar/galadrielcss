use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;

use crate::{
    asts::NAMER,
    error::{ErrorAction, ErrorKind, GaladrielError},
    intaker::intaker_contains_context_name::intaker_contains_context_name,
    GaladrielResult,
};

lazy_static! {
    pub static ref INJECTRON_RE: Regex = Regex::new(
        r#"\b(Class|Animation|Layout|Module)\s*\(\s*""\s*\)|\b(Class|Animation|Layout|Module)\s*\(\s*\)"#
    ).unwrap();
}

/// The `Injectron` struct is responsible for injecting context names, class names,
/// and animation names into a given content. It uses a combination of adjectives
/// and nouns from a predefined words bank to generate these names. The struct provides
/// methods for creating unique and meaningful names based on specific criteria.
#[derive(Clone, PartialEq, Debug)]
pub struct Injectron(String);

impl Injectron {
    /// Creates a new `Injectron` instance.
    ///
    /// # Arguments
    ///
    /// - `input` - A string slice representing the input content to be processed for injections.
    ///
    /// # Returns
    ///
    /// A new `Injectron` instance containing the provided input.
    pub fn new(input: &str) -> Self {
        Injectron(input.to_string())
    }

    /// Injects context, class, and animation names into the content stored in the `Injectron`.
    ///
    /// This method uses the stored content to perform replacements based on predefined patterns.
    /// It generates names dynamically using available adjectives and nouns from the words bank.
    ///
    /// # Returns
    ///
    /// A `GaladrielResult` containing the modified content if successful, or an error if the
    /// operation fails (e.g., due to missing words bank data).
    pub fn inject(&self) -> GaladrielResult<String> {
        tracing::info!("Starting injection process.");

        let adjectives = self.get_adjectives()?; // Retrieve adjectives from the words bank.
        let nouns = self.get_nouns()?; // Retrieve nouns from the words bank.

        self.start_injection(&adjectives, &nouns)
    }

    /// Performs the main injection logic by replacing patterns in the content.
    ///
    /// # Arguments
    ///
    /// - `adjectives` - A vector of adjective strings to be used for name generation.
    /// - `nouns` - A vector of noun strings to be used for name generation.
    ///
    /// # Returns
    ///
    /// A `GaladrielResult` containing the modified content or an error if injection fails.
    fn start_injection(
        &self,
        adjectives: &Vec<String>,
        nouns: &Vec<String>,
    ) -> GaladrielResult<String> {
        tracing::info!("Performing main injection logic.");

        let mut unavailable_names: Vec<String> = vec![];

        // Perform regex-based replacements on the content.
        let modified_content = INJECTRON_RE.replace_all(&self.0, |caps: &regex::Captures| {
            let capture = &caps[0];

            if capture.contains("Layout") || capture.contains("Module") {
                // Generate a context name for Layout or Module.
                let context_name = self.generates_context_names(adjectives, nouns);

                tracing::info!("Generated context name: {}", context_name);

                if capture.starts_with("Layout") {
                    return format!("Layout(\"{}\")", context_name);
                } else if capture.starts_with("Module") {
                    return format!("Module(\"{}\")", context_name);
                }
            } else if capture.contains("Class") || capture.contains("Animation") {
                // Generate a context-relative unique name for Class or Animation, avoiding duplicates.
                let relative_name =
                    self.generates_relative_names(adjectives, nouns, &unavailable_names);

                tracing::info!("Generated relative name: {}", relative_name);

                unavailable_names.push(relative_name.to_owned());

                if capture.starts_with("Class") {
                    return format!("Class(\"{}\")", relative_name);
                } else if capture.starts_with("Animation") {
                    return format!("Animation(\"{}\")", relative_name);
                }
            }

            tracing::warn!("No match for capture: {}", capture);

            // Return the original capture if no conditions match.
            return capture.to_string();
        });

        tracing::debug!("Modified content: {}", modified_content);

        Ok(modified_content.to_string())
    }

    /// Generates a context-relative unique name for classes or animations.
    ///
    /// # Arguments
    ///
    /// - `adjectives` - A vector of adjectives.
    /// - `nouns` - A vector of nouns.
    /// - `unavailable_names` - A list of names that are already in use and should be avoided.
    ///
    /// # Returns
    ///
    /// A unique name that does not clash with any in `unavailable_names`.
    fn generates_relative_names(
        &self,
        adjectives: &Vec<String>,
        nouns: &Vec<String>,
        unavailable_names: &Vec<String>,
    ) -> String {
        loop {
            let current_name = self.generates_name(adjectives, nouns);

            if !unavailable_names.contains(&current_name) {
                return current_name;
            }
        }
    }

    /// Generates a unique name for Layout or Module context.
    ///
    /// # Arguments
    ///
    /// - `adjectives` - A vector of adjectives.
    /// - `nouns` - A vector of nouns.
    ///
    /// # Returns
    ///
    /// A unique name that is not already part of an existing context.
    fn generates_context_names(&self, adjectives: &Vec<String>, nouns: &Vec<String>) -> String {
        loop {
            let current_name = self.generates_name(adjectives, nouns);

            if !intaker_contains_context_name(&current_name) {
                return current_name;
            }
        }
    }

    /// Generates a name by combining an adjective and a noun.
    ///
    /// # Arguments
    ///
    /// - `adjectives` - A vector of adjectives.
    /// - `nouns` - A vector of nouns.
    ///
    /// # Returns
    ///
    /// A string representing the generated name.
    fn generates_name(&self, adjectives: &Vec<String>, nouns: &Vec<String>) -> String {
        let mut rng = rand::thread_rng();

        let adj_index = rng.gen_range(0..adjectives.len());
        let noun_index = rng.gen_range(0..nouns.len());

        let adj_word = &adjectives[adj_index];
        let noun_word = &nouns[noun_index];

        format!("{}{}", adj_word, self.pascalify(noun_word))
    }

    /// Converts a string into PascalCase format.
    ///
    /// # Arguments
    ///
    /// - `input` - The input string to be PascalCased.
    ///
    /// # Returns
    ///
    /// The PascalCased version of the input string.
    fn pascalify(&self, input: &str) -> String {
        input
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|w| {
                let mut chars = w.chars();

                chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>())
                    .unwrap_or_default()
                    + chars.as_str()
            })
            .collect()
    }

    /// Retrieves adjectives from the words bank.
    ///
    /// # Returns
    ///
    /// A `GaladrielResult` containing a vector of adjectives or an error if the data is unavailable.
    fn get_adjectives(&self) -> GaladrielResult<Vec<String>> {
        tracing::info!("Retrieving adjectives from words bank.");

        if let Some(adjectives) = NAMER.get("adjectives") {
            return Ok(adjectives.value().to_owned());
        }

        Err(GaladrielError::raise_general_other_error(
            ErrorKind::Other,
            "Adjectives could not be retrieved from the words bank. Injection names for contexts, classes, or animations could not be operated due to this failure.",
            ErrorAction::Notify,
        ))
    }

    /// Retrieves nouns from the words bank.
    ///
    /// # Returns
    ///
    /// A `GaladrielResult` containing a vector of nouns or an error if the data is unavailable.
    fn get_nouns(&self) -> GaladrielResult<Vec<String>> {
        tracing::info!("Retrieving nouns from words bank.");

        if let Some(nouns) = NAMER.get("nouns") {
            return Ok(nouns.value().to_owned());
        }

        Err(GaladrielError::raise_general_other_error(
            ErrorKind::Other,
            "Nouns could not be retrieved from the words bank. Injection names for contexts, classes, or animations could not be operated due to this failure.",
            ErrorAction::Notify,
        ))
    }
}
