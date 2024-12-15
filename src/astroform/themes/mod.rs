use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind},
    types::Stylitron,
};

use super::Astroform;

impl Astroform {
    /// Transforms the themes from the STYLITRON AST into CSS rules.
    ///
    /// This function retrieves the "themes" section from the `STYLITRON` AST and processes it into
    /// CSS rules for both light and dark themes. It creates the necessary CSS rules by formatting
    /// context-specific theme variables and adding appropriate media queries for the `prefers-color-scheme`.
    /// Additionally, comments are added to the output when not minified.
    ///
    /// # Returns
    /// - A `JoinHandle<String>` containing the resulting CSS rules for the themes as a string.
    pub fn transform_themes(&self) -> JoinHandle<String> {
        let palantir_sender = self.palantir_sender.clone();
        let tab = self.tab.to_owned();
        let space = self.space.to_owned();
        let newline = self.newline.to_owned();
        let is_minified = self.is_minified;

        // Spawn a blocking task to process the themes.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting to transform themes from STYLITRON AST.");

            let themes_map = match STYLITRON.get("themes") {
                Some(stylitron_data) => match &*stylitron_data {
                    Stylitron::Themes(ref themes) => themes.to_owned(),
                    _ => return String::new(),
                },
                None => {
                    Self::send_palantir_error_notification(
                        ErrorKind::AccessDeniedToStylitronAST,
                        ErrorAction::Restart,
                        "Failed to access the themes section in STYLITRON AST",
                        palantir_sender.clone(),
                    );

                    return String::new();
                }
            };

            // Vectors to store the variables for the light and dark themes.
            let mut light_variables: Vec<String> = vec![];
            let mut dark_variables: Vec<String> = vec![];

            // Iterate through the themes map to process each context and theme schema.
            for (context_name, context_variables) in themes_map {
                tracing::info!("Processing theme context: {}", context_name);

                for (theme_schema, schema_variables) in context_variables {
                    tracing::debug!("Processing theme schema: {}", theme_schema);

                    // Transform the context variables into CSS variables.
                    let formatted_variables = Self::transform_context_variables(
                        &tab,
                        &space,
                        &newline,
                        2,
                        schema_variables,
                    );

                    if formatted_variables.is_empty() {
                        continue;
                    }

                    // Check if the theme is "light" or "dark" and add the appropriate variables.
                    if theme_schema == "light" {
                        // Add a comment for the light theme if not minified.
                        Self::add_comment_if_minified(
                            &tab,
                            is_minified,
                            &context_name,
                            &mut light_variables,
                        );

                        light_variables.push(formatted_variables);
                    } else {
                        // Add a comment for the light theme if not minified.
                        Self::add_comment_if_minified(
                            &tab,
                            is_minified,
                            &context_name,
                            &mut dark_variables,
                        );

                        dark_variables.push(formatted_variables);
                    }
                }
            }

            // Format the light and dark theme variables into valid CSS and return the result.
            Self::format_themes(&tab, &space, &newline, light_variables, dark_variables)
        })
    }

    /// Formats the light and dark theme variables into valid CSS rules.
    ///
    /// This function takes the variables for both light and dark themes and formats them into valid
    /// CSS rules, ensuring the proper structure and separation between the two themes.
    ///
    /// # Parameters
    /// - `tab`: The tab string used for indentation.
    /// - `space`: The space string used for formatting.
    /// - `newline`: The newline string used for formatting.
    /// - `light_variables`: The vector containing the variables for the light theme.
    /// - `dark_variables`: The vector containing the variables for the dark theme.
    ///
    /// # Returns
    /// A string containing the formatted CSS rules for both the light and dark themes.
    fn format_themes(
        tab: &str,
        space: &str,
        newline: &str,
        light_variables: Vec<String>,
        dark_variables: Vec<String>,
    ) -> String {
        tracing::debug!("Formatting light and dark theme variables into CSS rules.");

        // Format the light and dark theme variables using the format_theme_schema method.
        let formatted_light_schema =
            Self::format_theme_schema(&tab, &space, &newline, "light", light_variables);

        let formatted_dark_schema =
            Self::format_theme_schema(&tab, &space, &newline, "dark", dark_variables);

        // Combine the light and dark theme schemas, ensuring proper structure.
        if !formatted_light_schema.is_empty() && !formatted_dark_schema.is_empty() {
            tracing::info!("Themes transformation completed.");

            format!(
                "{}{}{}",
                formatted_light_schema, newline, formatted_dark_schema
            )
        } else if !formatted_light_schema.is_empty() {
            tracing::debug!("No variables for dark theme to format.");

            formatted_light_schema
        } else if !formatted_dark_schema.is_empty() {
            tracing::debug!("No variables for light theme to format.");

            formatted_dark_schema
        } else {
            tracing::debug!("Both light and dark theme schemas are empty.");

            String::new()
        }
    }

    /// Formats the theme schema into valid CSS rules with a media query.
    ///
    /// This function takes the theme variables and creates a valid CSS media query
    /// for the given theme schema (either "light" or "dark"), and then formats the variables
    /// into the appropriate CSS rules under the `:root` selector.
    ///
    /// # Parameters
    /// - `tab`: The tab string used for indentation.
    /// - `space`: The space string used for formatting.
    /// - `newline`: The newline string used for formatting.
    /// - `schema`: The theme schema, either "light" or "dark".
    /// - `variables`: The vector containing the variables for the theme.
    ///
    /// # Returns
    /// A string containing the CSS rules for the given theme schema.
    fn format_theme_schema(
        tab: &str,
        space: &str,
        newline: &str,
        schema: &str,
        variables: Vec<String>,
    ) -> String {
        tracing::debug!("Formatting theme schema for: {}", schema);

        // Return an empty string if there are no variables to format.
        if variables.is_empty() {
            return String::new();
        }

        format!(
            "@media{}(prefers-color-scheme:{}){}{{{}{}:root{}{{{}{}{}{}}}{}}}",
            space,
            schema,
            space,
            newline,
            tab,
            space,
            newline,
            variables.join(newline),
            newline,
            tab,
            newline
        )
    }

    /// Adds a comment for the context if not minified.
    ///
    /// This function adds a comment indicating the context from which the theme variables are sourced.
    /// The comment is added only if the `is_minified` flag is set to `false`.
    ///
    /// # Parameters
    /// - `tab`: The tab string used for indentation.
    /// - `is_minified`: A boolean indicating whether the output is minified.
    /// - `context_name`: The name of the context from which the variables are sourced.
    /// - `schema_vec`: The vector to which the comment should be added if not minified.
    fn add_comment_if_minified(
        tab: &str,
        is_minified: bool,
        context_name: &str,
        schema_vec: &mut Vec<String>,
    ) {
        if !is_minified {
            tracing::info!("Adding comment for context: {}", context_name);

            let context_name = Self::resolve_context_name(context_name);

            schema_vec.push(format!(
                "{}/* Variable (s) sourced from the '{}' context */",
                tab.repeat(2),
                context_name
            ));
        }
    }
}
