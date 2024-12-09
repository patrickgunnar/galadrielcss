use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind},
    types::Stylitron,
};

use super::Astroform;

impl Astroform {
    /// Transforms responsive styles from the AST into CSS media query rules.
    ///
    /// This function processes the responsive styles section from the `STYLITRON` AST
    /// and converts it into properly formatted CSS `@media` rules. The function runs
    /// in a separate blocking task to ensure that the async runtime is not affected.
    ///
    /// # Returns
    /// - A `JoinHandle<String>` that resolves to a string containing all responsive CSS rules.
    pub fn transform_responsive_styles(&self) -> JoinHandle<String> {
        let palantir_sender = self.palantir_sender.clone();
        let tab = self.tab.to_owned();
        let space = self.space.to_owned();
        let newline = self.newline.to_owned();

        // Spawn a blocking task for processing the responsive styles.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting transformation of responsive styles.");

            let mut responsive_css_rules: Vec<String> = vec![];

            let responsive_styles_map = match STYLITRON.get("responsive") {
                Some(stylitron_data) => match &*stylitron_data {
                    Stylitron::ResponsiveStyles(ref styles) => styles.to_owned(),
                    _ => return String::new(),
                },
                None => {
                    Self::send_palantir_error_notification(
                        ErrorKind::AccessDeniedToStylitronAST,
                        ErrorAction::Restart,
                        "Failed to access the responsive styles section in STYLITRON AST",
                        palantir_sender.clone(),
                    );

                    return String::new();
                }
            };

            // Iterate over the responsive styles map to process each breakpoint and its styles.
            for (breakpoint, styles_map) in responsive_styles_map {
                tracing::debug!(
                    "Processing responsive styles for breakpoint: {}",
                    breakpoint
                );

                // Transform the styles for the current breakpoint using `transform_pseudo_selector`.
                let style_rules =
                    Self::transform_pseudo_selector(&tab, &space, &newline, 2, styles_map);

                // Format the rules into a `@media` query and add it to the result vector.
                responsive_css_rules.push(format!(
                    "@media screen and ({}){}{{{}{}{}}}",
                    breakpoint, space, newline, style_rules, newline
                ));
            }

            tracing::info!("Finished transforming responsive styles into CSS media queries.");

            // Join all formatted `@media` rules with the newline character and return.
            responsive_css_rules.join(&newline)
        })
    }
}
