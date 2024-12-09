use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind},
    types::Stylitron,
};

use super::Astroform;

impl Astroform {
    /// Transforms the main styles from the STYLITRON AST into CSS rules.
    ///
    /// This function retrieves the "styles" section from the `STYLITRON` AST and processes
    /// the data to convert it into valid CSS rules. It uses the helper method `transform_pseudo_selector`
    /// to handle the transformation of the styles, and runs the operation in a blocking task
    /// to ensure it does not interfere with the asynchronous runtime.
    ///
    /// # Returns
    /// - A `JoinHandle<String>` containing the resulting CSS rules as a string.
    pub fn transform_styles(&self) -> JoinHandle<String> {
        let palantir_sender = self.palantir_sender.clone();
        let tab = self.tab.to_owned();
        let space = self.space.to_owned();
        let newline = self.newline.to_owned();

        // Spawn a blocking task to process the styles.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting the transformation of styles.");

            let styles_map = match STYLITRON.get("styles") {
                Some(stylitron_data) => match &*stylitron_data {
                    Stylitron::Styles(ref styles) => styles.to_owned(),
                    _ => return String::new(),
                },
                None => {
                    Self::send_palantir_error_notification(
                        ErrorKind::AccessDeniedToStylitronAST,
                        ErrorAction::Restart,
                        "Failed to access the styles section in STYLITRON AST",
                        palantir_sender.clone(),
                    );

                    return String::new();
                }
            };

            tracing::debug!("Starting to transform styles map into CSS rules using 'transform_pseudo_selector'.");

            // Use the `transform_pseudo_selector` helper function to process the styles map into CSS rules.
            // The second argument (1) indicates the level of indentation for the resulting rules.
            Self::transform_pseudo_selector(&tab, &space, &newline, 1, styles_map)
        })
    }
}
