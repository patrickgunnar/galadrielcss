use chrono::Local;
use classes::ContextType;
use futures::future::join_all;
use nenyr::types::{
    ast::NenyrAst, central::CentralContext, layout::LayoutContext, module::ModuleContext,
};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    GaladrielResult,
};

mod animations;
mod breakpoints;
mod classes;
mod mocks;
mod processors;
mod utils;
mod variables;

type CrealionResult = GaladrielResult<(Option<Vec<ShellscapeAlerts>>, Option<Vec<String>>)>;

#[derive(Clone, PartialEq, Debug)]
pub struct Crealion {
    central_context_identifier: String,
    alerts: Vec<ShellscapeAlerts>,
    parsed_ast: NenyrAst,
    path: String,
}

impl Crealion {
    pub fn new(parsed_ast: NenyrAst, path: String) -> Self {
        Self {
            central_context_identifier: "gCtxCen_8Xq4ZJ".to_string(),
            alerts: vec![],
            parsed_ast,
            path,
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
        let inherited_contexts: Vec<String> = vec![self.central_context_identifier.to_owned()];

        let futures = join_all(vec![
            self.process_breakpoints(context.breakpoints.to_owned()),
            self.process_variables(
                self.central_context_identifier.to_owned(),
                context.variables.to_owned(),
            ),
        ])
        .await;

        futures.iter().for_each(|result| match result {
            Ok(alerts) => {
                self.alerts.append(&mut alerts.to_vec());
            }
            Err(err) => {
                self.alerts.push(ShellscapeAlerts::create_galadriel_error(
                    Local::now(),
                    GaladrielError::raise_general_other_error(
                        ErrorKind::TaskFailure,
                        &err.to_string(),
                        ErrorAction::Notify,
                    ),
                ));
            }
        });

        let animations_futures = join_all(context.animations.as_ref().map_or_else(
            || vec![],
            |animations| {
                animations
                    .iter()
                    .map(|(_, animation)| {
                        self.process_animation(
                            self.central_context_identifier.to_owned(),
                            inherited_contexts.to_vec(),
                            animation.to_owned(),
                        )
                    })
                    .collect::<Vec<_>>()
            },
        ))
        .await;

        animations_futures.iter().for_each(|result| match result {
            Ok(alerts) => {
                self.alerts.append(&mut alerts.to_vec());
            }
            Err(err) => {
                self.alerts.push(ShellscapeAlerts::create_galadriel_error(
                    Local::now(),
                    GaladrielError::raise_general_other_error(
                        ErrorKind::TaskFailure,
                        &err.to_string(),
                        ErrorAction::Notify,
                    ),
                ));
            }
        });

        let classes_futures = context.classes.as_ref().map_or_else(
            || vec![],
            |classes| {
                classes
                    .iter()
                    .map(|(_, class)| {
                        self.process_class(inherited_contexts.to_vec(), class.to_owned())
                    })
                    .collect::<Vec<_>>()
            },
        );

        let classes_results = join_all(classes_futures).await;
        let classes = classes_results
            .iter()
            .filter_map(|result| match result {
                Ok((class, alerts)) => {
                    self.alerts.append(&mut alerts.to_vec());

                    Some(class.to_owned())
                }
                Err(err) => {
                    self.alerts.push(ShellscapeAlerts::create_galadriel_error(
                        Local::now(),
                        GaladrielError::raise_general_other_error(
                            ErrorKind::TaskFailure,
                            &err.to_string(),
                            ErrorAction::Notify,
                        ),
                    ));

                    None
                }
            })
            .collect::<Vec<_>>();

        let result = self
            .handle_class_definitions(
                self.central_context_identifier.to_owned(),
                None,
                classes,
                ContextType::Central,
            )
            .await
            .map_err(|err| {
                GaladrielError::raise_general_other_error(
                    ErrorKind::TaskFailure,
                    &err.to_string(),
                    ErrorAction::Notify,
                )
            });

        match result {
            Ok(alerts) => {
                self.alerts.append(&mut alerts.to_vec());
            }
            Err(err) => {
                self.alerts
                    .push(ShellscapeAlerts::create_galadriel_error(Local::now(), err));
            }
        }

        Ok((Some(self.alerts.clone()), None))
    }

    pub async fn init_layout_collector(&mut self, _context: &LayoutContext) -> CrealionResult {
        Ok((None, None))
    }

    pub async fn init_module_collector(&mut self, _context: &ModuleContext) -> CrealionResult {
        Ok((None, None))
    }
}
