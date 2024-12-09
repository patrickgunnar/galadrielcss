use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind},
    types::Stylitron,
};

use super::Astroform;

impl Astroform {
    /// Transforms variable definitions from the AST into CSS root variables.
    ///
    /// # Returns
    /// - A `JoinHandle` wrapping a `String` containing all formatted CSS variables.
    pub fn transform_variables(&self) -> JoinHandle<String> {
        let palantir_sender = self.palantir_sender.clone();
        let tab = self.tab.to_owned();
        let space = self.space.to_owned();
        let newline = self.newline.to_owned();
        let is_minified = self.is_minified;

        // Spawn a blocking task to process variables in a separate thread.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting variables transformation process.");

            let variables_map = match STYLITRON.get("variables") {
                Some(stylitron_data) => match &*stylitron_data {
                    Stylitron::Variables(ref variables) => variables.to_owned(),
                    _ => return String::new(),
                },
                None => {
                    Self::send_palantir_error_notification(
                        ErrorKind::AccessDeniedToStylitronAST,
                        ErrorAction::Restart,
                        "Failed to access the variables section in STYLITRON AST",
                        palantir_sender.clone(),
                    );

                    return String::new();
                }
            };

            tracing::info!("Completed variables transformation process");

            // Transform the retrieved variables map into CSS rules.
            Self::transform_variables_map(&tab, &space, &newline, 1, is_minified, variables_map)
        })
    }

    /// Transforms a variables map into CSS rules inside a `:root` block.
    ///
    /// # Arguments
    /// - `tab`: Indentation string (e.g., tabs or spaces).
    /// - `space`: A space character for formatting.
    /// - `newline`: A newline character for formatting.
    /// - `tab_size`: The indentation level for the current scope.
    /// - `is_minified`: Flag indicating whether the output should be minified.
    /// - `variables_map`: A map of variable contexts and their corresponding variable definitions.
    ///
    /// # Returns
    /// - A `String` containing formatted CSS root variables.
    fn transform_variables_map(
        tab: &str,
        space: &str,
        newline: &str,
        tab_size: usize,
        is_minified: bool,
        variables_map: IndexMap<String, IndexMap<String, Vec<String>>>,
    ) -> String {
        // Select either tabs for indentation or an empty string for minified output.
        let tab_or_empty = Self::select_tab_or_empty(&tab, tab_size);
        let mut variables_rules: Vec<String> = vec![];

        // Iterate over each context in the variables map.
        for (context_name, context_variables) in variables_map {
            tracing::trace!("Processing context '{}'", context_name);

            if !is_minified {
                // Resolve the context name and add a comment in the CSS for non-minified output.
                let context_name = Self::resolve_context_name(&context_name);

                variables_rules.push(format!(
                    "{}/* Variable (s) sourced from the '{}' context */",
                    tab.repeat(tab_size),
                    context_name
                ));
            }

            tracing::trace!("Transforming variables for context '{}'", context_name);

            // Transform the variables for the current context.
            let transformed_variables =
                Self::transform_context_variables(tab, space, newline, tab_size, context_variables);

            variables_rules.push(transformed_variables);
        }

        // Combine all variables into a single `:root` block.
        format!(
            "{}:root{}{{{}{}{}{}}}",
            tab_or_empty,
            space,
            newline,
            variables_rules.join(newline),
            newline,
            tab_or_empty
        )
    }
}
