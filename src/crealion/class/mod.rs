use chrono::Local;
use futures::future::join_all;
use nenyr::types::class::NenyrStyleClass;
use tokio::task::JoinHandle;
use types::Class;

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
};

use super::Crealion;

mod patterns;
mod responsive;
mod styles;
pub mod types;

impl Crealion {
    /// Processes a given style class asynchronously.
    ///
    /// This method spawns a task to process the provided `NenyrStyleClass`. It handles both
    /// non-responsive and responsive styles, combines the results, and generates the final
    /// `Class` object along with any associated alerts.
    ///
    /// # Parameters
    /// - `inherited_contexts`: A vector of strings representing inherited style contexts.
    /// - `class`: An instance of `NenyrStyleClass` containing the class name, style patterns,
    ///   responsive patterns, and an optional `is_important` flag.
    ///
    /// # Returns
    /// A `JoinHandle` that resolves to a tuple containing:
    /// - `Class`: The processed class with its associated styles and utility names.
    /// - `Vec<ShellscapeAlerts>`: A list of alerts or warnings encountered during processing.
    pub fn process_class(
        &self,
        inherited_contexts: Vec<String>,
        class: NenyrStyleClass,
    ) -> JoinHandle<(Class, Vec<ShellscapeAlerts>)> {
        // Spawn an asynchronous task using `tokio::task::spawn`.
        tokio::task::spawn(async move {
            // Initialize a vector to collect alerts.
            let mut alerts = vec![];
            // Clone the class name for processing.
            let class_name = class.class_name.to_owned();
            // Determine if the class has the `!important` flag.
            let is_important = match class.is_important {
                Some(v) => v,  // Use the provided value if `Some`.
                None => false, // Default to `false` if `None`.
            };

            // Create a new `Class` instance, initializing it with the class name
            // and any derivation information.
            let mut my_class = Class::new(&class_name, class.deriving_from);

            // Process non-responsive and responsive styles concurrently.
            let results = join_all(vec![
                // Collect non-responsive styles.
                Self::collect_non_responsive_styles(
                    inherited_contexts.clone(),      // Clone the inherited contexts.
                    class_name.clone(),              // Clone the class name.
                    is_important,                    // Pass the `!important` flag.
                    class.style_patterns.to_owned(), // Clone style patterns.
                ),
                // Process responsive styles.
                Self::process_responsive_styles(
                    inherited_contexts,                   // Pass inherited contexts.
                    class_name,                           // Pass the class name.
                    is_important,                         // Pass the `!important` flag.
                    class.responsive_patterns.to_owned(), // Clone responsive patterns.
                ),
            ])
            .await; // Await the completion of both tasks.

            // Iterate through the results of the asynchronous tasks.
            results.iter().for_each(|result| match result {
                // Handle successful results.
                Ok((utility_classes, process_alerts, utility_names)) => {
                    // Append any alerts generated during processing.
                    alerts.append(&mut process_alerts.to_vec());
                    // Update the `Class` instance with the utility classes.
                    my_class.set_classes(&mut utility_classes.to_vec());
                    // Add utility names to the `Class` instance.
                    my_class.set_utility_names(&mut utility_names.to_vec());
                }
                // Handle errors encountered during processing.
                Err(err) => {
                    alerts.push(ShellscapeAlerts::create_galadriel_error(
                        Local::now(),
                        GaladrielError::raise_general_other_error(
                            ErrorKind::TaskFailure,
                            &err.to_string(),
                            ErrorAction::Notify,
                        ),
                    ));
                }
            });

            // Return the processed class and any collected alerts.
            (my_class, alerts)
        })
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext, class::NenyrStyleClass};

    use crate::crealion::{
        mocks::test_helpers::{
            mock_aliases_node, mock_animations_node, mock_breakpoints_node, mock_themes_node,
            mock_variable_node,
        },
        Crealion,
    };

    fn mock_class(deriving_from: Option<String>) -> NenyrStyleClass {
        let mut class = NenyrStyleClass::new("myNenyrClassName".to_string(), deriving_from);

        class.is_important = Some(true);
        class.style_patterns = Some(IndexMap::from([
            (
                "_stylesheet".to_string(),
                IndexMap::from([
                    (
                        "nickname;bgdColor".to_string(),
                        "${secondaryColor}".to_string(),
                    ),
                    ("display".to_string(), "block".to_string()),
                    (
                        "animation-name".to_string(),
                        "${mySecondaryAnimation}".to_string(),
                    ),
                ]),
            ),
            (
                ":hover".to_string(),
                IndexMap::from([
                    (
                        "nickname;bgdColor".to_string(),
                        "${secondaryColor}".to_string(),
                    ),
                    ("display".to_string(), "block".to_string()),
                    (
                        "animation-name".to_string(),
                        "${mySecondaryAnimation}".to_string(),
                    ),
                ]),
            ),
        ]));

        class.responsive_patterns = Some(IndexMap::from([
            (
                "myMob02".to_string(),
                IndexMap::from([
                    (
                        "_stylesheet".to_string(),
                        IndexMap::from([
                            (
                                "nickname;bgdColor".to_string(),
                                "${secondaryColor}".to_string(),
                            ),
                            ("display".to_string(), "block".to_string()),
                            (
                                "animation-name".to_string(),
                                "${mySecondaryAnimation}".to_string(),
                            ),
                        ]),
                    ),
                    (
                        ":hover".to_string(),
                        IndexMap::from([
                            (
                                "nickname;bgdColor".to_string(),
                                "${secondaryColor}".to_string(),
                            ),
                            ("display".to_string(), "block".to_string()),
                            (
                                "animation-name".to_string(),
                                "${mySecondaryAnimation}".to_string(),
                            ),
                        ]),
                    ),
                ]),
            ),
            (
                "myDesk02".to_string(),
                IndexMap::from([
                    (
                        "_stylesheet".to_string(),
                        IndexMap::from([
                            (
                                "nickname;bgdColor".to_string(),
                                "${secondaryColor}".to_string(),
                            ),
                            ("display".to_string(), "block".to_string()),
                            (
                                "animation-name".to_string(),
                                "${mySecondaryAnimation}".to_string(),
                            ),
                        ]),
                    ),
                    (
                        ":hover".to_string(),
                        IndexMap::from([
                            (
                                "nickname;bgdColor".to_string(),
                                "${secondaryColor}".to_string(),
                            ),
                            ("display".to_string(), "block".to_string()),
                            (
                                "animation-name".to_string(),
                                "${mySecondaryAnimation}".to_string(),
                            ),
                        ]),
                    ),
                ]),
            ),
        ]));

        class
    }

