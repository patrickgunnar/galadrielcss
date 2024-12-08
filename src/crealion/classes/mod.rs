use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use nenyr::types::class::NenyrStyleClass;
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{
    asts::STYLITRON,
    crealion::utils::{camelify::camelify, pascalify::pascalify},
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    types::Stylitron,
    utils::generates_node_styles::generates_node_styles,
};

use super::{
    processors::{
        aliases::resolve_alias_identifier,
        breakpoints::resolve_breakpoint_identifier,
        variables::{resolve_variable_from_str, VariablesOption},
    },
    utils::generate_utility_class_name::generate_utility_class_name,
    Crealion,
};

impl Crealion {
    /// Processes style classes asynchronously for a given context.
    ///
    /// # Arguments
    /// - `context_name`: The name of the current context being processed.
    /// - `inherited_contexts`: A vector of parent contexts to inherit styles from.
    /// - `parent_context`: An optional name of the parent context.
    /// - `classes_data`: A map of class names to their corresponding style definitions.
    /// - `context_type`: The type of context being processed (e.g., layout, module).
    pub async fn process_classes(
        &self,
        context_name: String,
        inherited_contexts: Vec<String>,
        classes_data: IndexMap<String, NenyrStyleClass>,
        tracking_map: &mut IndexMap<String, IndexMap<String, Vec<String>>>,
    ) {
        tracing::info!(
            context_name = %context_name,
            num_classes = classes_data.len(),
            "Starting to process classes for context"
        );

        // Iterate over all style classes in the provided data.
        for class in classes_data.into_values() {
            let class_name = class.class_name;
            let derived_from = class.deriving_from.unwrap_or("_".to_string());
            let is_important = class.is_important.unwrap_or(false);

            tracing::debug!(
                class_name = %class_name,
                derived_from = %derived_from,
                is_important,
                "Processing class"
            );

            // Process both non-responsive and responsive styles concurrently.
            let tracking_cls_names = join_all(vec![
                self.process_non_responsive_styles(
                    class_name.to_owned(),
                    is_important,
                    context_name.to_owned(),
                    inherited_contexts.to_vec(),
                    class.style_patterns,
                ),
                self.process_responsive_styles(
                    class_name.to_owned(),
                    is_important,
                    context_name.to_owned(),
                    inherited_contexts.to_vec(),
                    class.responsive_patterns,
                ),
            ])
            .await
            .iter()
            .fold(vec![], |mut acc, result| {
                match result {
                    Ok(map) => {
                        tracing::trace!(
                            class_name = %class_name,
                            processed_styles = map.len(),
                            "Successfully processed styles"
                        );

                        acc.append(&mut map.to_vec());
                    }
                    Err(err) => {
                        tracing::error!(
                            class_name = %class_name,
                            error = %err,
                            "Failed to process styles"
                        );

                        self.handle_classes_task_failure(&err.to_string());
                    }
                }

                acc
            });

            tracing::info!(
                class_name = %class_name,
                num_tracking_names = tracking_cls_names.len(),
                "Finished processing styles for class"
            );

            // Retrieve or initialize the mapping for the parent context (`derived_from`) in the `tracking_map`.
            // If no entry exists for `derived_from`, a default value is created.
            //
            // Then, insert the `class_name` and its associated `tracking_cls_names` into this context's mapping.
            // This ensures the derived class relationship is recorded and managed within the tracking system.
            let derived_from_map = tracking_map.entry(derived_from).or_default();
            derived_from_map.insert(class_name, tracking_cls_names);
        }

        tracing::info!(context_name = %context_name, "Completed processing all classes for context");
    }

