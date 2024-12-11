use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use nenyr::types::{
    ast::NenyrAst, central::CentralContext, layout::LayoutContext, module::ModuleContext,
};
use tokio::{sync::broadcast, task::JoinError};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    GaladrielResult,
};

mod aliases;
mod animations;
mod breakpoints;
mod classes;
mod classinator;
mod gatekeeper;
mod imports;
mod intaker;
mod processors;
mod themes;
mod typefaces;
mod utils;
mod variables;

pub const CENTRAL_CONTEXT_NAME: &str = "gCtxCen_8Xq4ZJ";

#[derive(Clone, PartialEq, Debug)]
pub enum CrealionContextType {
    Central,
    Layout,
    Module,
}

type CrealionResult = GaladrielResult<(CrealionContextType, Option<Vec<String>>)>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Crealion {
    sender: broadcast::Sender<GaladrielAlerts>,
    central_context_identifier: String,
    parsed_ast: NenyrAst,
    path: String,
}

impl Crealion {
    /// Creates a new instance of `Crealion` with the given parameters.
    ///
    /// # Parameters
    /// - `sender`: A channel for sending `GaladrielAlerts`.
    /// - `parsed_ast`: The parsed Abstract Syntax Tree (AST) of type `NenyrAst`.
    /// - `path`: The path related to the AST.
    pub fn new(
        sender: broadcast::Sender<GaladrielAlerts>,
        parsed_ast: NenyrAst,
        path: String,
    ) -> Self {
        Self {
            central_context_identifier: CENTRAL_CONTEXT_NAME.to_string(),
            parsed_ast,
            path,
            sender,
        }
    }

    /// Creates the appropriate collector based on the parsed AST type.
    ///
    /// This method identifies the context type (`CentralContext`, `LayoutContext`, or `ModuleContext`)
    /// and initializes the corresponding collector.
    ///
    /// # Returns
    /// A `CrealionResult` which signifies the outcome of the creation process.
    pub async fn create(&mut self) -> CrealionResult {
        match self.parsed_ast.clone() {
            NenyrAst::CentralContext(context) => self.init_central_collector(&context).await,
            NenyrAst::LayoutContext(context) => self.init_layout_collector(&context).await,
            NenyrAst::ModuleContext(context) => self.init_module_collector(&context).await,
        }
    }

