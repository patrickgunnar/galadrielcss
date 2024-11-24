use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use regex::Regex;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    crealion::utils::{camelify::camelify, pascalify::pascalify},
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
    GaladrielResult,
};

const SCHEMAS: &[&str] = &["light", "dark"];

#[derive(Clone, Debug)]
pub struct VariablesProcessor {
    re: Regex,
    inherited_contexts: Vec<String>,
}

impl VariablesProcessor {
    pub fn new(inherited_contexts: Vec<String>) -> GaladrielResult<Self> {
        let re = Regex::new(r"\$\{(.*?)\}").map_err(|err| {
            GaladrielError::raise_general_other_error(
                ErrorKind::Other,
                &err.to_string(),
                ErrorAction::Notify,
            )
        })?;

        Ok(Self {
            inherited_contexts,
            re,
        })
    }

    pub async fn process(
        &self,
        value: &str,
        property: &str,
        class_name: &str,
        pattern: &str,
        breakpoint: &Option<String>,
        alerts: &mut Vec<ShellscapeAlerts>,
    ) -> GaladrielResult<Option<String>> {
        let mut resolved_value = value.to_string();
        let mut offset_index = 0;

        for capture in self.re.captures_iter(value) {
            let relative_name = &capture[1].to_string();

            let results = join_all(vec![
                self.process_from_variables_node(relative_name.to_owned()),
                self.process_from_themes_node(relative_name.to_owned()),
                self.process_from_animations_node(relative_name.to_owned()),
            ])
            .await;

            let resolved_result = results.iter().find_map(|result| match result {
                Ok(Some(v)) => Some(Ok(v.to_owned())),
                Err(err) => Some(Err(err.to_string())),
                Ok(None) => None,
            });

            match resolved_result {
                Some(Ok(replacement_value)) => {
                    let (start_pos, end_pos) = self.get_positions(capture)?;

                    resolved_value.replace_range(
                        start_pos.saturating_add(offset_index)
                            ..end_pos.saturating_add(offset_index),
                        &replacement_value,
                    );

                    let adjustment = if end_pos <= start_pos {
                        end_pos.saturating_sub(start_pos)
                    } else {
                        0
                    };

                    offset_index += replacement_value.len().saturating_sub(adjustment);
                }
                Some(Err(err)) => {
                    return Err(GaladrielError::raise_general_other_error(
                        ErrorKind::TaskFailure,
                        &err,
                        ErrorAction::Notify,
                    ))
                }
                None => {
                    let formatted_property = camelify(&property);
                    let formatted_pattern = pascalify(&pattern);
                    let panoramic_message = match breakpoint {
                        Some(name) => format!(
                            " This occurred in the `{}` breakpoint of the PanoramicViewer method.",
                            name
                        ),
                        None => String::new(),
                    };

                    let message = format!(
                        "The `{}` property in the `{}` class for the `{}` pattern references an unrecognized `{}` variable, preventing the style from being created. Please review and update the variable to ensure the style is generated correctly.{}",
                        formatted_property, class_name, formatted_pattern, relative_name, panoramic_message
                    );

                    alerts.insert(0, ShellscapeAlerts::create_warning(Local::now(), &message));
                    return Ok(None);
                }
            }
        }

        Ok(Some(resolved_value))
    }

    fn process_from_variables_node(&self, relative_name: String) -> JoinHandle<Option<String>> {
        let inherited_contexts = self.inherited_contexts.clone();

        tokio::task::spawn_blocking(move || {
            STYLITRON
                .get("variables")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Variables(variables_definitions) => {
                        inherited_contexts.iter().find_map(|context_name| {
                            variables_definitions
                                .get(context_name)
                                .and_then(|context_variables| {
                                    context_variables.get(&relative_name).and_then(
                                        |variable_entry| {
                                            variable_entry
                                                .get_index(0)
                                                .map(|(resolved_name, _)| resolved_name.to_owned())
                                        },
                                    )
                                })
                        })
                    }
                    _ => None,
                })
        })
    }

    fn process_from_themes_node(&self, relative_name: String) -> JoinHandle<Option<String>> {
        let inherited_contexts = self.inherited_contexts.clone();

        tokio::task::spawn_blocking(move || {
            STYLITRON
                .get("themes")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Themes(themes_definitions) => {
                        inherited_contexts.iter().find_map(|context_name| {
                            themes_definitions
                                .get(context_name)
                                .and_then(|context_themes| {
                                    SCHEMAS.iter().find_map(|schema| {
                                        Self::process_theme_schema(
                                            schema,
                                            &relative_name,
                                            context_themes,
                                        )
                                    })
                                })
                        })
                    }
                    _ => None,
                })
        })
    }

    fn process_theme_schema(
        schema: &str,
        relative_name: &str,
        context_themes: &IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
    ) -> Option<String> {
        context_themes.get(schema).and_then(|schema_themes| {
            schema_themes.get(relative_name).and_then(|variable_entry| {
                variable_entry
                    .get_index(0)
                    .map(|(resolved_name, _)| resolved_name.to_owned())
            })
        })
    }

    fn process_from_animations_node(&self, relative_name: String) -> JoinHandle<Option<String>> {
        let inherited_contexts = self.inherited_contexts.clone();

        tokio::task::spawn_blocking(move || {
            STYLITRON
                .get("animations")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Animation(animations_definitions) => {
                        inherited_contexts.iter().find_map(|context_name| {
                            animations_definitions.get(context_name).and_then(
                                |context_animations| {
                                    context_animations.get(&relative_name).and_then(
                                        |animation_entry| {
                                            animation_entry
                                                .get_index(0)
                                                .map(|(resolved_name, _)| resolved_name.to_owned())
                                        },
                                    )
                                },
                            )
                        })
                    }
                    _ => None,
                })
        })
    }

    fn get_positions(&self, capture: regex::Captures<'_>) -> GaladrielResult<(usize, usize)> {
        capture
            .get(0)
            .and_then(|cap| Some((cap.start(), cap.end())))
            .ok_or_else(|| {
                GaladrielError::raise_general_other_error(
                    ErrorKind::Other,
                    "Failed to retrieve capture group positions: capture group 0 not found.",
                    ErrorAction::Notify,
                )
            })
    }
}
