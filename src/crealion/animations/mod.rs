use chrono::Local;
use indexmap::IndexMap;
use nenyr::types::animations::{NenyrAnimation, NenyrAnimationKind, NenyrKeyframe};

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
};

use super::{
    utils::generates_variable_or_animation_name::generates_variable_or_animation_name, Crealion,
};

impl Crealion {
    pub async fn process_animations(
        &self,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        animations_data: IndexMap<String, NenyrAnimation>,
    ) {
        let sender = self.sender.clone();

        animations_data.into_values().for_each(|animation| {
            let animation_name = animation.animation_name;
            let animation_kind = animation.kind.unwrap_or(NenyrAnimationKind::None);

            match animation_kind {
                NenyrAnimationKind::Fraction => {
                    self.process_fraction_animation(
                        &animation_name,
                        context_name,
                        inherited_contexts,
                        animation.keyframe,
                    );
                }
                NenyrAnimationKind::Progressive => {
                    let progressive_size = animation.progressive_count.unwrap_or(0);

                    self.process_progressive_animation(
                        &animation_name,
                        progressive_size,
                        context_name,
                        inherited_contexts,
                        animation.keyframe,
                    );
                }
                NenyrAnimationKind::Transitive => {
                    self.process_transitive_animation(
                        &animation_name,
                        context_name,
                        inherited_contexts,
                        animation.keyframe,
                    );
                }
                NenyrAnimationKind::None => {
                    self.apply_animation_to_stylitron(
                        &animation_name,
                        context_name,
                        IndexMap::new(),
                    );

                    let warning = ShellscapeAlerts::create_warning(
                        Local::now(),
                        &format!(
                            "The animation `{}` within the context `{}` has been detected as empty.",
                            animation_name, if context_name == self.central_context_identifier { "central" } else { context_name }
                        ),
                    );

                    if let Err(err) = sender.send(warning) {
                        tracing::error!("{:?}", err);
                    }
                }
            }
        });
    }

    fn process_fraction_animation(
        &self,
        animation_name: &str,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        keyframes: Vec<NenyrKeyframe>,
    ) {
        let transformed_keyframes = keyframes
            .into_iter()
            .filter_map(|keyframe| {
                if let NenyrKeyframe::Fraction { stops, properties } = keyframe {
                    let fraction_stops = stops
                        .into_iter()
                        .map(|v| format!("{v}%"))
                        .collect::<Vec<_>>()
                        .join(",");

                    let processed_properties = self.process_animation_properties(
                        animation_name,
                        context_name,
                        inherited_contexts,
                        properties,
                    );

                    return Some((fraction_stops, processed_properties));
                }

                None
            })
            .collect::<IndexMap<String, IndexMap<String, String>>>();

        self.apply_animation_to_stylitron(animation_name, context_name, transformed_keyframes);
    }

    fn process_progressive_animation(
        &self,
        animation_name: &str,
        progressive_size: i64,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        keyframes: Vec<NenyrKeyframe>,
    ) {
        let progressive_value = if progressive_size == 1 {
            100.0
        } else if progressive_size > 1 {
            100.0 / progressive_size.saturating_sub(1) as f64
        } else {
            0.0
        };

        let transformed_keyframes = keyframes
            .into_iter()
            .enumerate()
            .filter_map(|(index, keyframe)| {
                if let NenyrKeyframe::Progressive(properties) = keyframe {
                    let progressive_stop = format!("{}%", progressive_value * index as f64);
                    let processed_properties = self.process_animation_properties(
                        animation_name,
                        context_name,
                        inherited_contexts,
                        properties,
                    );

                    return Some((progressive_stop, processed_properties));
                }

                None
            })
            .collect::<IndexMap<String, IndexMap<String, String>>>();

        self.apply_animation_to_stylitron(animation_name, context_name, transformed_keyframes);
    }

