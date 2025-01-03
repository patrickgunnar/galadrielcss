use chrono::Local;
use indexmap::IndexMap;

use crate::{
    asts::CLASSINATOR,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    types::Classinator,
};

use super::{Crealion, CrealionContextType};

impl Crealion {
    /// Applies a tracking map to the Classinator structure based on the provided parameters.
    ///
    /// # Parameters
    /// - `class_name`: The name of the class to track.
    /// - `context_name`: The name of the context where the class is applied.
    /// - `derived_from`: The origin of the class (e.g., its parent definition).
    /// - `parent_context`: An optional parent context name.
    /// - `tracking_cls_names`: A vector of class names used for tracking.
    /// - `context_type`: The type of context (Central, Layout, Module).
    pub fn apply_tracking_map_to_classinator(
        &self,
        context_name: String,
        parent_context: Option<String>,
        context_type: CrealionContextType,
        tracking_map: IndexMap<String, IndexMap<String, Vec<String>>>,
    ) {
        let sender = self.sender.clone();

        tracing::debug!(
            "Applying tracking map to Classinator: context_name={}, context_type={:?}",
            context_name,
            context_type
        );

        // Determine the appropriate context node name based on the context type.
        let context_node_name = match context_type {
            CrealionContextType::Central => "central",
            CrealionContextType::Layout => "layouts",
            CrealionContextType::Module => "modules",
        };

        // Attempt to access the Classinator's section corresponding to the context node name.
        let mut classinator_data = match CLASSINATOR.get_mut(context_node_name) {
            Some(data) => {
                tracing::debug!(
                    "Successfully accessed the {context_node_name} section in CLASSINATOR AST."
                );

                data
            }
            None => {
                tracing::error!(
                    "Failed to access the {context_node_name} section in CLASSINATOR AST for context: {context_name}"
                );

                // If the section is not found, raise a critical error.
                let error = GaladrielError::raise_critical_other_error(
                    ErrorKind::AccessDeniedToClassinatorAST,
                    &format!("Failed to access the {context_node_name} section in CLASSINATOR AST"),
                    ErrorAction::Restart,
                );

                tracing::error!("Critical error raised: {:?}", error);

                // Create a notification to report the error.
                let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

                // Attempt to send the notification and log any failures.
                if let Err(err) = sender.send(notification) {
                    tracing::error!("Failed to send notification: {}", err);
                }

                return;
            }
        };

        // Match the Classinator data type and apply the tracking map accordingly.
        match *classinator_data {
            Classinator::Central(ref mut central_definitions) => {
                tracing::debug!("Applying tracking map to Central node.");

                *central_definitions = tracking_map
            }
            Classinator::Layouts(ref mut layouts_definitions) => {
                tracing::debug!("Applying tracking map to Layouts node.");

                self.apply_tracking_map_to_layouts_node(
                    context_name,
                    tracking_map,
                    layouts_definitions,
                );
            }
            Classinator::Modules(ref mut modules_definitions) => {
                tracing::debug!("Applying tracking map to Modules node.");

                self.apply_tracking_map_to_modules_node(
                    context_name,
                    parent_context,
                    tracking_map,
                    modules_definitions,
                );
            }
        }
    }

    /// Applies a tracking map to the layouts node within the Classinator structure.
    ///
    /// # Parameters
    /// - `derived_from`: The parent or source of the class definition.
    /// - `class_name`: The class name to add to the layouts node.
    /// - `context_name`: The context name associated with the layouts node.
    /// - `tracking_cls_names`: A vector of class names for tracking.
    /// - `layouts_definitions`: The layouts node definitions map to update.
    fn apply_tracking_map_to_layouts_node(
        &self,
        context_name: String,
        tracking_map: IndexMap<String, IndexMap<String, Vec<String>>>,
        layouts_definitions: &mut IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>,
    ) {
        tracing::info!(
            "Applying tracking map to layouts node: context_name={}",
            context_name
        );

        // Retrieve or create the map for the specified context name.
        let layouts_map = layouts_definitions.entry(context_name).or_default();

        *layouts_map = tracking_map;
    }

    /// Applies a tracking map to the modules node within the Classinator structure.
    ///
    /// # Parameters
    /// - `derived_from`: The parent or source of the class definition.
    /// - `class_name`: The class name to add to the modules node.
    /// - `context_name`: The context name associated with the modules node.
    /// - `parent_context`: An optional parent context name.
    /// - `tracking_cls_names`: A vector of class names for tracking.
    /// - `modules_definitions`: The modules node definitions map to update.
    fn apply_tracking_map_to_modules_node(
        &self,
        context_name: String,
        parent_context: Option<String>,
        tracking_map: IndexMap<String, IndexMap<String, Vec<String>>>,
        modules_definitions: &mut IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>,
        >,
    ) {
        // Use the provided parent context or default to "_".
        let parent_context = parent_context.unwrap_or("_".to_string());

        tracing::info!(
            "Applying tracking map to modules node: context_name={}, parent_context={:?}",
            context_name,
            parent_context
        );

        // Retrieve or create the map for the specified parent and context name.
        let parents_map = modules_definitions.entry(parent_context).or_default();
        let modules_map = parents_map.entry(context_name).or_default();

        *modules_map = tracking_map;
    }
}