    /// Initializes the collector for a central context.
    ///
    /// Processes all aspects of a `CentralContext` including variables, themes, breakpoints,
    /// aliases, typefaces, imports, animations, and classes, and applies them to their respective handlers.
    ///
    /// # Parameters
    /// - `context`: A reference to the `CentralContext` to initialize.
    ///
    /// # Returns
    /// A `CrealionResult` indicating the result of the operation.
    async fn init_central_collector(&mut self, context: &CentralContext) -> CrealionResult {
        tracing::debug!("Initializing Central Context Collector");

        // Context name derived from the unique identifier.
        let context_name = self.central_context_identifier.to_owned();

        // Extract variables or use a default empty map.
        let variables_data = self.get_value(
            context.variables.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted variables: {:?}", variables_data);

        // Extract themes (light and dark schemas) or use default values.
        let (light_data, dark_data) = self.get_value(
            context
                .themes
                .as_ref()
                .map(|v| (v.light_schema.to_owned(), v.dark_schema.to_owned())),
            (None, None),
        );

        tracing::debug!(
            "Extracted themes - Light: {:?}, Dark: {:?}",
            light_data,
            dark_data
        );

        // Extract breakpoints for mobile and desktop or use defaults.
        let (mobile_data, desktop_data) = self.get_value(
            context
                .breakpoints
                .as_ref()
                .map(|v| (v.mobile_first.to_owned(), v.desktop_first.to_owned())),
            (None, None),
        );

        tracing::debug!(
            "Extracted breakpoints - Mobile: {:?}, Desktop: {:?}",
            mobile_data,
            desktop_data
        );

        // Extract aliases or use a default empty map.
        let aliases_data = self.get_value(
            context.aliases.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted aliases: {:?}", aliases_data);

        // Extract typefaces or use a default empty map.
        let typefaces_data = self.get_value(
            context.typefaces.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted typefaces: {:?}", typefaces_data);

        // Extract imports or use a default empty map.
        let imports_data = self.get_value(
            context.imports.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted imports: {:?}", imports_data);

        // Process various aspects of the central context concurrently.
        join_all(vec![
            self.process_variables(context_name.to_owned(), variables_data),
            self.process_themes(context_name.to_owned(), light_data, dark_data),
            self.process_breakpoints(mobile_data, desktop_data),
            self.apply_aliases_to_stylitron(context_name.to_owned(), aliases_data),
            self.apply_typefaces_to_stylitron(typefaces_data),
            self.apply_imports_to_stylitron(imports_data),
        ])
        .await
        .iter()
        .for_each(|future_result| match future_result {
            Err(join_error) => {
                tracing::error!("Error in concurrent processing: {:?}", join_error);
                self.handle_join_error(join_error)
            }
            Ok(()) => {
                tracing::debug!("Concurrent task completed successfully");
            }
        });

        // Maintain a list of inherited contexts for animation and class processing.
        let inherited_contexts = vec![context_name.to_owned()];

        tracing::debug!(
            "Inherited contexts prepared for animations and classes: {:?}",
            inherited_contexts
        );

        // Extract animations or use a default empty map.
        let animations_data = self.get_value(
            context.animations.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted animations: {:?}", animations_data);

        // Process animations with the inherited contexts.
        self.process_animations(&context_name, &inherited_contexts, animations_data);

        // Extract classes or use a default empty map.
        let classes_data = self.get_value(
            context.classes.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted classes: {:?}", classes_data);

        // Initialize a hierarchical tracking map using `IndexMap`.
        //
        // The outer map uses `String` keys to represent derived class names. Each derived class
        // name maps to an inner `IndexMap`, where keys are class names (`String`) and values are vectors
        // of strings (`Vec<String>`), representing the tracking relationships for CSS utility classes.
        //
        // The initial map includes a default context `"_"`, which is pre-populated with an empty `IndexMap`
        // to serve as a fallback for Nenyr classes that does not derive any other class.
        let mut tracking_map: IndexMap<String, IndexMap<String, Vec<String>>> =
            IndexMap::from([("_".to_string(), IndexMap::new())]);

        // Process classes for the central context.
        self.process_classes(
            context_name.to_owned(),
            inherited_contexts,
            classes_data,
            &mut tracking_map,
        )
        .await;

        // Integrates the processed tracking data into the `CLASSINATOR` system,
        // which manages the mapping between Nenyr classes and their corresponding CSS utility classes,
        // including inheritance.
        self.apply_tracking_map_to_classinator(
            context_name,
            None,
            CrealionContextType::Central,
            tracking_map,
        );

        tracing::info!("Central Context Collector initialization completed");

        Ok((CrealionContextType::Central, None))
    }

    /// Initializes the layout context by processing variables, themes, aliases, animations, and classes.
    ///
    /// # Arguments
    /// * `context` - A reference to the layout context containing the configuration details.
    async fn init_layout_collector(&mut self, context: &LayoutContext) -> CrealionResult {
        // Extract the layout name for use as the context identifier.
        let context_name = context.layout_name.to_owned();

        // Ensures that the current context is not being used by another context.
        // Context names must be globally unique to prevent conflicts.
        self.validates_context_name(context_name.to_owned(), self.path.to_owned())?;

        tracing::debug!(
            "Initializing Layout Context Collector for context: {}",
            context_name
        );

        // Retrieve the variable definitions from the layout context, if any.
        let variables_data = self.get_value(
            context.variables.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted variables: {:?}", variables_data);

        // Retrieve the light and dark theme schemas from the context, if defined.
        let (light_data, dark_data) = self.get_value(
            context
                .themes
                .as_ref()
                .map(|v| (v.light_schema.to_owned(), v.dark_schema.to_owned())),
            (None, None),
        );

        tracing::debug!(
            "Extracted themes - Light: {:?}, Dark: {:?}",
            light_data,
            dark_data
        );

        // Retrieve alias definitions from the layout context, if provided.
        let aliases_data = self.get_value(
            context.aliases.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted aliases: {:?}", aliases_data);

        // Process variables, themes, and aliases concurrently.
        join_all(vec![
            self.process_variables(context_name.to_owned(), variables_data),
            self.process_themes(context_name.to_owned(), light_data, dark_data),
            self.apply_aliases_to_stylitron(context_name.to_owned(), aliases_data),
        ])
        .await
        .iter()
        .for_each(|future_result| match future_result {
            Err(join_error) => {
                tracing::error!("Error in concurrent processing: {:?}", join_error);
                self.handle_join_error(join_error)
            }
            Ok(()) => {
                tracing::debug!("Concurrent task completed successfully");
            }
        });

        // Create a list of inherited contexts for use in subsequent processing, including this context, and the central context.
        let inherited_contexts = vec![
            context_name.to_owned(),
            self.central_context_identifier.to_owned(),
        ];

        tracing::debug!(
            "Inherited contexts prepared for animations and classes: {:?}",
            inherited_contexts
        );

        // Retrieve animation definitions from the context, if available.
        let animations_data = self.get_value(
            context.animations.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted animations: {:?}", animations_data);

        // Process animations and integrate them with the inherited contexts.
        self.process_animations(&context_name, &inherited_contexts, animations_data);

        // Retrieve class definitions from the layout context, if any.
        let classes_data = self.get_value(
            context.classes.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted classes: {:?}", classes_data);

        // Initialize a hierarchical tracking map using `IndexMap`.
        //
        // The outer map uses `String` keys to represent derived class names. Each derived class
        // name maps to an inner `IndexMap`, where keys are class names (`String`) and values are vectors
        // of strings (`Vec<String>`), representing the tracking relationships for CSS utility classes.
        //
        // The initial map includes a default context `"_"`, which is pre-populated with an empty `IndexMap`
        // to serve as a fallback for Nenyr classes that does not derive any other class.
        let mut tracking_map: IndexMap<String, IndexMap<String, Vec<String>>> =
            IndexMap::from([("_".to_string(), IndexMap::new())]);

        // Process class definitions, applying the layout-specific settings.
        self.process_classes(
            context_name.to_owned(),
            inherited_contexts,
            classes_data,
            &mut tracking_map,
        )
        .await;

        // Integrates the processed tracking data into the `CLASSINATOR` system,
        // which manages the mapping between Nenyr classes and their corresponding CSS utility classes,
        // including inheritance.
        self.apply_tracking_map_to_classinator(
            context_name.to_owned(),
            None,
            CrealionContextType::Layout,
            tracking_map,
        );

        tracing::info!(
            "Layout Context Collector initialization completed for: {}",
            context_name
        );

        // Collects the paths related to the current layout context.
        // If a relationship exists, Galadriel CSS reprocesses the modules associated with the current layout context.
        // This ensures that all related contexts are updated with the latest styles, maintaining consistency and synchronization.
        let related_contexts = self.retrieve_module_layout_relationship(&context_name);

        Ok((CrealionContextType::Layout, related_contexts))
    }

    /// Initializes the module context by processing variables, aliases, animations, and classes.
    ///
    /// # Arguments
    /// * `context` - A reference to the module context containing module-specific configurations.
    pub async fn init_module_collector(&mut self, context: &ModuleContext) -> CrealionResult {
        // Extract the module name to use as the context identifier.
        let context_name = context.module_name.to_owned();
        // Extract the name of the context from which this module extends, or use an empty string if none.
        let extended_from = context.extending_from.to_owned().unwrap_or("_".to_string());

        // Ensures that the current context is not being used by another context.
        // Context names must be globally unique to prevent conflicts.
        self.validates_context_name(context_name.to_string(), self.path.to_string())?;

        tracing::debug!(
            "Initializing Module Context Collector for context: {}",
            context_name
        );

        // Retrieve variable definitions from the module context, if any.
        let variables_data = self.get_value(
            context.variables.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted variables: {:?}", variables_data);

        // Retrieve alias definitions from the module context, if provided.
        let aliases_data = self.get_value(
            context.aliases.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted aliases: {:?}", aliases_data);

        // Process variables and aliases concurrently.
        join_all(vec![
            self.process_variables(context_name.to_owned(), variables_data),
            self.apply_aliases_to_stylitron(context_name.to_owned(), aliases_data),
        ])
        .await
        .iter()
        .for_each(|future_result| match future_result {
            Err(join_error) => {
                tracing::error!("Error in concurrent processing: {:?}", join_error);
                self.handle_join_error(join_error)
            }
            Ok(()) => {
                tracing::debug!("Concurrent task completed successfully");
            }
        });

        // Build the list of inherited contexts, including this context, its parent, and the central context.
        let mut inherited_contexts = vec![
            context_name.to_owned(),
            extended_from.to_owned(),
            self.central_context_identifier.to_owned(),
        ];

        // Remove empty context names from the list to ensure only valid entries remain.
        inherited_contexts.retain(|v| v != "_");

        tracing::debug!(
            "Inherited contexts prepared for animations and classes: {:?}",
            inherited_contexts
        );

        // Retrieve animation definitions from the module context, if available.
        let animations_data = self.get_value(
            context.animations.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted animations: {:?}", animations_data);

        // Process animations and integrate them with the inherited contexts.
        self.process_animations(&context_name, &inherited_contexts, animations_data);

        // Retrieve class definitions from the module context, if any.
        let classes_data = self.get_value(
            context.classes.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        tracing::debug!("Extracted classes: {:?}", classes_data);

        // Initialize a hierarchical tracking map using `IndexMap`.
        //
        // The outer map uses `String` keys to represent derived class names. Each derived class
        // name maps to an inner `IndexMap`, where keys are class names (`String`) and values are vectors
        // of strings (`Vec<String>`), representing the tracking relationships for CSS utility classes.
        //
        // The initial map includes a default context `"_"`, which is pre-populated with an empty `IndexMap`
        // to serve as a fallback for Nenyr classes that does not derive any other class.
        let mut tracking_map: IndexMap<String, IndexMap<String, Vec<String>>> =
            IndexMap::from([("_".to_string(), IndexMap::new())]);

        // Process class definitions, integrating any extended context and applying module-specific settings.
        self.process_classes(
            context_name.to_owned(),
            inherited_contexts,
            classes_data,
            &mut tracking_map,
        )
        .await;

        // Integrates the processed tracking data into the `CLASSINATOR` system,
        // which manages the mapping between Nenyr classes and their corresponding CSS utility classes,
        // including inheritance.
        self.apply_tracking_map_to_classinator(
            context_name.to_owned(),
            context.extending_from.to_owned(),
            CrealionContextType::Module,
            tracking_map,
        );

        tracing::info!(
            "Module Context Collector initialization completed for: {}",
            context_name
        );

        // Registers the relationship between the extended layout (specified by its name) and the current file path.
        self.register_module_layout_relationship(extended_from, self.path.to_owned());

        Ok((CrealionContextType::Module, None))
    }

    /// Transforms the given context name.
    ///
    /// If the provided context name matches the central context identifier,
    /// it is transformed into the string `"central"`. Otherwise, the original
    /// context name is returned.
    ///
    /// # Arguments
    /// * `context_name` - A string slice representing the name of the context to transform.
    ///
    /// # Returns
    /// A `String` containing the transformed context name.
    pub fn transform_context_name(&self, context_name: &str) -> String {
        if context_name == self.central_context_identifier {
            "central".to_string()
        } else {
            context_name.to_string()
        }
    }

    /// Retrieves a value from an `Option` or returns a default if the option is `None`.
    ///
    /// # Arguments
    /// * `opt` - An `Option` that may contain a value of type `T`.
    /// * `default` - A default value of type `T` to use if `opt` is `None`.
    ///
    /// # Returns
    /// The contained value if `opt` is `Some`, otherwise the `default` value.
    ///
    /// # Notes
    /// Logs a warning if the option is `None` and the default value is used.
    fn get_value<T>(&self, opt: Option<T>, default: T) -> T {
        opt.unwrap_or_else(|| {
            tracing::warn!("Expected value but found None, using default.");
            default
        })
    }

    /// Handles errors resulting from joining asynchronous tasks.
    ///
    /// This function raises a `GaladrielError` for a task failure, creates a
    /// notification using `GaladrielAlerts`, and attempts to send it via the
    /// internal sender.
    ///
    /// # Arguments
    /// * `join_error` - A reference to a `JoinError` representing the error that occurred.
    fn handle_join_error(&self, join_error: &JoinError) {
        let sender = self.sender.clone();

        let error = GaladrielError::raise_general_other_error(
            ErrorKind::TaskFailure,
            &join_error.to_string(),
            ErrorAction::Notify,
        );

        let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

        if let Err(err) = sender.send(notification) {
            tracing::error!(
                "Failed to send notification for join error: {}. Send error: {}",
                join_error,
                err
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::NenyrParser;
    use tokio::sync::broadcast;

    use crate::{
        asts::STYLITRON, types::Stylitron, utils::generates_node_styles::generates_node_styles,
    };

    use super::{Crealion, CrealionContextType};

    fn reset_stylitron() {
        STYLITRON.clear();

        STYLITRON.insert("imports".to_string(), Stylitron::Imports(IndexMap::new()));
        STYLITRON.insert("aliases".to_string(), Stylitron::Aliases(IndexMap::new()));
        STYLITRON.insert(
            "breakpoints".to_string(),
            Stylitron::Breakpoints(IndexMap::new()),
        );
        STYLITRON.insert(
            "typefaces".to_string(),
            Stylitron::Typefaces(IndexMap::new()),
        );
        STYLITRON.insert(
            "variables".to_string(),
            Stylitron::Variables(IndexMap::new()),
        );
        STYLITRON.insert("themes".to_string(), Stylitron::Themes(IndexMap::new()));
        STYLITRON.insert(
            "animations".to_string(),
            Stylitron::Animation(IndexMap::new()),
        );
        STYLITRON.insert(
            "styles".to_string(),
            Stylitron::Styles(generates_node_styles()),
        );
        STYLITRON.insert(
            "responsive".to_string(),
            Stylitron::ResponsiveStyles(IndexMap::new()),
        );
    }

    #[tokio::test]
    async fn central_context_created_with_success() {
        tokio::time::sleep(tokio::time::Duration::from_secs(25)).await;

        match std::fs::read_to_string("src/crealion/mocks/central.nyr") {
            Ok(raw_nenyr) => {
                let mut parser = NenyrParser::new();

                match parser.parse(raw_nenyr, "src/crealion/mocks/central.nyr".to_string()) {
                    Ok(parsed_ast) => {
                        reset_stylitron();

                        let (sender, _) = broadcast::channel(10);

                        let mut crealion = Crealion::new(
                            sender,
                            parsed_ast,
                            "src/crealion/mocks/central.nyr".to_string(),
                        );

                        let result = crealion.create().await;

                        assert!(result.is_ok());
                        assert_eq!(result.unwrap(), (CrealionContextType::Central, None));
                        assert_eq!(
                            STYLITRON.get("breakpoints").map(|v| format!("{:?}", &*v)),
                            Some("Breakpoints({\"mobile-first\": {\"onMobXs\": \"min-width:360px\"}, \"desktop-first\": {\"onDeskSmall\": \"max-width:1024px\"}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("typefaces").map(|v| format!("{:?}", &*v)),
                            Some("Typefaces({\"roseMartin\": \"./typefaces/rosemartin.regular.otf\"})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("imports").map(|v| format!("{:?}", &*v)),
                            Some("Imports({\"https://fonts.googleapis.com/css2?family=Matemasie&display=swap\": ()})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("themes").map(|v| format!("{:?}", &*v)),
                            Some("Themes({\"gCtxCen_8Xq4ZJ\": {\"light\": {\"primaryColor\": [\"--gNKGUE7AAmy\", \"#FFFFFF\"]}, \"dark\": {\"primaryColor\": [\"--gNKGUE7AAmy\", \"#1E1E1E\"]}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("aliases").map(|v| format!("{:?}", &*v)),
                            Some("Aliases({\"gCtxCen_8Xq4ZJ\": {\"bgd\": \"background\", \"dp\": \"display\", \"transf\": \"transform\", \"pdg\": \"padding\", \"wd\": \"width\", \"hgt\": \"height\", \"flexDir\": \"flex-direction\"}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("variables").map(|v| format!("{:?}", &*v)),
                            Some("Variables({\"gCtxCen_8Xq4ZJ\": {\"myColor\": [\"--gW1yAqTMgoH\", \"#FF6677\"]}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("animations").map(|v| format!("{:?}", &*v)),
                            Some("Animation({\"gCtxCen_8Xq4ZJ\": {\"slideScale\": {\"giq8HPC3JaYa\": {\"20%\": {\"transform\": \"translateX(10%) scale(1.1)\"}, \"40%,60%\": {\"transform\": \"translateX(30%) scale(1.2)\"}, \"80%\": {\"transform\": \"translateX(50%) scale(0.9)\"}, \"100%\": {\"transform\": \"translateX(0) scale(1)\"}}}, \"borderFlash\": {\"gpKLT8POASvU\": {\"10%\": {\"border-color\": \"var(--gW1yAqTMgoH)\", \"border-width\": \"1px\"}, \"30%,50%,70%\": {\"border-color\": \"red\", \"border-width\": \"3px\"}, \"90%\": {\"border-color\": \"green\", \"border-width\": \"2px\"}, \"100%\": {\"border-color\": \"var(--gW1yAqTMgoH)\", \"border-width\": \"1px\"}}}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("styles").map(|v| format!("{:?}", &*v)),
                            Some("Styles({\"_\": {\"!important\": {\"background\": {\"\\\\!bgd-kobF\": \"var(--gNKGUE7AAmy)\"}, \"color\": {\"\\\\!clr-UZ6Q\": \"var(--gW1yAqTMgoH)\"}, \"padding\": {\"\\\\!pdg-3KtM\": \"10px\"}, \"display\": {\"\\\\!dpy-5TuI\": \"flex\"}, \"align-items\": {\"\\\\!lgn-tms-sLJ6\": \"center\"}}, \"_\": {}}, \"::after\": {\"!important\": {\"content\": {\"\\\\!ftr\\\\.ctt-WT3W\": \"' '\"}, \"display\": {\"\\\\!ftr\\\\.dpy-S4vd\": \"block\"}, \"width\": {\"\\\\!ftr\\\\.wth-YYq9\": \"100%\"}, \"height\": {\"\\\\!ftr\\\\.hht-9X8O\": \"2px\"}, \"background\": {\"\\\\!ftr\\\\.bgd-kobF\": \"var(--gNKGUE7AAmy)\"}}, \"_\": {}}, \"::before\": {\"!important\": {}, \"_\": {}}, \"::first-line\": {\"!important\": {}, \"_\": {}}, \"::first-letter\": {\"!important\": {}, \"_\": {}}, \":hover\": {\"!important\": {\"color\": {\"\\\\!hvr\\\\.clr-UZ6Q\": \"var(--gW1yAqTMgoH)\"}, \"border\": {\"\\\\!hvr\\\\.bdr-Csem\": \"2px solid var(--gNKGUE7AAmy)\"}, \"animation-name\": {\"\\\\!hvr\\\\.ntn-nm-Y1vH\": \"gpKLT8POASvU\"}}, \"_\": {}}, \":active\": {\"!important\": {}, \"_\": {}}, \":focus\": {\"!important\": {}, \"_\": {}}, \":first-child\": {\"!important\": {}, \"_\": {}}, \":last-child\": {\"!important\": {}, \"_\": {}}, \":first-of-type\": {\"!important\": {}, \"_\": {}}, \":last-of-type\": {\"!important\": {}, \"_\": {}}, \":only-child\": {\"!important\": {}, \"_\": {}}, \":only-of-type\": {\"!important\": {}, \"_\": {}}, \":target\": {\"!important\": {}, \"_\": {}}, \":visited\": {\"!important\": {}, \"_\": {}}, \":checked\": {\"!important\": {}, \"_\": {}}, \":disabled\": {\"!important\": {}, \"_\": {}}, \":enabled\": {\"!important\": {}, \"_\": {}}, \":read-only\": {\"!important\": {}, \"_\": {}}, \":read-write\": {\"!important\": {}, \"_\": {}}, \":placeholder-shown\": {\"!important\": {}, \"_\": {}}, \":valid\": {\"!important\": {}, \"_\": {}}, \":invalid\": {\"!important\": {}, \"_\": {}}, \":required\": {\"!important\": {}, \"_\": {}}, \":optional\": {\"!important\": {}, \"_\": {}}, \":fullscreen\": {\"!important\": {}, \"_\": {}}, \":focus-within\": {\"!important\": {}, \"_\": {}}, \":out-of-range\": {\"!important\": {}, \"_\": {}}, \":root\": {\"!important\": {}, \"_\": {}}, \":empty\": {\"!important\": {}, \"_\": {}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("responsive").map(|v| format!("{:?}", &*v)),
                            Some("ResponsiveStyles({\"min-width:360px\": {\"_\": {\"!important\": {\"display\": {\"nbs\\\\.\\\\!dpy-S4vd\": \"block\"}, \"flex-direction\": {\"nbs\\\\.\\\\!flx-dcn-w5ZN\": \"column\"}, \"padding\": {\"nbs\\\\.\\\\!pdg-3JDd\": \"15px\"}}, \"_\": {}}, \"::after\": {\"!important\": {}, \"_\": {}}, \"::before\": {\"!important\": {}, \"_\": {}}, \"::first-line\": {\"!important\": {}, \"_\": {}}, \"::first-letter\": {\"!important\": {}, \"_\": {}}, \":hover\": {\"!important\": {}, \"_\": {}}, \":active\": {\"!important\": {}, \"_\": {}}, \":focus\": {\"!important\": {}, \"_\": {}}, \":first-child\": {\"!important\": {}, \"_\": {}}, \":last-child\": {\"!important\": {}, \"_\": {}}, \":first-of-type\": {\"!important\": {}, \"_\": {}}, \":last-of-type\": {\"!important\": {}, \"_\": {}}, \":only-child\": {\"!important\": {}, \"_\": {}}, \":only-of-type\": {\"!important\": {}, \"_\": {}}, \":target\": {\"!important\": {}, \"_\": {}}, \":visited\": {\"!important\": {}, \"_\": {}}, \":checked\": {\"!important\": {}, \"_\": {}}, \":disabled\": {\"!important\": {}, \"_\": {}}, \":enabled\": {\"!important\": {}, \"_\": {}}, \":read-only\": {\"!important\": {}, \"_\": {}}, \":read-write\": {\"!important\": {}, \"_\": {}}, \":placeholder-shown\": {\"!important\": {}, \"_\": {}}, \":valid\": {\"!important\": {}, \"_\": {}}, \":invalid\": {\"!important\": {}, \"_\": {}}, \":required\": {\"!important\": {}, \"_\": {}}, \":optional\": {\"!important\": {}, \"_\": {}}, \":fullscreen\": {\"!important\": {}, \"_\": {}}, \":focus-within\": {\"!important\": {}, \"_\": {}}, \":out-of-range\": {\"!important\": {}, \"_\": {}}, \":root\": {\"!important\": {}, \"_\": {}}, \":empty\": {\"!important\": {}, \"_\": {}}}, \"max-width:1024px\": {\"_\": {\"!important\": {}, \"_\": {}}, \"::after\": {\"!important\": {}, \"_\": {}}, \"::before\": {\"!important\": {}, \"_\": {}}, \"::first-line\": {\"!important\": {}, \"_\": {}}, \"::first-letter\": {\"!important\": {}, \"_\": {}}, \":hover\": {\"!important\": {\"background\": {\"nSl\\\\.\\\\!hvr\\\\.bgd-kobF\": \"var(--gNKGUE7AAmy)\"}, \"padding\": {\"nSl\\\\.\\\\!hvr\\\\.pdg-3Kvn\": \"20px\"}}, \"_\": {}}, \":active\": {\"!important\": {}, \"_\": {}}, \":focus\": {\"!important\": {}, \"_\": {}}, \":first-child\": {\"!important\": {}, \"_\": {}}, \":last-child\": {\"!important\": {}, \"_\": {}}, \":first-of-type\": {\"!important\": {}, \"_\": {}}, \":last-of-type\": {\"!important\": {}, \"_\": {}}, \":only-child\": {\"!important\": {}, \"_\": {}}, \":only-of-type\": {\"!important\": {}, \"_\": {}}, \":target\": {\"!important\": {}, \"_\": {}}, \":visited\": {\"!important\": {}, \"_\": {}}, \":checked\": {\"!important\": {}, \"_\": {}}, \":disabled\": {\"!important\": {}, \"_\": {}}, \":enabled\": {\"!important\": {}, \"_\": {}}, \":read-only\": {\"!important\": {}, \"_\": {}}, \":read-write\": {\"!important\": {}, \"_\": {}}, \":placeholder-shown\": {\"!important\": {}, \"_\": {}}, \":valid\": {\"!important\": {}, \"_\": {}}, \":invalid\": {\"!important\": {}, \"_\": {}}, \":required\": {\"!important\": {}, \"_\": {}}, \":optional\": {\"!important\": {}, \"_\": {}}, \":fullscreen\": {\"!important\": {}, \"_\": {}}, \":focus-within\": {\"!important\": {}, \"_\": {}}, \":out-of-range\": {\"!important\": {}, \"_\": {}}, \":root\": {\"!important\": {}, \"_\": {}}, \":empty\": {\"!important\": {}, \"_\": {}}}})".to_string())
                        );
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                }
            }
            Err(err) => {
                panic!("{}", err.to_string());
            }
        }
    }

    #[tokio::test]
    async fn layout_context_created_with_success() {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        match std::fs::read_to_string("src/crealion/mocks/layout.nyr") {
            Ok(raw_nenyr) => {
                let mut parser = NenyrParser::new();

                match parser.parse(raw_nenyr, "src/crealion/mocks/layout.nyr".to_string()) {
                    Ok(parsed_ast) => {
                        reset_stylitron();

                        let (sender, _) = broadcast::channel(10);

                        let mut crealion = Crealion::new(
                            sender,
                            parsed_ast,
                            "src/crealion/mocks/layout.nyr".to_string(),
                        );

                        let result = crealion.create().await;

                        assert!(result.is_ok());
                        assert_eq!(result.unwrap(), (CrealionContextType::Layout, None));
                        assert_eq!(
                            STYLITRON.get("themes").map(|v| format!("{:?}", &*v)),
                            Some("Themes({\"dynamicLayout\": {\"light\": {\"primaryColor\": [\"--gNDnNldHTaq\", \"#FFFFFF\"]}, \"dark\": {\"primaryColor\": [\"--gNDnNldHTaq\", \"#1E1E1E\"]}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("aliases").map(|v| format!("{:?}", &*v)),
                            Some("Aliases({\"dynamicLayout\": {\"bgd\": \"background\", \"dp\": \"display\", \"transf\": \"transform\", \"pdg\": \"padding\", \"wd\": \"width\", \"hgt\": \"height\", \"flexDir\": \"flex-direction\"}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("variables").map(|v| format!("{:?}", &*v)),
                            Some("Variables({\"dynamicLayout\": {\"myColor\": [\"--gUcAVe3Ho2h\", \"#FF6677\"]}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("animations").map(|v| format!("{:?}", &*v)),
                            Some("Animation({\"dynamicLayout\": {\"borderFlash\": {\"gmLUMBMQvsEE\": {\"10%\": {\"border-color\": \"var(--gUcAVe3Ho2h)\", \"border-width\": \"1px\"}, \"30%,50%,70%\": {\"border-color\": \"red\", \"border-width\": \"3px\"}, \"90%\": {\"border-color\": \"green\", \"border-width\": \"2px\"}, \"100%\": {\"border-color\": \"var(--gUcAVe3Ho2h)\", \"border-width\": \"1px\"}}}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("styles").map(|v| format!("{:?}", &*v)),
                            Some("Styles({\"_\": {\"!important\": {\"background\": {\"\\\\!bgd-zwSc\": \"var(--gNDnNldHTaq)\"}, \"color\": {\"\\\\!clr-4W4E\": \"var(--gUcAVe3Ho2h)\"}, \"padding\": {\"\\\\!pdg-3KtM\": \"10px\"}, \"display\": {\"\\\\!dpy-5TuI\": \"flex\"}, \"align-items\": {\"\\\\!lgn-tms-sLJ6\": \"center\"}}, \"_\": {}}, \"::after\": {\"!important\": {\"content\": {\"\\\\!ftr\\\\.ctt-WT3W\": \"' '\"}, \"display\": {\"\\\\!ftr\\\\.dpy-S4vd\": \"block\"}, \"width\": {\"\\\\!ftr\\\\.wth-YYq9\": \"100%\"}, \"height\": {\"\\\\!ftr\\\\.hht-9X8O\": \"2px\"}, \"background\": {\"\\\\!ftr\\\\.bgd-zwSc\": \"var(--gNDnNldHTaq)\"}}, \"_\": {}}, \"::before\": {\"!important\": {}, \"_\": {}}, \"::first-line\": {\"!important\": {}, \"_\": {}}, \"::first-letter\": {\"!important\": {}, \"_\": {}}, \":hover\": {\"!important\": {\"color\": {\"\\\\!hvr\\\\.clr-4W4E\": \"var(--gUcAVe3Ho2h)\"}, \"border\": {\"\\\\!hvr\\\\.bdr-akTf\": \"2px solid var(--gNDnNldHTaq)\"}, \"animation-name\": {\"\\\\!hvr\\\\.ntn-nm-wVho\": \"gmLUMBMQvsEE\"}}, \"_\": {}}, \":active\": {\"!important\": {}, \"_\": {}}, \":focus\": {\"!important\": {}, \"_\": {}}, \":first-child\": {\"!important\": {}, \"_\": {}}, \":last-child\": {\"!important\": {}, \"_\": {}}, \":first-of-type\": {\"!important\": {}, \"_\": {}}, \":last-of-type\": {\"!important\": {}, \"_\": {}}, \":only-child\": {\"!important\": {}, \"_\": {}}, \":only-of-type\": {\"!important\": {}, \"_\": {}}, \":target\": {\"!important\": {}, \"_\": {}}, \":visited\": {\"!important\": {}, \"_\": {}}, \":checked\": {\"!important\": {}, \"_\": {}}, \":disabled\": {\"!important\": {}, \"_\": {}}, \":enabled\": {\"!important\": {}, \"_\": {}}, \":read-only\": {\"!important\": {}, \"_\": {}}, \":read-write\": {\"!important\": {}, \"_\": {}}, \":placeholder-shown\": {\"!important\": {}, \"_\": {}}, \":valid\": {\"!important\": {}, \"_\": {}}, \":invalid\": {\"!important\": {}, \"_\": {}}, \":required\": {\"!important\": {}, \"_\": {}}, \":optional\": {\"!important\": {}, \"_\": {}}, \":fullscreen\": {\"!important\": {}, \"_\": {}}, \":focus-within\": {\"!important\": {}, \"_\": {}}, \":out-of-range\": {\"!important\": {}, \"_\": {}}, \":root\": {\"!important\": {}, \"_\": {}}, \":empty\": {\"!important\": {}, \"_\": {}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("responsive").map(|v| format!("{:?}", &*v)),
                            Some("ResponsiveStyles({})".to_string())
                        );
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                }
            }
            Err(err) => {
                panic!("{}", err.to_string());
            }
        }
    }

    #[tokio::test]
    async fn module_context_created_with_success() {
        tokio::time::sleep(tokio::time::Duration::from_secs(35)).await;

        match std::fs::read_to_string("src/crealion/mocks/module.nyr") {
            Ok(raw_nenyr) => {
                let mut parser = NenyrParser::new();

                match parser.parse(raw_nenyr, "src/crealion/mocks/module.nyr".to_string()) {
                    Ok(parsed_ast) => {
                        reset_stylitron();

                        let (sender, _) = broadcast::channel(10);

                        let mut crealion = Crealion::new(
                            sender,
                            parsed_ast,
                            "src/crealion/mocks/module.nyr".to_string(),
                        );

                        let result = crealion.create().await;

                        assert!(result.is_ok());
                        assert_eq!(result.unwrap(), (CrealionContextType::Module, None));
                        assert_eq!(
                            STYLITRON.get("aliases").map(|v| format!("{:?}", &*v)),
                            Some("Aliases({\"modernCanvas\": {\"bgd\": \"background\", \"dp\": \"display\", \"transf\": \"transform\", \"pdg\": \"padding\", \"wd\": \"width\", \"hgt\": \"height\", \"flexDir\": \"flex-direction\"}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("variables").map(|v| format!("{:?}", &*v)),
                            Some("Variables({\"modernCanvas\": {\"myColor\": [\"--gdixceenEK6\", \"#FF6677\"]}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("animations").map(|v| format!("{:?}", &*v)),
                            Some("Animation({\"modernCanvas\": {\"borderFlash\": {\"gvd4g8WU1iS7\": {\"10%\": {\"border-color\": \"var(--gdixceenEK6)\", \"border-width\": \"1px\"}, \"30%,50%,70%\": {\"border-color\": \"red\", \"border-width\": \"3px\"}, \"90%\": {\"border-color\": \"green\", \"border-width\": \"2px\"}, \"100%\": {\"border-color\": \"var(--gdixceenEK6)\", \"border-width\": \"1px\"}}}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("styles").map(|v| format!("{:?}", &*v)),
                            Some("Styles({\"_\": {\"!important\": {\"color\": {\"\\\\!clr-nyD2\": \"var(--gdixceenEK6)\"}, \"padding\": {\"\\\\!pdg-3KtM\": \"10px\"}, \"display\": {\"\\\\!dpy-5TuI\": \"flex\"}, \"align-items\": {\"\\\\!lgn-tms-sLJ6\": \"center\"}}, \"_\": {}}, \"::after\": {\"!important\": {\"content\": {\"\\\\!ftr\\\\.ctt-WT3W\": \"' '\"}, \"display\": {\"\\\\!ftr\\\\.dpy-S4vd\": \"block\"}, \"width\": {\"\\\\!ftr\\\\.wth-YYq9\": \"100%\"}, \"height\": {\"\\\\!ftr\\\\.hht-9X8O\": \"2px\"}}, \"_\": {}}, \"::before\": {\"!important\": {}, \"_\": {}}, \"::first-line\": {\"!important\": {}, \"_\": {}}, \"::first-letter\": {\"!important\": {}, \"_\": {}}, \":hover\": {\"!important\": {\"color\": {\"\\\\!hvr\\\\.clr-nyD2\": \"var(--gdixceenEK6)\"}, \"animation-name\": {\"\\\\!hvr\\\\.ntn-nm-xyuz\": \"gvd4g8WU1iS7\"}}, \"_\": {}}, \":active\": {\"!important\": {}, \"_\": {}}, \":focus\": {\"!important\": {}, \"_\": {}}, \":first-child\": {\"!important\": {}, \"_\": {}}, \":last-child\": {\"!important\": {}, \"_\": {}}, \":first-of-type\": {\"!important\": {}, \"_\": {}}, \":last-of-type\": {\"!important\": {}, \"_\": {}}, \":only-child\": {\"!important\": {}, \"_\": {}}, \":only-of-type\": {\"!important\": {}, \"_\": {}}, \":target\": {\"!important\": {}, \"_\": {}}, \":visited\": {\"!important\": {}, \"_\": {}}, \":checked\": {\"!important\": {}, \"_\": {}}, \":disabled\": {\"!important\": {}, \"_\": {}}, \":enabled\": {\"!important\": {}, \"_\": {}}, \":read-only\": {\"!important\": {}, \"_\": {}}, \":read-write\": {\"!important\": {}, \"_\": {}}, \":placeholder-shown\": {\"!important\": {}, \"_\": {}}, \":valid\": {\"!important\": {}, \"_\": {}}, \":invalid\": {\"!important\": {}, \"_\": {}}, \":required\": {\"!important\": {}, \"_\": {}}, \":optional\": {\"!important\": {}, \"_\": {}}, \":fullscreen\": {\"!important\": {}, \"_\": {}}, \":focus-within\": {\"!important\": {}, \"_\": {}}, \":out-of-range\": {\"!important\": {}, \"_\": {}}, \":root\": {\"!important\": {}, \"_\": {}}, \":empty\": {\"!important\": {}, \"_\": {}}})".to_string())
                        );
                        assert_eq!(
                            STYLITRON.get("responsive").map(|v| format!("{:?}", &*v)),
                            Some("ResponsiveStyles({})".to_string())
                        );
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                }
            }
            Err(err) => {
                panic!("{}", err.to_string());
            }
        }
    }
}