    fn process_transitive_animation(
        &self,
        animation_name: &str,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        keyframes: Vec<NenyrKeyframe>,
    ) {
        let transformed_keyframes = keyframes
            .into_iter()
            .filter_map(|keyframe| {
                match keyframe {
                    NenyrKeyframe::From(properties) => {
                        let processed_properties = self.process_animation_properties(
                            animation_name,
                            context_name,
                            inherited_contexts,
                            properties,
                        );

                        return Some(("0%".to_string(), processed_properties));
                    }
                    NenyrKeyframe::Halfway(properties) => {
                        let processed_properties = self.process_animation_properties(
                            animation_name,
                            context_name,
                            inherited_contexts,
                            properties,
                        );

                        return Some(("50%".to_string(), processed_properties));
                    }
                    NenyrKeyframe::To(properties) => {
                        let processed_properties = self.process_animation_properties(
                            animation_name,
                            context_name,
                            inherited_contexts,
                            properties,
                        );

                        return Some(("100%".to_string(), processed_properties));
                    }
                    _ => {}
                }

                None
            })
            .collect::<IndexMap<String, IndexMap<String, String>>>();

        self.apply_animation_to_stylitron(animation_name, context_name, transformed_keyframes);
    }

    fn process_animation_properties(
        &self,
        _animation_name: &str,
        _context_name: &str,
        _inherited_contexts: &Vec<String>,
        properties: IndexMap<String, String>,
    ) -> IndexMap<String, String> {
        let _sender = self.sender.clone();

        // TODO: Resolve the aliases and variables in the properties.

        // TEMP
        properties
    }

    fn apply_animation_to_stylitron(
        &self,
        animation_name: &str,
        context_name: &str,
        keyframes: IndexMap<String, IndexMap<String, String>>,
    ) {
        let sender = self.sender.clone();

        tracing::info!(
            "Starting to apply animations for context: {}, animation: {}",
            context_name,
            animation_name
        );

        // Attempt to retrieve the `animations` section of the STYLITRON AST.
        let mut stylitron_data = match STYLITRON.get_mut("animations") {
            Some(data) => {
                tracing::debug!("Successfully accessed the animations section in STYLITRON AST.");
                data
            }
            None => {
                tracing::error!(
                    "Failed to access the animations section in STYLITRON AST for context: {}",
                    context_name
                );

                // If the `animations` section is not found, raise a critical error.
                let error = GaladrielError::raise_critical_other_error(
                    ErrorKind::AccessDeniedToStylitronAST,
                    "Failed to access the animations section in STYLITRON AST",
                    ErrorAction::Restart,
                );

                tracing::error!("Critical error raised: {:?}", error);

                // Create a notification to report the error.
                let notification = ShellscapeAlerts::create_galadriel_error(Local::now(), error);

                // Attempt to send the notification and log any failures.
                if let Err(err) = sender.send(notification) {
                    tracing::error!("Failed to send notification: {}", err);
                }

                return;
            }
        };

        let animation_unique_name =
            generates_variable_or_animation_name(context_name, animation_name, false);

        // Check if the retrieved data matches the `Animations` variant.
        match *stylitron_data {
            // If it matches `Stylitron::Animations`, update its content with the provided data.
            Stylitron::Animation(ref mut animations_definitions) => {
                tracing::info!(
                    "Found `Animations` section in STYLITRON AST for context: {}",
                    context_name
                );

                // Find or create the animation for the specified context.
                let context_animations = animations_definitions
                    .entry(context_name.to_owned())
                    .or_default();

                // Update the animations for the context with the provided data.
                context_animations.insert(
                    animation_name.to_string(),
                    IndexMap::from([(animation_unique_name, keyframes)]),
                );

                tracing::debug!(
                    "Animation `{}` for context '{}' updated successfully.",
                    animation_name,
                    context_name,
                );
            }
            _ => {}
        }

        tracing::info!(
            "Completed animation application for context: {}, animation: {}",
            context_name,
            animation_name
        );
    }
}