    /// Spawns a task to process non-responsive styles for a class.
    ///
    /// # Arguments
    /// - `class_name`: Name of the style class.
    /// - `is_important`: Indicates if the class has the `!important` flag.
    /// - `context_name`: Name of the current context.
    /// - `inherited_contexts`: List of contexts from which styles are inherited.
    /// - `styles_data`: Optional map of non-responsive style patterns.
    ///
    /// # Returns
    /// A handle to the spawned task which returns a vector of processed style names.
    fn process_non_responsive_styles(
        &self,
        class_name: String,
        is_important: bool,
        context_name: String,
        inherited_contexts: Vec<String>,
        styles_data: Option<IndexMap<String, IndexMap<String, String>>>,
    ) -> JoinHandle<Vec<String>> {
        tracing::debug!(
            class_name = %class_name,
            context_name = %context_name,
            is_important,
            "Spawning task to process non-responsive styles"
        );

        // Transform the context name to be used in alerts and clone the sender.
        let transformed_context_name = self.transform_context_name(&context_name);
        let sender = self.sender.clone();

        tokio::task::spawn_blocking(move || {
            let mut tracking_cls_names: Vec<String> = vec![];

            // If style patterns exist, process them.
            if let Some(styles_map) = styles_data {
                tracing::debug!(
                    class_name = %class_name,
                    num_styles = styles_map.len(),
                    "Processing non-responsive styles"
                );

                Self::process_patterns(
                    class_name.to_owned(),
                    is_important,
                    context_name,
                    None,
                    None,
                    inherited_contexts,
                    transformed_context_name,
                    &mut tracking_cls_names,
                    sender,
                    styles_map,
                );

                tracing::trace!(
                    class_name = %class_name,
                    processed_styles = tracking_cls_names.len(),
                    "Completed processing non-responsive styles"
                );
            }

            // Return the processed class names.
            tracking_cls_names
        })
    }

    /// Spawns a task to process responsive styles for a class.
    ///
    /// # Arguments
    /// - `class_name`: Name of the style class.
    /// - `is_important`: Indicates if the class has the `!important` flag.
    /// - `context_name`: Name of the current context.
    /// - `inherited_contexts`: List of contexts from which styles are inherited.
    /// - `styles_data`: Optional map of responsive style patterns grouped by breakpoints.
    ///
    /// # Returns
    /// A handle to the spawned task which returns a vector of processed style names.
    fn process_responsive_styles(
        &self,
        class_name: String,
        is_important: bool,
        context_name: String,
        inherited_contexts: Vec<String>,
        styles_data: Option<IndexMap<String, IndexMap<String, IndexMap<String, String>>>>,
    ) -> JoinHandle<Vec<String>> {
        // Transform the context name to be used in alerts and clone the sender.
        let transformed_context_name = self.transform_context_name(&context_name);
        let sender = self.sender.clone();

        tracing::info!(
            "Spawning task to process responsive styles for class '{}'. Context: '{}', Important: {}, Inherited contexts: {:?}",
            class_name, context_name, is_important, inherited_contexts
        );

        tokio::task::spawn_blocking(move || {
            let mut tracking_cls_names: Vec<String> = vec![];

            // If responsive style patterns exist, process them for each breakpoint.
            if let Some(styles_map) = styles_data {
                tracing::debug!(
                    "Processing {} responsive style breakpoints for class '{}'.",
                    styles_map.len(),
                    class_name
                );

                styles_map
                    .into_iter()
                    .for_each(
                        |(breakpoint_name, patterns_map)| match resolve_breakpoint_identifier(
                            &breakpoint_name,
                        ) {
                            Some(breakpoint) => {
                                tracing::debug!(
                                    "Processing breakpoint '{}' for class '{}'.",
                                    breakpoint_name,
                                    class_name
                                );

                                // Process patterns for the given breakpoint.
                                Self::process_patterns(
                                    class_name.to_owned(),
                                    is_important,
                                    context_name.to_owned(),
                                    Some(breakpoint),
                                    Some(breakpoint_name),
                                    inherited_contexts.to_vec(),
                                    transformed_context_name.to_owned(),
                                    &mut tracking_cls_names,
                                    sender.clone(),
                                    patterns_map,
                                );
                            }
                            None => {}
                        },
                    );

                tracing::trace!(
                    class_name = %class_name,
                    processed_styles = tracking_cls_names.len(),
                    "Completed processing responsive styles"
                );
            }

            // Return the processed class names.
            tracking_cls_names
        })
    }

