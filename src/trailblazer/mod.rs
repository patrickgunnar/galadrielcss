use indexmap::IndexMap;

use crate::{
    asts::{CLASSINATOR, CLASTRACK},
    types::{Classinator, Clastrack},
};

#[derive(Clone, Debug)]
pub struct Trailblazer {}

impl Default for Trailblazer {
    fn default() -> Self {
        Self {}
    }
}

/// Implements the logic for processing and inheriting Nenyr classes
/// and their corresponding CSS utility classes.
///
/// This implementation handles the mapping and inheritance between
/// different contextual categories: "central", "layouts", and "modules."
impl Trailblazer {
    /// Processes and updates the `CLASTRACK` with the transformed and inherited
    /// mappings for central, layouts, and modules contexts.
    pub fn blazer(&self) {
        tracing::info!("Starting the processing of class mappings in Trailblazer.");

        // Extract the central, layouts and modules class mappings.
        let central_map = self.extract_central_node();
        let layouts_map = self.extract_layouts_node();
        let modules_map = self.extract_modules_node();

        tracing::debug!("Extracted central, layouts and modules node.");

        // Transform the central class mapping.
        let transformed_central = self.transform_class_map(&central_map);

        tracing::info!("Processing inheritance for central, layout and module contexts.");

        // Process inheritance for the central, layouts and modules class mappings.
        let inherited_central = self.process_inheritance(&vec![], &transformed_central);
        let inherited_layouts = self.process_layouts_inheritance(&inherited_central, layouts_map);
        let inherited_modules =
            self.process_modules_inheritance(&inherited_central, &inherited_layouts, modules_map);

        tracing::info!("Updating CLASTRACK with processed mappings.");

        // Update the CLASTRACK with the processed mappings for each context.
        CLASTRACK.insert("central".to_string(), Clastrack::Central(inherited_central));
        CLASTRACK.insert("layouts".to_string(), Clastrack::Layouts(inherited_layouts));
        CLASTRACK.insert("modules".to_string(), Clastrack::Modules(inherited_modules));

        tracing::info!("Completed processing of class mappings in Trailblazer.");
    }

    /// Processes inheritance for modules, combining central and layouts context mappings.
    ///
    /// # Parameters
    /// - `central_map`: The inherited central class mapping.
    /// - `layouts_map`: The inherited layouts class mapping.
    /// - `modules_map`: The raw modules class mapping to be transformed and inherited.
    ///
    /// # Returns
    /// A transformed and inherited class mapping for modules.
    fn process_modules_inheritance(
        &self,
        central_map: &IndexMap<String, String>,
        layouts_map: &IndexMap<String, IndexMap<String, String>>,
        modules_map: IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>,
        >,
    ) -> IndexMap<String, IndexMap<String, String>> {
        tracing::info!("Starting module inheritance processing.");

        modules_map
            .iter()
            .flat_map(|(layout_name, module_map)| {
                // Retrieve the layout mapping or use an empty map if it doesn't exist.
                let layout_map = match layouts_map.get(layout_name) {
                    Some(map) => map.to_owned(),
                    None => IndexMap::new(),
                };

                // Process each module's inheritance using central and layout mappings.
                module_map
                    .iter()
                    .map(|(module_name, inherited_map)| {
                        // Create the inheritance vector, using the collected layout and central maps.
                        let mut inheritance_vec =
                            vec![layout_map.to_owned(), central_map.to_owned()];

                        // Remove empty context from inheritance vector.
                        inheritance_vec.retain(|v| !v.is_empty());

                        // Transform and process inheritance for the module.
                        let transformed_map = self.transform_class_map(inherited_map);
                        let processed_inheritance =
                            self.process_inheritance(&inheritance_vec, &transformed_map);

                        (module_name.to_owned(), processed_inheritance)
                    })
                    .collect::<IndexMap<_, _>>()
            })
            .collect::<IndexMap<_, _>>()
    }

    /// Processes inheritance for layouts using the central context mapping.
    ///
    /// # Parameters
    /// - `central_map`: The inherited central class mapping.
    /// - `layouts_map`: The raw layouts class mapping to be transformed and inherited.
    ///
    /// # Returns
    /// A transformed and inherited class mapping for layouts.
    fn process_layouts_inheritance(
        &self,
        central_map: &IndexMap<String, String>,
        layouts_map: IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>,
    ) -> IndexMap<String, IndexMap<String, String>> {
        tracing::info!("Starting layout inheritance processing.");

        layouts_map
            .iter()
            .map(|(layout_name, inherited_map)| {
                // Create the inheritance vector, using the central map.
                let inheritance_vec = vec![central_map.to_owned()];

                // Transform and process inheritance for each layout.
                let transformed_map = self.transform_class_map(inherited_map);
                let processed_inheritance =
                    self.process_inheritance(&inheritance_vec, &transformed_map);

                (layout_name.to_owned(), processed_inheritance)
            })
            .collect::<IndexMap<_, _>>()
    }

