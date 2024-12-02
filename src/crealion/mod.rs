use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use nenyr::types::{
    ast::NenyrAst, central::CentralContext, layout::LayoutContext, module::ModuleContext,
};
use tokio::{sync::mpsc::UnboundedSender, task::JoinError};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    GaladrielResult,
};

mod aliases;
mod animations;
mod breakpoints;
mod classes;
mod classinator;
mod imports;
mod processors;
mod themes;
mod typefaces;
mod utils;
mod variables;

type CrealionResult = GaladrielResult<Option<Vec<String>>>;

#[derive(Clone, PartialEq, Debug)]
pub enum CrealionContextType {
    Central,
    Layout,
    Module,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Crealion {
    sender: UnboundedSender<ShellscapeAlerts>,
    central_context_identifier: String,
    parsed_ast: NenyrAst,
    path: String,
}

impl Crealion {
    /// Creates a new instance of `Crealion` with the given parameters.
    ///
    /// # Parameters
    /// - `sender`: A channel for sending `ShellscapeAlerts`.
    /// - `parsed_ast`: The parsed Abstract Syntax Tree (AST) of type `NenyrAst`.
    /// - `path`: The path related to the AST.
    pub fn new(
        sender: UnboundedSender<ShellscapeAlerts>,
        parsed_ast: NenyrAst,
        path: String,
    ) -> Self {
        Self {
            central_context_identifier: "gCtxCen_8Xq4ZJ".to_string(),
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

        // Process classes for the central context.
        self.process_classes(
            context_name,
            inherited_contexts,
            None, // No parent context for central.
            classes_data,
            CrealionContextType::Central,
        )
        .await;

        tracing::info!("Central Context Collector initialization completed");

        Ok(None)
    }

    /// Initializes the layout context by processing variables, themes, aliases, animations, and classes.
    ///
    /// # Arguments
    /// * `context` - A reference to the layout context containing the configuration details.
    async fn init_layout_collector(&mut self, context: &LayoutContext) -> CrealionResult {
        // Extract the layout name for use as the context identifier.
        let context_name = context.layout_name.to_owned();

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

        // Process class definitions, applying the layout-specific settings.
        self.process_classes(
            context_name.to_owned(),
            inherited_contexts,
            None, // No parent context for central - it defaults to the central context, but it does not need to be defined here.
            classes_data,
            CrealionContextType::Layout,
        )
        .await;

        tracing::info!(
            "Layout Context Collector initialization completed for: {}",
            context_name
        );

        Ok(None)
    }

    /// Initializes the module context by processing variables, aliases, animations, and classes.
    ///
    /// # Arguments
    /// * `context` - A reference to the module context containing module-specific configurations.
    pub async fn init_module_collector(&mut self, context: &ModuleContext) -> CrealionResult {
        // Extract the module name to use as the context identifier.
        let context_name = context.module_name.to_owned();
        // Extract the name of the context from which this module extends, or use an empty string if none.
        let extended_from = context.extending_from.to_owned().unwrap_or("".to_string());

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
        inherited_contexts.retain(|v| !v.is_empty());

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

        // Process class definitions, integrating any extended context and applying module-specific settings.
        self.process_classes(
            context_name.to_owned(),
            inherited_contexts,
            context.extending_from.to_owned(), // The parent for this context.
            classes_data,
            CrealionContextType::Module,
        )
        .await;

        tracing::info!(
            "Module Context Collector initialization completed for: {}",
            context_name
        );

        Ok(None)
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
    /// notification using `ShellscapeAlerts`, and attempts to send it via the
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

        let notification = ShellscapeAlerts::create_galadriel_error(Local::now(), error);

        if let Err(err) = sender.send(notification) {
            tracing::error!(
                "Failed to send notification for join error: {}. Send error: {}",
                join_error,
                err
            )
        }
    }
}