    /// Processes style patterns for a class, generating and resolving properties and values
    /// while tracking utility class names for further processing.
    fn process_patterns(
        class_name: String,                         // Name of the class being processed.
        is_important: bool,                         // Whether the class is marked as important.
        context_name: String, // Name of the context to which the class belongs.
        breakpoint: Option<String>, // Breakpoint value, if applicable.
        breakpoint_name: Option<String>, // Name of the breakpoint, if applicable.
        inherited_contexts: Vec<String>, // Contexts inherited by the current context.
        transformed_context_name: String, // Transformed name of the current context, to be used in alerts.
        tracking_cls_names: &mut Vec<String>, // Vector to track generated utility class names.
        sender: broadcast::Sender<GaladrielAlerts>, // Channel to send warnings and alerts.
        styles_map: IndexMap<String, IndexMap<String, String>>, // Map of patterns and their properties.
    ) {
        tracing::debug!(
            "Processing patterns for class '{}' in context '{}'. Breakpoint: {:?}, Transformed context: '{}'.",
            class_name, context_name, breakpoint_name, transformed_context_name
        );

        // Iterate over each pattern and its associated properties.
        styles_map.iter().for_each(|(pattern_name, properties)| {
            tracing::debug!(
                "Processing pattern '{}' with {} properties for class '{}'.",
                pattern_name,
                properties.len(),
                class_name
            );

            // Iterate over properties and resolve each property-value pair.
            properties.iter().for_each(|(property, value)| {
                Self::resolve_property(
                    property,
                    value,
                    &class_name,
                    pattern_name,
                    is_important,
                    &context_name,
                    &breakpoint,
                    &breakpoint_name,
                    &inherited_contexts,
                    &transformed_context_name,
                    tracking_cls_names,
                    sender.clone(),
                );
            });
        });
    }

    /// Resolves a single property of a style pattern, checking for aliases and raising warnings
    /// if the alias cannot be resolved.
    fn resolve_property(
        property: &str,                             // The property to resolve.
        value: &str,                                // Value associated with the property.
        class_name: &str,                           // Name of the class being processed.
        pattern_name: &str, // Name of the pattern to which the property belongs.
        is_important: bool, // Whether the class is marked as important.
        context_name: &str, // Name of the context to which the class belongs.
        breakpoint: &Option<String>, // Breakpoint identifier, if applicable.
        breakpoint_name: &Option<String>, // Name of the breakpoint, if applicable.
        inherited_contexts: &Vec<String>, // Contexts inherited by the current context.
        transformed_context_name: &str, // Transformed name of the current context.
        tracking_cls_names: &mut Vec<String>, // Vector to track generated utility class names.
        sender: broadcast::Sender<GaladrielAlerts>, // Channel to send warnings and alerts.
    ) {
        // Attempt to resolve the property alias using inherited contexts.
        match resolve_alias_identifier(property, inherited_contexts) {
            Some(resolved_property) => {
                tracing::debug!(
                    "Resolved alias '{}' to '{}' for class '{}'.",
                    property,
                    resolved_property,
                    class_name
                );

                // If resolved, process the property value further.
                Self::resolve_value(
                    &resolved_property,
                    property,
                    value,
                    class_name,
                    pattern_name,
                    is_important,
                    context_name,
                    breakpoint,
                    breakpoint_name,
                    inherited_contexts,
                    transformed_context_name,
                    tracking_cls_names,
                    sender,
                );
            }
            None => {
                // Raise a warning if the alias cannot be resolved.
                let alias = property.trim_start_matches("nickname;");
                let pattern_name = pascalify(pattern_name);

                tracing::warn!(
                    "Unresolved alias '{}' of `{}` pattern for class '{}' in context '{}'.",
                    alias,
                    pattern_name,
                    class_name,
                    transformed_context_name
                );

                Self::raise_class_warning(
                    &format!(
                        "The `{}` alias of `{}` pattern in the `{}` class of the `{}` context was not identified in the current context or any of its extension contexts. As a result, the style corresponding to the `{}` alias was not created. Please verify the alias definition and its scope.",
                        alias, pattern_name, class_name, transformed_context_name, alias
                    ),
                    sender.clone()
                );
            }
        }
    }

