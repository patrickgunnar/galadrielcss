use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind},
    types::Stylitron,
};

use super::Astroform;

impl Astroform {
    /// Transforms import statements from the AST into CSS `@import` rules.
    ///
    /// This function processes the imports defined in the `STYLITRON` AST and
    /// converts them into a formatted CSS string containing `@import` statements.
    /// It runs in a separate blocking task to avoid blocking the async runtime.
    ///
    /// # Returns
    /// - A `JoinHandle<String>` that resolves to a string containing all `@import` rules.
    pub fn transform_imports(&self) -> JoinHandle<String> {
        let palantir_sender = self.palantir_sender.clone();
        let newline = self.newline.to_owned();

        // Spawn a blocking task to handle the transformation.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting transformation of import statements.");

            let mut import_css_rules: Vec<String> = vec![];

            let imports_map = match STYLITRON.get("imports") {
                Some(stylitron_data) => match &*stylitron_data {
                    Stylitron::Imports(ref imports) => imports.to_owned(),
                    _ => return String::new(),
                },
                None => {
                    Self::send_palantir_error_notification(
                        ErrorKind::AccessDeniedToStylitronAST,
                        ErrorAction::Restart,
                        "Failed to access the imports section in STYLITRON AST",
                        palantir_sender.clone(),
                    );

                    return String::new();
                }
            };

            // Iterate through the imports map to format each import as a CSS rule.
            for (import, _) in imports_map {
                tracing::debug!("Formatting @import rule for URL: {}", import);

                // Format the import URL as a CSS `@import` rule and add it to the list.
                import_css_rules.push(format!("@import url({:#?});", import));
            }

            tracing::info!("Finished transforming import statements into CSS @import rules.");

            // Join all formatted import rules using the newline character and return.
            import_css_rules.join(&newline)
        })
    }
}
