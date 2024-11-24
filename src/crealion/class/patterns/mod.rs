use chrono::Local;
use indexmap::IndexMap;

use crate::{
    crealion::{
        processors::{
            breakpoint::BreakpointProcessor, nickname::NicknameProcessor,
            variables::VariablesProcessor,
        },
        utils::{generate_utility_class_name::generate_utility_class_name, pascalify::pascalify},
        Crealion,
    },
    shellscape::alerts::ShellscapeAlerts,
};

use super::types::UtilityClass;

impl Crealion {
    pub async fn match_style_patterns(
        inherited_contexts: &Vec<String>,
        breakpoint: Option<String>,
        class_name: &str,
        is_important: bool,
        alerts: &mut Vec<ShellscapeAlerts>,
        classes: &mut Vec<UtilityClass>,
        patterns: IndexMap<String, IndexMap<String, String>>,
    ) {
        let variables_processor = match VariablesProcessor::new(inherited_contexts.to_vec()) {
            Ok(processor) => processor,
            Err(err) => {
                alerts.insert(
                    0,
                    ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                );

                return;
            }
        };

        let breakpoint_value = breakpoint
            .as_ref()
            .map(|b| BreakpointProcessor::new(b).process())
            .unwrap_or(None);

        let nickname_processor = NicknameProcessor::new(inherited_contexts.to_vec());

        for (pattern, style) in patterns {
            for (property, value) in style {
                match Self::process_style_property(
                    &property,
                    class_name,
                    &pattern,
                    &breakpoint,
                    alerts,
                    &nickname_processor,
                ) {
                    Ok(new_property) => match variables_processor
                        .process(&value, &property, class_name, &pattern, &breakpoint, alerts)
                        .await
                    {
                        Ok(Some(new_value)) => {
                            let new_pattern = pattern.trim_end_matches("stylesheet");
                            let utility_cls_name = generate_utility_class_name(
                                &breakpoint,
                                is_important,
                                new_pattern,
                                &new_property,
                                &new_value,
                            );

                            classes.push(UtilityClass::create_class(
                                &breakpoint_value,
                                new_pattern,
                                &utility_cls_name,
                                is_important,
                                &new_property,
                                &new_value,
                            ));
                        }
                        Err(err) => {
                            alerts.insert(
                                0,
                                ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                            );
                        }
                        Ok(None) => {}
                    },
                    Err(_) => {}
                }
            }
        }
    }

    fn process_style_property(
        alias: &str,
        class_name: &str,
        pattern: &str,
        breakpoint: &Option<String>,
        alerts: &mut Vec<ShellscapeAlerts>,
        nickname_processor: &NicknameProcessor,
    ) -> Result<String, ()> {
        if alias.starts_with("nickname;") {
            let alias_value = alias.trim_start_matches("nickname;");

            match nickname_processor.process(alias_value) {
                Some(processed_alias) => return Ok(processed_alias),
                None => {
                    let formatted_pattern = pascalify(&pattern);
                    let panoramic_message = match breakpoint {
                        Some(name) => format!(
                            " This occurred in the `{}` breakpoint of the PanoramicViewer method.",
                            name
                        ),
                        None => String::new(),
                    };

                    let message = format!(
                        "The alias `{}` in the `{}` class for the `{}` pattern was not recognized. The style could not be created for this value. Please review and update the alias to ensure the style is generated correctly.{}",
                        alias_value, class_name, formatted_pattern, panoramic_message
                    );

                    alerts.insert(0, ShellscapeAlerts::create_warning(Local::now(), &message));

                    return Err(());
                }
            }
        }

        Ok(alias.to_string())
    }
}