    /// Resolves the value of a property, checking for variables and generating the utility class name
    /// if successful, or raising warnings if unresolved.
    fn resolve_value(
        resolved_property: &str,                    // Resolved property name.
        property: &str,                             // Original property name.
        value: &str,                                // Value associated with the property.
        class_name: &str,                           // Name of the class being processed.
        pattern_name: &str, // Name of the pattern to which the property belongs.
        is_important: bool, // Whether the class is marked as important.
        context_name: &str, // Name of the context to which the class belongs.
        breakpoint: &Option<String>, // Breakpoint value, if applicable.
        breakpoint_name: &Option<String>, // Name of the breakpoint, if applicable.
        inherited_contexts: &Vec<String>, // Contexts inherited by the current context.
        transformed_context_name: &str, // Transformed name of the current context, to be used in alerts.
        tracking_cls_names: &mut Vec<String>, // Vector to track generated utility class names.
        sender: broadcast::Sender<GaladrielAlerts>, // Channel to send warnings and alerts.
    ) {
        // Resolve variable values using the provided string and inherited contexts.
        match resolve_variable_from_str(value.to_owned(), true, inherited_contexts) {
            VariablesOption::Some(resolved_value) => {
                tracing::info!(
                    "Variable resolved successfully for property '{}'. Resolved value: '{}'.",
                    resolved_property,
                    resolved_value
                );

                // Generate a utility class name if the variable resolves successfully.
                Self::generate_utility_class_name(
                    resolved_property,
                    &resolved_value,
                    pattern_name,
                    is_important,
                    context_name,
                    breakpoint,
                    breakpoint_name,
                    tracking_cls_names,
                    sender.clone(),
                );
            }
            VariablesOption::Unresolved(unresolved_variable) => {
                // Raise a warning if the variable cannot be resolved.
                let property = property.trim_start_matches("nickname;");
                let property = camelify(property);
                let pattern_name = pascalify(pattern_name);

                tracing::warn!(
                    "Unresolved variable in property '{}' of `{}` pattern for class '{}' in context '{}'.",
                    property, pattern_name, class_name, transformed_context_name
                );

                Self::raise_class_warning(
                    &format!(
                        "The `{}` property of `{}` pattern in the `{}` class of the `{}` context contains unresolved variable: `{}`. The variable were not found in the current context or any of its extension contexts. As a result, the style corresponding to the `{}` property was not created. Please verify the variable definitions and their scope.",
                        property, pattern_name, class_name, context_name, unresolved_variable, property
                    ),
                    sender.clone()
                );
            }
        }
    }

    /// Generates a utility class name based on the resolved property and value.
    /// The generated class name is added to the Stylitron AST and tracked.
    ///
    /// # Arguments
    /// - `resolved_property`: The property that was resolved.
    /// - `resolved_value`: The value associated with the resolved property.
    /// - `pattern_name`: The pattern name where the property belongs.
    /// - `is_important`: Whether the property should be marked as `!important`.
    /// - `context_name`: The context in which the property is being resolved.
    /// - `breakpoint`: Optional breakpoint value.
    /// - `breakpoint_name`: Optional breakpoint name.
    /// - `tracking_cls_names`: A mutable reference to the vector tracking generated class names.
    /// - `sender`: A sender to communicate alerts.
    fn generate_utility_class_name(
        resolved_property: &str,
        resolved_value: &str,
        pattern_name: &str,
        is_important: bool,
        context_name: &str,
        breakpoint: &Option<String>,
        breakpoint_name: &Option<String>,
        tracking_cls_names: &mut Vec<String>,
        sender: broadcast::Sender<GaladrielAlerts>,
    ) {
        // Trim specific suffixes from the pattern name to normalize it.
        let pattern_name = pattern_name.trim_end_matches("stylesheet");
        // Generate the utility class name using the resolved data and breakpoint name.
        let utility_cls_name = generate_utility_class_name(
            &breakpoint_name,
            is_important,
            pattern_name,
            resolved_property,
            resolved_value,
        );

        tracing::info!(
            "Generated utility class name: '{}'. Applying to Stylitron AST.",
            utility_cls_name
        );

        // Apply the utility class to the Stylitron AST.
        Self::apply_utility_class_to_stylitron(
            &utility_cls_name,
            resolved_property,
            resolved_value,
            pattern_name,
            is_important,
            context_name,
            breakpoint,
            sender.clone(),
        );

        // Add the generated class name to the tracking list.
        tracking_cls_names.push(utility_cls_name.to_owned());

        tracing::debug!(
            "Utility class name '{}' added to tracking list. Total tracked classes: {}.",
            utility_cls_name,
            tracking_cls_names.len()
        );
    }

