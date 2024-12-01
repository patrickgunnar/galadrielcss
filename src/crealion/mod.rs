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

    pub async fn create(&mut self) -> CrealionResult {
        match self.parsed_ast.clone() {
            NenyrAst::CentralContext(context) => self.init_central_collector(&context).await,
            NenyrAst::LayoutContext(context) => self.init_layout_collector(&context).await,
            NenyrAst::ModuleContext(context) => self.init_module_collector(&context).await,
        }
    }

    pub async fn init_central_collector(&mut self, context: &CentralContext) -> CrealionResult {
        let context_name = self.central_context_identifier.to_owned();

        let variables_data = self.get_value(
            context.variables.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        let (light_data, dark_data) = self.get_value(
            context
                .themes
                .as_ref()
                .map(|v| (v.light_schema.to_owned(), v.dark_schema.to_owned())),
            (None, None),
        );

        let (mobile_data, desktop_data) = self.get_value(
            context
                .breakpoints
                .as_ref()
                .map(|v| (v.mobile_first.to_owned(), v.desktop_first.to_owned())),
            (None, None),
        );

        let aliases_data = self.get_value(
            context.aliases.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        let typefaces_data = self.get_value(
            context.typefaces.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        let imports_data = self.get_value(
            context.imports.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

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
            Err(join_error) => self.handle_join_error(join_error),
            Ok(()) => {}
        });

        let inherited_contexts = vec![context_name.to_owned()];

        let animations_data = self.get_value(
            context.animations.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        self.process_animations(&context_name, &inherited_contexts, animations_data);

        let classes_data = self.get_value(
            context.classes.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        self.process_classes(
            context_name,
            inherited_contexts,
            None,
            classes_data,
            CrealionContextType::Central,
        )
        .await;

        Ok(None)
    }

    pub async fn init_layout_collector(&mut self, context: &LayoutContext) -> CrealionResult {
        let context_name = context.layout_name.to_owned();

        let variables_data = self.get_value(
            context.variables.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        let (light_data, dark_data) = self.get_value(
            context
                .themes
                .as_ref()
                .map(|v| (v.light_schema.to_owned(), v.dark_schema.to_owned())),
            (None, None),
        );

        let aliases_data = self.get_value(
            context.aliases.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        join_all(vec![
            self.process_variables(context_name.to_owned(), variables_data),
            self.process_themes(context_name.to_owned(), light_data, dark_data),
            self.apply_aliases_to_stylitron(context_name.to_owned(), aliases_data),
        ])
        .await
        .iter()
        .for_each(|future_result| match future_result {
            Err(join_error) => self.handle_join_error(join_error),
            Ok(()) => {}
        });

        let inherited_contexts = vec![
            context_name.to_owned(),
            self.central_context_identifier.to_owned(),
        ];

        let animations_data = self.get_value(
            context.animations.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        self.process_animations(&context_name, &inherited_contexts, animations_data);

        let classes_data = self.get_value(
            context.classes.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        self.process_classes(
            context_name,
            inherited_contexts,
            None,
            classes_data,
            CrealionContextType::Layout,
        )
        .await;

        Ok(None)
    }

    pub async fn init_module_collector(&mut self, context: &ModuleContext) -> CrealionResult {
        let context_name = context.module_name.to_owned();
        let extended_from = context.extending_from.to_owned().unwrap_or("".to_string());

        let variables_data = self.get_value(
            context.variables.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        let aliases_data = self.get_value(
            context.aliases.as_ref().map(|v| v.values.to_owned()),
            IndexMap::new(),
        );

        join_all(vec![
            self.process_variables(context_name.to_owned(), variables_data),
            self.apply_aliases_to_stylitron(context_name.to_owned(), aliases_data),
        ])
        .await
        .iter()
        .for_each(|future_result| match future_result {
            Err(join_error) => self.handle_join_error(join_error),
            Ok(()) => {}
        });

        let mut inherited_contexts = vec![
            context_name.to_owned(),
            extended_from.to_owned(),
            self.central_context_identifier.to_owned(),
        ];

        inherited_contexts.retain(|v| !v.is_empty());

        let animations_data = self.get_value(
            context.animations.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        self.process_animations(&context_name, &inherited_contexts, animations_data);

        let classes_data = self.get_value(
            context.classes.as_ref().map(|v| v.to_owned()),
            IndexMap::new(),
        );

        self.process_classes(
            context_name,
            inherited_contexts,
            context.extending_from.to_owned(),
            classes_data,
            CrealionContextType::Module,
        )
        .await;

        Ok(None)
    }

    pub fn transform_context_name(&self, context_name: &str) -> String {
        if context_name == self.central_context_identifier {
            "central".to_string()
        } else {
            context_name.to_string()
        }
    }

    fn get_value<T>(&self, opt: Option<T>, default: T) -> T {
        opt.unwrap_or_else(|| {
            tracing::warn!("Expected value but found None, using default.");
            default
        })
    }

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