    #[tokio::test]
    async fn test_process_class_with_no_derivation() {
        mock_animations_node();
        mock_breakpoints_node();
        mock_themes_node();
        mock_variable_node();
        mock_aliases_node();

        let inherited_contexts = vec!["myGlacialContext".to_string()];
        let class = mock_class(None);

        let crealion = Crealion::new(
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let (processed_class, alerts) = crealion
            .process_class(inherited_contexts, class)
            .await
            .unwrap();

        assert_eq!(alerts.len(), 0);
        assert_eq!(
            processed_class.get_class_name(),
            "myNenyrClassName".to_string()
        );

        assert_eq!(processed_class.get_deriving_from(), None);
        assert_eq!(processed_class.get_classes().len(), 18);

        assert_eq!(
            processed_class.get_utility_names(),
            vec![
                "\\!bgd-clr-exb8".to_string(),
                "\\!dpy-S4vd".to_string(),
                "\\!ntn-nm-N9V6".to_string(),
                "\\!hvr\\.bgd-clr-exb8".to_string(),
                "\\!hvr\\.dpy-S4vd".to_string(),
                "\\!hvr\\.ntn-nm-N9V6".to_string(),
                "mMb\\.\\!bgd-clr-exb8".to_string(),
                "mMb\\.\\!dpy-S4vd".to_string(),
                "mMb\\.\\!ntn-nm-N9V6".to_string(),
                "mMb\\.\\!hvr\\.bgd-clr-exb8".to_string(),
                "mMb\\.\\!hvr\\.dpy-S4vd".to_string(),
                "mMb\\.\\!hvr\\.ntn-nm-N9V6".to_string(),
                "mDk\\.\\!bgd-clr-exb8".to_string(),
                "mDk\\.\\!dpy-S4vd".to_string(),
                "mDk\\.\\!ntn-nm-N9V6".to_string(),
                "mDk\\.\\!hvr\\.bgd-clr-exb8".to_string(),
                "mDk\\.\\!hvr\\.dpy-S4vd".to_string(),
                "mDk\\.\\!hvr\\.ntn-nm-N9V6".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn test_process_class_with_derivation() {
        mock_animations_node();
        mock_breakpoints_node();
        mock_themes_node();
        mock_variable_node();
        mock_aliases_node();

        let inherited_contexts = vec!["myGlacialContext".to_string()];
        let class = mock_class(Some("myPlanetaryLayout".to_string()));

        let crealion = Crealion::new(
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let (processed_class, alerts) = crealion
            .process_class(inherited_contexts, class)
            .await
            .unwrap();

        assert_eq!(alerts.len(), 0);
        assert_eq!(
            processed_class.get_class_name(),
            "myNenyrClassName".to_string()
        );

        assert_eq!(
            processed_class.get_deriving_from(),
            Some("myPlanetaryLayout".to_string())
        );

        assert_eq!(processed_class.get_classes().len(), 18);

        assert_eq!(
            processed_class.get_utility_names(),
            vec![
                "\\!bgd-clr-exb8".to_string(),
                "\\!dpy-S4vd".to_string(),
                "\\!ntn-nm-N9V6".to_string(),
                "\\!hvr\\.bgd-clr-exb8".to_string(),
                "\\!hvr\\.dpy-S4vd".to_string(),
                "\\!hvr\\.ntn-nm-N9V6".to_string(),
                "mMb\\.\\!bgd-clr-exb8".to_string(),
                "mMb\\.\\!dpy-S4vd".to_string(),
                "mMb\\.\\!ntn-nm-N9V6".to_string(),
                "mMb\\.\\!hvr\\.bgd-clr-exb8".to_string(),
                "mMb\\.\\!hvr\\.dpy-S4vd".to_string(),
                "mMb\\.\\!hvr\\.ntn-nm-N9V6".to_string(),
                "mDk\\.\\!bgd-clr-exb8".to_string(),
                "mDk\\.\\!dpy-S4vd".to_string(),
                "mDk\\.\\!ntn-nm-N9V6".to_string(),
                "mDk\\.\\!hvr\\.bgd-clr-exb8".to_string(),
                "mDk\\.\\!hvr\\.dpy-S4vd".to_string(),
                "mDk\\.\\!hvr\\.ntn-nm-N9V6".to_string()
            ]
        );
    }
}