    /// Applies a utility class to the STYLITRON AST.
    ///
    /// This function determines the appropriate node in the STYLITRON AST
    /// (styles or responsive) based on whether a breakpoint is specified. It then
    /// updates the relevant node with the utility class and its associated styles.
    ///
    /// # Arguments
    /// - `utility_cls_name`: The name of the utility class to apply.
    /// - `resolved_property`: The CSS property being applied.
    /// - `resolved_value`: The resolved value of the CSS property.
    /// - `pattern_name`: The name of the pattern (pseudo class or element, or none) to which the style belongs.
    /// - `is_important`: Whether the CSS rule is marked as `!important`.
    /// - `context_name`: The name of the current context.
    /// - `breakpoint`: An optional breakpoint for responsive styles.
    /// - `sender`: A channel sender used to send alerts or notifications.
    fn apply_utility_class_to_stylitron(
        utility_cls_name: &str,
        resolved_property: &str,
        resolved_value: &str,
        pattern_name: &str,
        is_important: bool,
        context_name: &str,
        breakpoint: &Option<String>,
        sender: broadcast::Sender<GaladrielAlerts>,
    ) {
        // Determine which node in the Stylitron AST to access.
        let stylitron_node_name = match breakpoint {
            Some(_) => "responsive",
            None => "styles",
        };

        // Attempt to access the desired node in the Stylitron AST.
        let mut stylitron_data = match STYLITRON.get_mut(stylitron_node_name) {
            Some(data) => {
                tracing::debug!("Successfully accessed the styles section in STYLITRON AST.");
                data
            }
            None => {
                tracing::error!(
                    "Failed to access the styles section in STYLITRON AST for context: {}",
                    context_name
                );

                // If the `styles` section is not found, raise a critical error.
                let error = GaladrielError::raise_critical_other_error(
                    ErrorKind::AccessDeniedToStylitronAST,
                    "Failed to access the styles section in STYLITRON AST",
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

        // Process styles or responsive styles depending on the node type.
        match *stylitron_data {
            Stylitron::Styles(ref mut styles_definitions) => {
                tracing::debug!("Applying utility class to 'styles' node.");

                Self::apply_utility_class_to_styles_node(
                    pattern_name,
                    is_important,
                    resolved_property,
                    utility_cls_name,
                    resolved_value,
                    styles_definitions,
                );
            }
            Stylitron::ResponsiveStyles(ref mut styles_definitions) => {
                tracing::debug!("Applying utility class to 'responsive' node with breakpoint.");

                if let Some(breakpoint_value) = breakpoint {
                    tracing::debug!("Processing breakpoint: {}", breakpoint_value);

                    let breakpoints_styles = styles_definitions
                        .entry(breakpoint_value.to_owned())
                        .or_insert_with(generates_node_styles);

                    Self::apply_utility_class_to_styles_node(
                        pattern_name,
                        is_important,
                        resolved_property,
                        utility_cls_name,
                        resolved_value,
                        breakpoints_styles,
                    );
                }
            }
            _ => {}
        }

        tracing::info!("Utility class '{}' applied successfully.", utility_cls_name);
    }

    /// Adds a utility class and its styles to the given styles node.
    ///
    /// This function creates or updates nested structures within the styles node
    /// to store the utility class and its resolved styles.
    ///
    /// # Arguments
    /// - `pattern_name`: The name of the pattern (pseudo class or element, or none) to which the style belongs.
    /// - `is_important`: Whether the CSS rule is marked as `!important`.
    /// - `resolved_property`: The CSS property being applied.
    /// - `utility_cls_name`: The name of the utility class.
    /// - `resolved_value`: The resolved value of the CSS property.
    /// - `styles_definitions`: A mutable reference to the styles node to be updated.
    fn apply_utility_class_to_styles_node(
        pattern_name: &str,
        is_important: bool,
        resolved_property: &str,
        utility_cls_name: &str,
        resolved_value: &str,
        styles_definitions: &mut IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
        >,
    ) {
        // Determine the importance node based on the `is_important` flag.
        let importance_node = match is_important {
            true => "!important".to_string(),
            false => "_".to_string(),
        };

        tracing::debug!("Importance node determined as '{}'", importance_node);

        // Navigate through the styles definitions and insert the class.
        let patterns_styles = styles_definitions
            .entry(pattern_name.to_string())
            .or_default();

        let importance_styles = patterns_styles.entry(importance_node).or_default();
        let property_styles = importance_styles
            .entry(resolved_property.to_string())
            .or_default();

        property_styles
            .entry(utility_cls_name.to_string())
            .or_insert(resolved_value.to_string());

        tracing::info!(
            "Utility class '{}' successfully applied to the styles or responsive node for pattern '{}' and property '{}'.",
            utility_cls_name, pattern_name, resolved_property
        );
    }

    /// Raises a warning notification with the given message and sends it through the provided channel.
    ///
    /// # Parameters
    /// - `message`: A string slice containing the warning message to be displayed.
    /// - `sender`: An unbounded sender used to send the warning notification.
    fn raise_class_warning(message: &str, sender: broadcast::Sender<GaladrielAlerts>) {
        let notification = GaladrielAlerts::create_warning(Local::now(), message);

        // Attempt to send the warning notification.
        if let Err(err) = sender.send(notification) {
            tracing::error!("Failed to send warning notification: {:?}", err);
        }
    }

    /// Handles a task failure scenario by generating an error and sending an appropriate notification.
    ///
    /// # Parameters
    /// - `message`: A string slice containing the failure message to be used in the error.
    fn handle_classes_task_failure(&self, message: &str) {
        let sender = self.sender.clone();

        let error = GaladrielError::raise_general_other_error(
            ErrorKind::TaskFailure,
            message,
            ErrorAction::Notify,
        );

        let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

        if let Err(err) = sender.send(notification) {
            tracing::error!("Failed to send warning notification: {:?}", err);
        }
    }
}

#[cfg(test)]
mod classes_tests {
    use indexmap::IndexMap;
    use nenyr::types::{ast::NenyrAst, central::CentralContext, class::NenyrStyleClass};
    use tokio::sync::broadcast;

    use crate::{asts::STYLITRON, crealion::Crealion, types::Stylitron};

    fn mock_breakpoints() {
        let map = IndexMap::from([(
            "mobile-first".to_string(),
            IndexMap::from([("mobMd".to_string(), "min-width:740px".to_string())]),
        )]);

        STYLITRON.insert("breakpoints".to_string(), Stylitron::Breakpoints(map));
    }

    fn mock_classes() -> IndexMap<String, NenyrStyleClass> {
        IndexMap::from([
            (
                "thisJustAnotherClass".to_string(),
                NenyrStyleClass {
                    class_name: "thisJustAnotherClass".to_string(),
                    deriving_from: Some("oneExtraClass".to_string()),
                    is_important: Some(true),
                    style_patterns: Some(IndexMap::from([(
                        "_stylesheet".to_string(),
                        IndexMap::from([("background-color".to_string(), "#0000FF".to_string())]),
                    )])),
                    responsive_patterns: Some(IndexMap::from([(
                        "mobMd".to_string(),
                        IndexMap::from([(
                            "_stylesheet".to_string(),
                            IndexMap::from([(
                                "background-color".to_string(),
                                "#FF0000".to_string(),
                            )]),
                        )]),
                    )])),
                },
            ),
            (
                "oneExtraClass".to_string(),
                NenyrStyleClass {
                    class_name: "oneExtraClass".to_string(),
                    deriving_from: None,
                    is_important: None,
                    style_patterns: Some(IndexMap::from([(
                        ":hover".to_string(),
                        IndexMap::from([("background-color".to_string(), "#FFFF00".to_string())]),
                    )])),
                    responsive_patterns: Some(IndexMap::from([(
                        "mobMd".to_string(),
                        IndexMap::from([(
                            ":hover".to_string(),
                            IndexMap::from([(
                                "background-color".to_string(),
                                "#00FFFF".to_string(),
                            )]),
                        )]),
                    )])),
                },
            ),
        ])
    }

    #[tokio::test]
    async fn classes_exists_in_ast() {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        mock_breakpoints();

        let (sender, _) = broadcast::channel(10);
        let mut tracking_map: IndexMap<String, IndexMap<String, Vec<String>>> = IndexMap::new();

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let _ = crealion
            .process_classes(
                "firstClassContextName".to_string(),
                vec!["firstClassContextName".to_string()],
                mock_classes(),
                &mut tracking_map,
            )
            .await;

        let first_non_responsive_cls =
            STYLITRON
                .get("styles")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Styles(ref styles_defs) => {
                        styles_defs.get("_").and_then(|pattern_styles| {
                            pattern_styles
                                .get("!important")
                                .and_then(|importance_styles| {
                                    importance_styles.get("background-color").and_then(
                                        |property_styles| {
                                            property_styles.get_index(0).and_then(
                                                |(utility_name, _)| Some(utility_name.to_owned()),
                                            )
                                        },
                                    )
                                })
                        })
                    }
                    _ => None,
                });

        assert!(first_non_responsive_cls.is_some());
        assert_eq!(
            first_non_responsive_cls.unwrap(),
            "\\!bgd-clr-NmXB".to_string()
        );

        let first_responsive_cls =
            STYLITRON
                .get("responsive")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::ResponsiveStyles(ref styles_defs) => styles_defs
                        .get("min-width:740px")
                        .and_then(|breakpoint_styles| {
                            breakpoint_styles.get("_").and_then(|pattern_styles| {
                                pattern_styles
                                    .get("!important")
                                    .and_then(|importance_styles| {
                                        importance_styles.get("background-color").and_then(
                                            |property_styles| {
                                                property_styles.get_index(0).and_then(
                                                    |(utility_name, _)| {
                                                        Some(utility_name.to_owned())
                                                    },
                                                )
                                            },
                                        )
                                    })
                            })
                        }),
                    _ => None,
                });