    /// Processes a transformed class map by applying inherited classes from the provided contexts.
    ///
    /// # Parameters
    /// - `inheritance_vec`: A vector of previously inherited class mappings to apply.
    /// - `transformed_map`: The transformed class map to process.
    ///
    /// # Returns
    /// A fully inherited class mapping.
    fn process_inheritance(
        &self,
        inheritance_vec: &Vec<IndexMap<String, String>>,
        transformed_map: &IndexMap<String, IndexMap<String, String>>,
    ) -> IndexMap<String, String> {
        tracing::debug!("Starting inheritance processing...");

        let mut inherited_map = IndexMap::new();

        for (inherited_name, class_map) in transformed_map {
            tracing::debug!(inherited_name, "Processing class map for inherited name.");

            if inherited_name == "_" {
                tracing::debug!("Directly adding classes without additional inheritance for '_'.");

                // Directly add classes without additional inheritance.
                for (class_name, utility_names) in class_map {
                    inherited_map.insert(class_name.to_owned(), utility_names.to_owned());
                }

                continue;
            }

            // Build a context chain for resolving inheritance of classes.
            let mut inherited_contexts = inheritance_vec.to_vec();
            inherited_contexts.insert(0, inherited_map.to_owned());

            // Resolve the inherited utility classes for the current map.
            let inherited_classes = inherited_contexts
                .iter()
                .find_map(|context_map| match context_map.get(inherited_name) {
                    Some(v) => {
                        let mut inherited_value = v.to_owned();

                        if inherited_value.len() > 0 {
                            inherited_value.push_str(" ");
                        }

                        Some(inherited_value)
                    }
                    None => None,
                })
                .unwrap_or("".to_string());

            // Apply the inherited classes to each utility name in the map.
            for (class_name, utility_names) in class_map {
                let mut formatted_utility_names = utility_names.to_owned();
                formatted_utility_names.insert_str(0, &inherited_classes);

                tracing::debug!(
                    class_name,
                    "Inheriting utilities: {}",
                    formatted_utility_names
                );

                inherited_map.insert(class_name.to_owned(), formatted_utility_names);
            }
        }

        tracing::debug!("Finished processing inheritance.");

        inherited_map
    }

    /// Transforms a class map by joining the utility classes into a single space-separated string.
    ///
    /// # Arguments
    /// - `class_map`: A reference to an `IndexMap` where each key is an inherited name, and each value
    ///   is another map associating class names with a vector of utility class names.
    ///
    /// # Returns
    /// - An `IndexMap` where each utility map is transformed into a single string of space-separated
    ///   class names.
    fn transform_class_map(
        &self,
        class_map: &IndexMap<String, IndexMap<String, Vec<String>>>,
    ) -> IndexMap<String, IndexMap<String, String>> {
        tracing::debug!("Transforming class map...");

        class_map
            .iter()
            .map(|(inherited_name, class_map)| {
                // Transform each class's utility map into a single string.
                let transformed_class_map = class_map
                    .iter()
                    .map(|(class_name, utility_map)| {
                        let joined_utilities = utility_map.join(" ");

                        tracing::debug!(class_name, "Joined utilities: {}", joined_utilities);

                        (class_name.to_owned(), joined_utilities)
                    })
                    .collect::<IndexMap<String, String>>();

                // Map the inherited name to its transformed class map.
                (inherited_name.to_owned(), transformed_class_map)
            })
            .collect::<IndexMap<_, _>>()
    }

    /// Extracts the central node from the `CLASSINATOR` global map.
    ///
    /// # Returns
    /// - An `IndexMap` representing the central class map, where each key is an inherited name,
    ///   and each value is a map of class names to vectors of utility classes.
    fn extract_central_node(&self) -> IndexMap<String, IndexMap<String, Vec<String>>> {
        tracing::debug!("Extracting central node...");

        match CLASSINATOR.get("central") {
            Some(classinator_data) => match &*classinator_data {
                Classinator::Central(ref central_data) => central_data.to_owned(),
                _ => IndexMap::new(),
            },
            None => IndexMap::new(),
        }
    }

    /// Extracts the layouts node from the `CLASSINATOR` global map.
    ///
    /// # Returns
    /// - An `IndexMap` representing the layouts class map, where each key is an inherited name,
    ///   and each value is a nested map of class names to vectors of utility classes.
    fn extract_layouts_node(
        &self,
    ) -> IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>> {
        tracing::debug!("Extracting layouts node...");

        match CLASSINATOR.get("layouts") {
            Some(classinator_data) => match &*classinator_data {
                Classinator::Layouts(ref layouts_data) => layouts_data.to_owned(),
                _ => IndexMap::new(),
            },
            None => IndexMap::new(),
        }
    }

