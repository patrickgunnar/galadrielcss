use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind},
    types::Stylitron,
};

use super::Astroform;

impl Astroform {
    /// Transforms the typefaces section of the STYLITRON AST into valid CSS `@font-face` rules.
    ///
    /// This function processes the typefaces data stored in the STYLITRON AST, formats it into
    /// CSS rules, and returns them as a string. It handles different font formats and ensures
    /// proper error handling when the STYLITRON AST is inaccessible.
    ///
    /// # Returns
    /// A `JoinHandle<String>` which contains the CSS rules as a `String` when the task completes.
    pub fn transform_typefaces(&self) -> JoinHandle<String> {
        let palantir_sender = self.palantir_sender.clone();
        let tab = self.tab.to_owned();
        let space = self.space.to_owned();
        let newline = self.newline.to_owned();

        // Spawn a blocking task to process the typefaces
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting typefaces transformation process");

            let mut typefaces_css_rules: Vec<String> = vec![];

            let typefaces_map = match STYLITRON.get("typefaces") {
                Some(stylitron_data) => match &*stylitron_data {
                    Stylitron::Typefaces(ref typefaces) => typefaces.to_owned(),
                    _ => return String::new(),
                },
                None => {
                    Self::send_palantir_error_notification(
                        ErrorKind::AccessDeniedToStylitronAST,
                        ErrorAction::Restart,
                        "Failed to access the typefaces section in STYLITRON AST",
                        palantir_sender.clone(),
                    );

                    return String::new();
                }
            };

            // Iterate over the typefaces data
            for (identifier, value) in typefaces_map {
                tracing::debug!("Processing typeface: {}", identifier);

                // Split the value (path) of the font to get its extension
                let split_value: Vec<&str> = value.split(".").collect();

                // Check if there's an extension and format the CSS rule accordingly
                if let Some(extension) = split_value.get(split_value.len() - 1) {
                    // Get the proper font format from the extension
                    if let Some(current_format) = Self::typefaces_format(extension) {
                        tracing::debug!(
                            "Mapping extension '{}' to CSS format '{}'.",
                            extension,
                            current_format
                        );

                        // Format the @font-face CSS rule and add it to the list
                        let formatted_typeface = format!(
                            "@font-face{}{{{}{}font-family:{}{};{}{}src:{}url({:#?}){}format({:#?}){}}}",
                            space, newline, tab, space, identifier, newline, tab, space, value, space, current_format, newline
                        );

                        tracing::info!(
                            "Successfully created CSS rule for typeface: {}",
                            identifier
                        );

                        typefaces_css_rules.push(formatted_typeface);
                    }
                }
            }

            tracing::info!("CSS @font-face rule generation complete.");

            // Join the generated CSS rules into a single string and return
            typefaces_css_rules.join(&newline)
        })
    }

    /// Returns the corresponding CSS format for a given font file extension.
    ///
    /// This helper function maps font file extensions to their corresponding CSS format
    /// (e.g., "woff" to "woff", "ttf" to "truetype").
    ///
    /// # Arguments
    /// * `current_format` - The file extension of the font (e.g., "woff", "ttf").
    ///
    /// # Returns
    /// An `Option<String>` containing the CSS format for the font, or `None` if the format is unrecognized.
    fn typefaces_format(current_format: &str) -> Option<String> {
        // Match the font extension to the corresponding CSS format
        match current_format {
            "woff" => Some("woff".to_string()),
            "woff2" => Some("woff2".to_string()),
            "ttf" => Some("truetype".to_string()),
            "otf" => Some("opentype".to_string()),
            "eot" => Some("embedded-opentype".to_string()),
            "svg" => Some("svg".to_string()),
            _ => None,
        }
    }
}
