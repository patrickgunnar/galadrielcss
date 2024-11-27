use chrono::Local;
use futures::future::join_all;
use indexmap::IndexMap;
use nenyr::types::animations::{NenyrAnimation, NenyrAnimationKind, NenyrKeyframe};
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
    GaladrielResult,
};

use super::{
    processors::{nickname::NicknameProcessor, variables::VariablesProcessor},
    utils::generates_variable_or_animation_name::generates_variable_or_animation_name,
    Crealion,
};

impl Crealion {
    pub fn process_animation(
        &self,
        context: String,
        inherited_contexts: Vec<String>,
        animation: NenyrAnimation,
    ) -> JoinHandle<Vec<ShellscapeAlerts>> {
        tokio::task::spawn(async move {
            let mut alerts: Vec<ShellscapeAlerts> = vec![];
            let animation_name = animation.animation_name.to_owned();
            let animation_kind = animation.kind.to_owned();
            let keyframes = animation.keyframe.to_owned();

            let processed_keyframes = match animation_kind {
                Some(NenyrAnimationKind::Fraction) => {
                    Self::process_fraction_keyframes(
                        &mut alerts,
                        &animation_name,
                        &keyframes,
                        &inherited_contexts,
                    )
                    .await
                }
                Some(NenyrAnimationKind::Progressive) => match animation.progressive_count {
                    Some(progressive_count) => {
                        Self::process_progressive_keyframes(
                            &mut alerts,
                            &animation_name,
                            &keyframes,
                            &inherited_contexts,
                            progressive_count,
                        )
                        .await
                    }
                    None => IndexMap::new(),
                },
                Some(NenyrAnimationKind::Transitive) => {
                    Self::process_transitive_keyframes(
                        &mut alerts,
                        &animation_name,
                        &keyframes,
                        &inherited_contexts,
                    )
                    .await
                }
                _ => IndexMap::new(),
            };

            let unique_animation_name =
                generates_variable_or_animation_name(&context, &animation_name, false);

            if let Err(err) = Self::apply_animation_to_stylitron(
                context,
                animation_name,
                unique_animation_name,
                processed_keyframes,
            ) {
                alerts.insert(
                    0,
                    ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                );
            }

            alerts
        })
    }

    fn apply_animation_to_stylitron(
        context: String,
        animation_name: String,
        unique_animation_name: String,
        animation: IndexMap<String, IndexMap<String, String>>,
    ) -> GaladrielResult<()> {
        match STYLITRON.get_mut("animations") {
            Some(mut stylitron_data) => {
                match *stylitron_data {
                    Stylitron::Animation(ref mut animation_definitions) => {
                        let context_maps = animation_definitions.entry(context).or_default();
                        let animation_map = context_maps.entry(animation_name).or_default();

                        animation_map.insert(unique_animation_name, animation);
                    }
                    _ => {}
                }

                Ok(())
            }
            None => Err(GaladrielError::raise_critical_other_error(
                ErrorKind::AccessDeniedToStylitronAST,
                "Failed to access the 'animations' entry in the Stylitron AST.",
                ErrorAction::Restart,
            )),
        }
    }