    /// Extracts the modules node from the `CLASSINATOR` global map.
    ///
    /// # Returns
    /// - An `IndexMap` representing the modules class map, where each key is an inherited name,
    ///   and each value is a deeply nested map of class names to vectors of utility classes.
    fn extract_modules_node(
        &self,
    ) -> IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>> {
        tracing::debug!("Extracting modules node...");

        match CLASSINATOR.get("modules") {
            Some(classinator_data) => match &*classinator_data {
                Classinator::Modules(ref modules_data) => modules_data.to_owned(),
                _ => IndexMap::new(),
            },
            None => IndexMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::{
        asts::{CLASSINATOR, CLASTRACK},
        types::{Classinator, Clastrack},
    };

    use super::Trailblazer;

    fn mock_central_node() {
        CLASSINATOR.insert(
            "central".to_string(),
            Classinator::Central(IndexMap::from([(
                "_".to_string(),
                IndexMap::from([(
                    "myCentralClassName".to_string(),
                    vec![
                        "utility-name-one".to_string(),
                        "utility-name-two".to_string(),
                        "utility-name-three".to_string(),
                    ],
                )]),
            )])),
        );
    }

    fn mock_layouts_node() {
        CLASSINATOR.insert(
            "layouts".to_string(),
            Classinator::Layouts(IndexMap::from([(
                "myClassinatorLayoutNam".to_string(),
                IndexMap::from([(
                    "myCentralClassName".to_string(),
                    IndexMap::from([(
                        "myLayoutClassName".to_string(),
                        vec![
                            "utility-layout-name-one".to_string(),
                            "utility-layout-name-two".to_string(),
                            "utility-layout-name-three".to_string(),
                        ],
                    )]),
                )]),
            )])),
        );
    }

    fn mock_modules_node() {
        CLASSINATOR.insert(
            "modules".to_string(),
            Classinator::Modules(IndexMap::from([(
                "myClassinatorLayoutNam".to_string(),
                IndexMap::from([(
                    "myClassinatorModuleNam".to_string(),
                    IndexMap::from([(
                        "myLayoutClassName".to_string(),
                        IndexMap::from([(
                            "myModuleClassName".to_string(),
                            vec![
                                "utility-module-name-one".to_string(),
                                "utility-module-name-two".to_string(),
                                "utility-module-name-three".to_string(),
                            ],
                        )]),
                    )]),
                )]),
            )])),
        );
    }

    #[test]
    fn central_map_is_success() {
        mock_central_node();

        Trailblazer::default().blazer();

        let utility_cls = match CLASTRACK.get("central") {
            Some(clastrack_data) => match &*clastrack_data {
                Clastrack::Central(ref central_node) => {
                    central_node.get("myCentralClassName").map(|v| v.to_owned())
                }
                _ => None,
            },
            None => None,
        };

        assert!(utility_cls.is_some());
        assert_eq!(
            utility_cls.unwrap(),
            "utility-name-one utility-name-two utility-name-three"
        );
    }

    #[test]
    fn layout_map_is_success() {
        mock_central_node();
        mock_layouts_node();

        Trailblazer::default().blazer();

        let utility_cls = match CLASTRACK.get("layouts") {
            Some(clastrack_data) => match &*clastrack_data {
                Clastrack::Layouts(ref layouts_node) => layouts_node
                    .get("myClassinatorLayoutNam")
                    .and_then(|context_map| {
                        context_map.get("myLayoutClassName").map(|v| v.to_owned())
                    }),
                _ => None,
            },
            None => None,
        };

        assert!(utility_cls.is_some());
        assert_eq!(
            utility_cls.unwrap(),
            "utility-name-one utility-name-two utility-name-three utility-layout-name-one utility-layout-name-two utility-layout-name-three"
        );
    }

    #[test]
    fn module_map_is_success() {
        mock_central_node();
        mock_layouts_node();
        mock_modules_node();

        Trailblazer::default().blazer();

        let utility_cls = match CLASTRACK.get("modules") {
            Some(clastrack_data) => match &*clastrack_data {
                Clastrack::Modules(ref modules_node) => modules_node
                    .get("myClassinatorModuleNam")
                    .and_then(|context_map| {
                        context_map.get("myModuleClassName").map(|v| v.to_owned())
                    }),
                _ => None,
            },
            None => None,
        };

        assert!(utility_cls.is_some());
        assert_eq!(
            utility_cls.unwrap(),
            "utility-name-one utility-name-two utility-name-three utility-layout-name-one utility-layout-name-two utility-layout-name-three utility-module-name-one utility-module-name-two utility-module-name-three"
        );
    }
}