#[cfg(test)]
mod classinator_tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext};
    use tokio::sync::broadcast;

    use crate::{
        asts::CLASSINATOR,
        crealion::{Crealion, CrealionContextType},
        types::Classinator,
    };

    #[test]
    fn central_map_should_exists_in_ast() {
        let (sender, _) = broadcast::channel(10);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        crealion.apply_tracking_map_to_classinator(
            "central".to_string(),
            None,
            CrealionContextType::Central,
            IndexMap::from([(
                "_".to_string(),
                IndexMap::from([(
                    "myTestClassinatorClass".to_string(),
                    vec![
                        "utility-cls-one".to_string(),
                        "utility-cls-two".to_string(),
                        "utility-cls-three".to_string(),
                    ],
                )]),
            )]),
        );

        let cls_map =
            CLASSINATOR
                .get("central")
                .and_then(|classinator_data| match &*classinator_data {
                    Classinator::Central(ref central_map) => central_map.get("_").and_then(|map| {
                        map.get("myTestClassinatorClass")
                            .and_then(|cls_map| Some(cls_map.to_owned()))
                    }),
                    _ => None,
                });

        assert!(cls_map.is_some());
        assert_eq!(
            cls_map.unwrap(),
            vec![
                "utility-cls-one".to_string(),
                "utility-cls-two".to_string(),
                "utility-cls-three".to_string(),
            ]
        );
    }

    #[test]
    fn layout_map_should_exists_in_ast() {
        let (sender, _) = broadcast::channel(10);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        crealion.apply_tracking_map_to_classinator(
            "classinatorLayoutContextName".to_string(),
            None,
            CrealionContextType::Layout,
            IndexMap::from([(
                "_".to_string(),
                IndexMap::from([(
                    "myTestClassinatorClass".to_string(),
                    vec![
                        "utility-cls-one".to_string(),
                        "utility-cls-two".to_string(),
                        "utility-cls-three".to_string(),
                    ],
                )]),
            )]),
        );

        let cls_map =
            CLASSINATOR
                .get("layouts")
                .and_then(|classinator_data| match &*classinator_data {
                    Classinator::Layouts(ref layouts_map) => layouts_map
                        .get("classinatorLayoutContextName")
                        .and_then(|context_map| {
                            context_map.get("_").and_then(|map| {
                                map.get("myTestClassinatorClass")
                                    .and_then(|cls_map| Some(cls_map.to_owned()))
                            })
                        }),
                    _ => None,
                });

        assert!(cls_map.is_some());
        assert_eq!(
            cls_map.unwrap(),
            vec![
                "utility-cls-one".to_string(),
                "utility-cls-two".to_string(),
                "utility-cls-three".to_string(),
            ]
        );
    }

    #[test]
    fn module_map_should_exists_in_ast() {
        let (sender, _) = broadcast::channel(10);

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        crealion.apply_tracking_map_to_classinator(
            "classinatorModuleContextName".to_string(),
            None,
            CrealionContextType::Module,
            IndexMap::from([(
                "_".to_string(),
                IndexMap::from([(
                    "myTestClassinatorClass".to_string(),
                    vec![
                        "utility-cls-one".to_string(),
                        "utility-cls-two".to_string(),
                        "utility-cls-three".to_string(),
                    ],
                )]),
            )]),
        );

        let cls_map =
            CLASSINATOR
                .get("modules")
                .and_then(|classinator_data| match &*classinator_data {
                    Classinator::Modules(ref modules_map) => {
                        modules_map.get("_").and_then(|no_parent_map| {
                            no_parent_map.get("classinatorModuleContextName").and_then(
                                |context_map| {
                                    context_map.get("_").and_then(|map| {
                                        map.get("myTestClassinatorClass")
                                            .and_then(|cls_map| Some(cls_map.to_owned()))
                                    })
                                },
                            )
                        })
                    }
                    _ => None,
                });

        assert!(cls_map.is_some());
        assert_eq!(
            cls_map.unwrap(),
            vec![
                "utility-cls-one".to_string(),
                "utility-cls-two".to_string(),
                "utility-cls-three".to_string(),
            ]
        );
    }
}