    async fn process_fraction_keyframes(
        alerts: &mut Vec<ShellscapeAlerts>,
        animation_name: &str,
        keyframes: &[NenyrKeyframe],
        inherited_contexts: &[String],
    ) -> IndexMap<String, IndexMap<String, String>> {
        futures::future::join_all(keyframes.iter().filter_map(|keyframe| {
            if let NenyrKeyframe::Fraction { stops, properties } = keyframe {
                Some(Self::process_keyframe(
                    animation_name.to_string(),
                    stops.clone(),
                    properties.clone(),
                    inherited_contexts.to_vec(),
                ))
            } else {
                None
            }
        }))
        .await
        .iter()
        .filter_map(|result| match result {
            Ok((data, my_alerts)) => {
                alerts.append(&mut my_alerts.to_vec());

                data.to_owned()
            }
            Err(err) => {
                let error = GaladrielError::raise_general_other_error(
                    ErrorKind::TaskFailure,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                alerts.push(ShellscapeAlerts::create_galadriel_error(
                    Local::now(),
                    error,
                ));

                None
            }
        })
        .collect::<IndexMap<_, _>>()
    }

    async fn process_progressive_keyframes(
        alerts: &mut Vec<ShellscapeAlerts>,
        animation_name: &str,
        keyframes: &[NenyrKeyframe],
        inherited_contexts: &[String],
        progressive_count: i64,
    ) -> IndexMap<String, IndexMap<String, String>> {
        let stop_value = 100.0 / progressive_count as f64;

        futures::future::join_all(keyframes.iter().enumerate().filter_map(|(idx, keyframe)| {
            if let NenyrKeyframe::Progressive(properties) = keyframe {
                Some(Self::process_keyframe(
                    animation_name.to_string(),
                    vec![stop_value * idx as f64],
                    properties.clone(),
                    inherited_contexts.to_vec(),
                ))
            } else {
                None
            }
        }))
        .await
        .iter()
        .filter_map(|result| match result {
            Ok((data, my_alerts)) => {
                alerts.append(&mut my_alerts.to_vec());

                data.to_owned()
            }
            Err(err) => {
                let error = GaladrielError::raise_general_other_error(
                    ErrorKind::TaskFailure,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                alerts.push(ShellscapeAlerts::create_galadriel_error(
                    Local::now(),
                    error,
                ));

                None
            }
        })
        .collect::<IndexMap<_, _>>()
    }

    async fn process_transitive_keyframes(
        alerts: &mut Vec<ShellscapeAlerts>,
        animation_name: &str,
        keyframes: &[NenyrKeyframe],
        inherited_contexts: &[String],
    ) -> IndexMap<String, IndexMap<String, String>> {
        futures::future::join_all(keyframes.iter().filter_map(|keyframe| match keyframe {
            NenyrKeyframe::From(properties) => Some(Self::process_keyframe(
                animation_name.to_string(),
                vec![0.0],
                properties.clone(),
                inherited_contexts.to_vec(),
            )),
            NenyrKeyframe::Halfway(properties) => Some(Self::process_keyframe(
                animation_name.to_string(),
                vec![50.0],
                properties.clone(),
                inherited_contexts.to_vec(),
            )),
            NenyrKeyframe::To(properties) => Some(Self::process_keyframe(
                animation_name.to_string(),
                vec![100.0],
                properties.clone(),
                inherited_contexts.to_vec(),
            )),
            _ => None,
        }))
        .await
        .iter()
        .filter_map(|result| match result {
            Ok((data, my_alerts)) => {
                alerts.append(&mut my_alerts.to_vec());

                data.to_owned()
            }
            Err(err) => {
                let error = GaladrielError::raise_general_other_error(
                    ErrorKind::TaskFailure,
                    &err.to_string(),
                    ErrorAction::Notify,
                );

                alerts.push(ShellscapeAlerts::create_galadriel_error(
                    Local::now(),
                    error,
                ));

                None
            }
        })
        .collect::<IndexMap<_, _>>()
    }

    fn process_keyframe(
        animation_name: String,
        stops: Vec<f64>,
        properties: IndexMap<String, String>,
        inherited_contexts: Vec<String>,
    ) -> JoinHandle<(
        Option<(String, IndexMap<String, String>)>,
        Vec<ShellscapeAlerts>,
    )> {
        tokio::task::spawn(async move {
            let mut alerts: Vec<ShellscapeAlerts> = vec![];

            let stops_value = stops
                .iter()
                .map(|v| format!("{v}%"))
                .collect::<Vec<_>>()
                .join(",");

            let property_futures = properties.iter().map(|(property, value)| {
                Self::process_keyframe_property_value(
                    animation_name.to_owned(),
                    property.to_owned(),
                    value.to_owned(),
                    inherited_contexts.to_vec(),
                )
            });

            let processed_properties = join_all(property_futures)
                .await
                .into_iter()
                .filter_map(|result| match result {
                    Ok((data, my_alerts)) => {
                        alerts.extend(my_alerts);

                        data.to_owned()
                    }
                    Err(err) => {
                        let error = GaladrielError::raise_general_other_error(
                            ErrorKind::TaskFailure,
                            &err.to_string(),
                            ErrorAction::Notify,
                        );

                        alerts.push(ShellscapeAlerts::create_galadriel_error(
                            Local::now(),
                            error,
                        ));

                        None
                    }
                })
                .collect::<IndexMap<_, _>>();

            (Some((stops_value, processed_properties)), alerts)
        })
    }

    fn process_keyframe_property_value(
        animation_name: String,
        property: String,
        value: String,
        inherited_contexts: Vec<String>,
    ) -> JoinHandle<(Option<(String, String)>, Vec<ShellscapeAlerts>)> {
        tokio::task::spawn(async move {
            let mut alerts: Vec<ShellscapeAlerts> = vec![];

            let updated_property = match property.strip_prefix("nickname;") {
                Some(alias_value) => {
                    let nickname_processor = NicknameProcessor::new(inherited_contexts.to_vec());

                    match nickname_processor.process(alias_value) {
                        Some(processed_alias) => processed_alias,
                        None => {
                            alerts.insert(
                                0,
                                ShellscapeAlerts::create_warning(
                                    Local::now(),
                                    &format!(
                                        "The alias `{}` in the `{}` animation was not recognized. The style could not be created for this value. Please review and update the alias to ensure the style is generated correctly.",
                                        alias_value, animation_name
                                    )
                                )
                            );

                            return (None, alerts);
                        }
                    }
                }
                None => property.to_owned(),
            };

            let variables_processor = match VariablesProcessor::new(inherited_contexts) {
                Ok(processor) => processor,
                Err(err) => {
                    alerts.insert(
                        0,
                        ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                    );

                    return (None, alerts);
                }
            };

            let updated_value = match variables_processor
                .process(
                    &value,
                    &property,
                    &animation_name,
                    "",
                    &None,
                    &mut alerts,
                    true,
                )
                .await
            {
                Ok(Some(val)) => val,
                Ok(None) => {
                    return (None, alerts);
                }
                Err(err) => {
                    alerts.insert(
                        0,
                        ShellscapeAlerts::create_galadriel_error(Local::now(), err),
                    );

                    return (None, alerts);
                }
            };

            (Some((updated_property, updated_value)), alerts)
        })
    }
}
