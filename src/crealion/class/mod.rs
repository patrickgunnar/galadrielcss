use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use nenyr::types::class::NenyrStyleClass;
use tokio::task::JoinHandle;
use types::{Class, UtilityClass};

use crate::{
    asts::{CLASSINATOR, STYLITRON},
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::{Classinator, Stylitron},
    utils::generates_node_styles::generates_node_styles,
    GaladrielResult,
};

use super::Crealion;

mod patterns;
mod responsive;
mod styles;
pub mod types;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum ContextType {
    Central,
    Layout,
    Module,
}

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
            results
                .iter()
                .enumerate()
                .for_each(|(idx, result)| match result {
                    // Handle successful results.
                    Ok((utility_classes, process_alerts, utility_names)) => {
                        // Append any alerts generated during processing.
                        alerts.append(&mut process_alerts.to_vec());
                        // Add utility names to the `Class` instance.
                        my_class.set_utility_names(&mut utility_names.to_vec());

                        if idx == 0 {
                            // Update the `Class` instance with the utility classes.
                            my_class.set_classes(&mut utility_classes.to_vec());
                        } else {
                            // Update the `Class` instance with the utility responsive classes.
                            my_class.set_responsive_classes(&mut utility_classes.to_vec());
                        }
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

    /// Handles class definitions asynchronously by processing each class's utility,
    /// responsive classes, and configuration in the context of the provided context type.
    ///
    /// # Parameters
    /// - `context`: The name of the current context.
    /// - `parent_context`: Optional parent context if applicable.
    /// - `class_definitions`: A vector of `Class` objects to process.
    /// - `context_type`: Type of the context (e.g., Central, Layout, Module).
    ///
    /// # Returns
    /// A `JoinHandle` that resolves to a vector of `ShellscapeAlerts`, containing any
    /// notifications or errors generated during processing.
    pub fn handle_class_definitions(
        &self,
        context: String,
        parent_context: Option<String>,
        class_definitions: Vec<Class>,
        context_type: ContextType,
    ) -> JoinHandle<Vec<ShellscapeAlerts>> {
        tokio::task::spawn(async move {
            let mut alerts = vec![];

            // Iterate over all provided class definitions.
            for class_definition in class_definitions {
                // Launch processing tasks for each class definition.
                let processing_tasks = join_all(vec![
                    // Process utility classes.
                    Self::set_utility_classes(class_definition.get_classes(), false),
                    // Process responsive utility classes.
                    Self::set_utility_classes(class_definition.get_responsive_classes(), true),
                    // Save utility names to be tracked later.
                    Self::configure_class(
                        context.to_owned(),
                        parent_context.to_owned(),
                        class_definition.get_class_name(),
                        class_definition.get_deriving_from(),
                        class_definition.get_utility_names(),
                        context_type.to_owned(),
                    ),
                ])
                .await;

                // Handle the results of each processing task.
                processing_tasks.iter().for_each(|result| match result {
                    Err(err) => {
                        // Handle task-level error.
                        let error = GaladrielError::raise_general_other_error(
                            ErrorKind::TaskFailure,
                            &err.to_string(),
                            ErrorAction::Notify,
                        );

                        let notification =
                            ShellscapeAlerts::create_galadriel_error(Local::now(), error);

                        alerts.push(notification);
                    }
                    Ok(Err(err)) => {
                        let notification =
                            ShellscapeAlerts::create_galadriel_error(Local::now(), err.to_owned());

                        alerts.push(notification);
                    }
                    _ => {}
                });
            }

            alerts
        })
    }

    /// Sets utility classes for styling or responsive purposes by interacting with the STYLITRON AST.
    ///
    /// # Parameters
    /// - `utility_classes`: Vector of `UtilityClass` to add to the styles.
    /// - `is_responsive`: Indicates if the styles are responsive.
    ///
    /// # Returns
    /// A `JoinHandle` resolving to a `GaladrielResult<()>` indicating success or failure.
    fn set_utility_classes(
        utility_classes: Vec<UtilityClass>,
        is_responsive: bool,
    ) -> JoinHandle<GaladrielResult<()>> {
        tokio::task::spawn_blocking(move || {
            // Determine the STYLITRON node name based on responsiveness.
            let node_name = match is_responsive {
                true => "responsive",
                false => "styles",
            };

            // Retrieve mutable reference to the styles data from STYLITRON.
            let mut styles_data = match STYLITRON.get_mut(node_name) {
                Some(data) => data,
                None => {
                    // Return an error if access is denied.
                    return Err(GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToStylitronAST,
                        &format!("Access denied to STYLITRON AST for node: {}", node_name),
                        ErrorAction::Restart,
                    ));
                }
            };

            // Process the styles based on the node type.
            match *styles_data {
                Stylitron::Styles(ref mut styles_definitions) => {
                    // Add each utility class to the styles definitions.
                    for utility_class in utility_classes {
                        Self::add_style_definition(utility_class, styles_definitions);
                    }
                }
                Stylitron::ResponsiveStyles(ref mut responsive_definitions) => {
                    for utility_class in utility_classes {
                        let breakpoint = utility_class
                            .get_breakpoint()
                            .unwrap_or_else(|| "no-breakpoint".to_string());

                        // Retrieve or create the breakpoint styles.
                        let breakpoint_styles = responsive_definitions
                            .entry(breakpoint.clone())
                            .or_insert_with(generates_node_styles);

                        // Add the utility class to the breakpoint styles.
                        Self::add_style_definition(utility_class, breakpoint_styles);
                    }
                }
                _ => {}
            }

            Ok(())
        })
    }

    /// Adds a single style definition to the provided definitions map.
    ///
    /// # Parameters
    /// - `utility_class`: The `UtilityClass` to be added.
    /// - `definitions`: The mutable reference to the definitions map.
    fn add_style_definition(
        utility_class: UtilityClass,
        definitions: &mut IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
        >,
    ) {
        // Insert the utility class into the appropriate map hierarchy.
        let pattern_map = definitions
            .entry(utility_class.get_pattern())
            .or_insert_with(IndexMap::new);

        let importance_map = pattern_map
            .entry(utility_class.get_important())
            .or_insert_with(IndexMap::new);

        let property_map = importance_map
            .entry(utility_class.get_property())
            .or_insert_with(IndexMap::new);

        property_map
            .entry(utility_class.get_class_name())
            .or_insert(utility_class.get_value());
    }

    /// Configures a class in the CLASSINATOR AST for the specified context and type.
    ///
    /// # Parameters
    /// - `context`: Current context name.
    /// - `parent_context`: Optional parent context name.
    /// - `class_name`: Name of the class being configured.
    /// - `base_class`: Base class for inheritance, if any.
    /// - `utility_names`: Vector of utility class names to associate with this class.
    /// - `context_type`: Type of the context.
    ///
    /// # Returns
    /// A `JoinHandle` resolving to a `GaladrielResult<()>`.
    fn configure_class(
        context: String,
        parent_context: Option<String>,
        class_name: String,
        base_class: Option<String>,
        utility_names: Vec<String>,
        context_type: ContextType,
    ) -> JoinHandle<GaladrielResult<()>> {
        tokio::task::spawn_blocking(move || {
            // Determine the CLASSINATOR node name based on context type.
            let context_name = match context_type {
                ContextType::Central => "central",
                ContextType::Layout => "layouts",
                ContextType::Module => "modules",
            };

            // Retrieve mutable reference to the context data from CLASSINATOR.
            let mut context_data = match CLASSINATOR.get_mut(context_name) {
                Some(data) => data,
                None => {
                    // Return an error if access is denied.
                    return Err(GaladrielError::raise_critical_other_error(
                        ErrorKind::AccessDeniedToClassinatorAST,
                        &format!(
                            "Access denied to CLASSINATOR AST for node: {}",
                            context_name
                        ),
                        ErrorAction::Restart,
                    ));
                }
            };

            // Configure the class based on context type.
            match *context_data {
                Classinator::Central(ref mut central_definitions) => {
                    Self::set_classinator_children(
                        base_class,
                        class_name,
                        utility_names,
                        central_definitions,
                    );
                }
                Classinator::Layouts(ref mut layout_definitions) => {
                    Self::set_classinator_layout(
                        context,
                        base_class,
                        class_name,
                        utility_names,
                        layout_definitions,
                    );
                }
                Classinator::Modules(ref mut module_definitions) => {
                    Self::set_classinator_module(
                        context,
                        parent_context,
                        base_class,
                        class_name,
                        utility_names,
                        module_definitions,
                    );
                }
            }

            Ok(())
        })
    }

    /// Sets the child class definitions under a specified base class in the provided definitions map.
    ///
    /// # Arguments
    /// - `base_class`: The name of the base class (if any) under which the class is inherited.
    /// - `class_name`: The name of the class.
    /// - `utility_names`: A vector of utility names associated with the class.
    /// - `definitions`: A mutable reference to the map storing class hierarchy definitions.
    fn set_classinator_children(
        base_class: Option<String>,
        class_name: String,
        utility_names: Vec<String>,
        definitions: &mut IndexMap<String, IndexMap<String, Vec<String>>>,
    ) {
        // If no base/inherit class is provided, use a default placeholder "_".
        let base_class = match base_class {
            Some(name) => name,
            None => "_".to_string(),
        };

        // Retrieve or create a map of class definitions for the base/inherit class.
        let inherits_map = definitions.entry(base_class).or_insert_with(IndexMap::new);

        // Insert the class along with its utility names.
        inherits_map.entry(class_name).or_insert(utility_names);
    }

    /// Configures class definitions specific to a layout context.
    ///
    /// # Arguments
    /// - `context`: The name of the layout context.
    /// - `base_class`: The name of the base/inherit class (if any) within the layout.
    /// - `class_name`: The name of the class to define within the layout.
    /// - `utility_names`: A vector of utility names associated with the class.
    /// - `definitions`: A mutable reference to the map storing layout class definitions.
    fn set_classinator_layout(
        context: String,
        base_class: Option<String>,
        class_name: String,
        utility_names: Vec<String>,
        definitions: &mut IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>,
    ) {
        // Retrieve or create a layout-specific map for the given context.
        let layout_map = definitions.entry(context).or_insert_with(IndexMap::new);

        // Use the shared logic to add the class to the layout hierarchy.
        Self::set_classinator_children(base_class, class_name, utility_names, layout_map);
    }

    /// Configures class definitions specific to a module context.
    ///
    /// # Arguments
    /// - `context`: The name of the module context.
    /// - `parent_context`: The name of the parent context (if any) for the module.
    /// - `base_class`: The name of the base/inherit class (if any).
    /// - `class_name`: The name of the class to define within the module.
    /// - `utility_names`: A vector of utility names associated with the class.
    /// - `definitions`: A mutable reference to the map storing module class definitions.
    fn set_classinator_module(
        context: String,
        parent_context: Option<String>,
        base_class: Option<String>,
        class_name: String,
        utility_names: Vec<String>,
        definitions: &mut IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>,
        >,
    ) {
        // If no parent context is provided, use a default placeholder "_".
        let parent = match parent_context {
            Some(name) => name,
            None => "_".to_string(),
        };

        // Retrieve or create a parent-specific map for the module hierarchy.
        let parent_map = definitions.entry(parent).or_insert_with(IndexMap::new);
        // Retrieve or create a context-specific map within the parent hierarchy.
        let module_map = parent_map.entry(context).or_insert_with(IndexMap::new);

        // Use the shared logic to add the class to the module hierarchy.
        Self::set_classinator_children(base_class, class_name, utility_names, module_map);
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext, class::NenyrStyleClass};

    use crate::{
        asts::{CLASSINATOR, STYLITRON},
        crealion::{
            class::ContextType,
            mocks::test_helpers::{
                mock_aliases_node, mock_animations_node, mock_breakpoints_node, mock_themes_node,
                mock_variable_node,
            },
            Crealion,
        },
        types::{Classinator, Stylitron},
    };

    use super::types::{Class, UtilityClass};

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
        assert_eq!(processed_class.get_classes().len(), 6);
        assert_eq!(processed_class.get_responsive_classes().len(), 12);

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

        assert_eq!(processed_class.get_classes().len(), 6);
        assert_eq!(processed_class.get_responsive_classes().len(), 12);

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

    fn mock_class_definition() -> Class {
        let mut my_class = Class::new("myClassName", None);

        my_class.set_classes(&mut vec![UtilityClass::create_class(
            &None,
            "_",
            "\\!dpy-Yj4t",
            true,
            "display",
            "block",
        )]);

        my_class.set_responsive_classes(&mut vec![UtilityClass::create_class(
            &Some("min-width:430px".to_string()),
            ":hover",
            "hvr-bgd-color-d6t5",
            false,
            "background-color",
            "blue",
        )]);

        my_class.set_utility_names(&mut vec![
            "\\!dpy-Yj4t".to_string(),
            "hvr-bgd-color-d6t5".to_string(),
        ]);

        my_class
    }

    #[tokio::test]
    async fn test_handle_class_definitions() {
        let my_class = mock_class_definition();
        let crealion = Crealion::new(
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let result = crealion
            .handle_class_definitions(
                "myContextName".to_string(),
                None,
                vec![my_class],
                ContextType::Central,
            )
            .await;

        assert!(result.is_ok());

        let class_name_exists = STYLITRON.get("styles").and_then(|data| match &*data {
            Stylitron::Styles(styles) => styles.get("_").and_then(|pattern_node| {
                pattern_node.get("!important").and_then(|importance_node| {
                    importance_node.get("display").and_then(|property_node| {
                        property_node.get("\\!dpy-Yj4t").and_then(|_| Some(true))
                    })
                })
            }),
            _ => None,
        });

        assert!(class_name_exists.unwrap());

        let track_map = CLASSINATOR.get("central").and_then(|data| match &*data {
            Classinator::Central(central_node) => central_node.get("_").and_then(|inherit_node| {
                inherit_node
                    .get("myClassName")
                    .and_then(|class_entry| Some(class_entry.to_owned()))
            }),
            _ => None,
        });

        assert_eq!(
            track_map,
            Some(vec![
                "\\!dpy-Yj4t".to_string(),
                "hvr-bgd-color-d6t5".to_string(),
            ])
        );
    }
}
