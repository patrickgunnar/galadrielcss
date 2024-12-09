use indexmap::IndexMap;

use super::Astroform;

impl Astroform {
    /// Transforms context-specific variables into CSS variable declarations.
    ///
    /// # Arguments
    /// - `tab`: The string used for indentation (e.g., spaces or tabs).
    /// - `space`: A space character for formatting.
    /// - `newline`: A newline character for formatting.
    /// - `tab_size`: The level of indentation to apply for the variables.
    /// - `context_variables`: A map containing variable names and their values.
    ///
    /// # Returns
    /// - A `String` containing the formatted CSS variable declarations.
    pub fn transform_context_variables(
        tab: &str,
        space: &str,
        newline: &str,
        tab_size: usize,
        context_variables: IndexMap<String, Vec<String>>,
    ) -> String {
        tracing::info!("Transforming context-specific variables into CSS variable declarations.");

        let mut variables_rules: Vec<String> = vec![];

        // Iterate through each variable in the context variables map.
        context_variables.iter().for_each(|(_, variable_entry)| {
            // Ensure the variable entry contains exactly two elements: name and value.
            if let [unique_var_name, var_value] = variable_entry.as_slice() {
                tracing::debug!(
                    "Transforming variable: {} with value: {}",
                    unique_var_name,
                    var_value
                );

                // Format the variable declaration and push it into the rules list.
                variables_rules.push(format!(
                    "{}{}:{}{};",
                    tab.repeat(tab_size),
                    unique_var_name,
                    space,
                    var_value
                ));
            }
        });

        // Join all variable declarations with newline characters.
        variables_rules.join(newline)
    }

    /// Transforms pseudo-selector styles into CSS rules.
    ///
    /// # Arguments
    /// - `tab`: The string used for indentation (e.g., spaces or tabs).
    /// - `space`: A space character for formatting.
    /// - `newline`: A newline character for formatting.
    /// - `tab_size`: The level of indentation to apply for the pseudo-selector styles.
    /// - `styles_map`: A map containing pseudo-selectors and their respective styles.
    ///
    /// # Returns
    /// - A `String` containing the formatted CSS rules for pseudo-selectors.
    pub fn transform_pseudo_selector(
        tab: &str,
        space: &str,
        newline: &str,
        tab_size: usize,
        styles_map: IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>>,
    ) -> String {
        tracing::info!("Transforming pseudo-selector styles into CSS rules.");

        // Select the indentation based on the tab size.
        let tab_or_empty = Self::select_tab_or_empty(&tab, tab_size);
        let mut formatted_css_rules: Vec<String> = vec![];

        // Iterate through each pseudo-selector in the styles map.
        for (pseudo_selector, importance_map) in styles_map {
            // Remove any leading underscores from the pseudo-selector name.
            let pseudo_selector = pseudo_selector.trim_start_matches('_');

            tracing::debug!("Processing pseudo-selector: {}", pseudo_selector);

            for (importance, properties_map) in importance_map {
                // Remove any leading underscores from the importance value.
                let importance = importance.trim_start_matches('_');

                tracing::debug!("Processing importance: {}", importance);

                // Iterate through properties and class mappings.
                properties_map.iter().for_each(|(property, class_map)| {
                    class_map.iter().for_each(|(class_name, value)| {
                        tracing::debug!(
                            "Transforming class: {} with property: {} = {} and importance: {}",
                            class_name,
                            property,
                            value,
                            importance
                        );

                        // Format each CSS rule and add it to the list of formatted rules.
                        let class = format!(
                            "{}.{}{}{}{{{}{}{}:{}{}{}{}{}}}",
                            tab_or_empty,
                            class_name,
                            pseudo_selector,
                            space,
                            newline,
                            tab.repeat(tab_size),
                            property,
                            space,
                            value,
                            importance,
                            newline,
                            tab_or_empty
                        );

                        formatted_css_rules.push(class);
                    });
                });
            }
        }

        // Join all formatted rules with newline characters.
        formatted_css_rules.join(&newline)
    }

    /// Selects either an indentation string or an empty string based on the tab size.
    ///
    /// # Arguments
    /// - `tab`: The string used for indentation (e.g., spaces or tabs).
    /// - `tab_size`: The level of indentation to determine whether to use the tab string.
    ///
    /// # Returns
    /// - A `String` containing the tab string if `tab_size > 1`, otherwise an empty string.
    pub fn select_tab_or_empty(tab: &str, tab_size: usize) -> String {
        if tab_size > 1 {
            return tab.to_string();
        }

        String::new()
    }
}