        assert!(first_responsive_cls.is_some());
        assert_eq!(
            first_responsive_cls.unwrap(),
            "mMd\\.\\!bgd-clr-a1Ib".to_string()
        );

        let second_non_responsive_cls =
            STYLITRON
                .get("styles")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Styles(ref styles_defs) => {
                        styles_defs.get(":hover").and_then(|pattern_styles| {
                            pattern_styles.get("_").and_then(|importance_styles| {
                                importance_styles.get("background-color").and_then(
                                    |property_styles| {
                                        property_styles.get_index(0).and_then(
                                            |(utility_name, _)| Some(utility_name.to_owned()),
                                        )
                                    },
                                )
                            })
                        })
                    }
                    _ => None,
                });

        assert!(second_non_responsive_cls.is_some());
        assert_eq!(
            second_non_responsive_cls.unwrap(),
            "hvr\\.bgd-clr-DA0P".to_string()
        );

        let second_responsive_cls =
            STYLITRON
                .get("responsive")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::ResponsiveStyles(ref styles_defs) => styles_defs
                        .get("min-width:740px")
                        .and_then(|breakpoint_styles| {
                            breakpoint_styles.get(":hover").and_then(|pattern_styles| {
                                pattern_styles.get("_").and_then(|importance_styles| {
                                    importance_styles.get("background-color").and_then(
                                        |property_styles| {
                                            property_styles.get_index(0).and_then(
                                                |(utility_name, _)| Some(utility_name.to_owned()),
                                            )
                                        },
                                    )
                                })
                            })
                        }),
                    _ => None,
                });

        assert!(second_responsive_cls.is_some());
        assert_eq!(
            second_responsive_cls.unwrap(),
            "mMd\\.hvr\\.bgd-clr-fWgf".to_string()
        );
    }
}
