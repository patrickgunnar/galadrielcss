use chrono::Local;
use indexmap::IndexMap;
use nenyr::types::animations::{NenyrAnimation, NenyrAnimationKind, NenyrKeyframe};

use crate::{
    asts::STYLITRON,
    crealion::{processors::variables::VariablesOption, utils::camelify::camelify},
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
    types::Stylitron,
};

use super::{
    processors::{aliases::resolve_alias_identifier, variables::resolve_variable_from_str},
    utils::generates_variable_or_animation_name::generates_variable_or_animation_name,
    Crealion,
};

impl Crealion {
    /// Processes animations for the specified context by iterating through animation data
    /// and delegating processing based on the animation kind.
    ///
    /// # Parameters
    /// - `context_name`: The name of the current context where the animations are being applied.
    /// - `inherited_contexts`: A list of inherited context names to consider during processing.
    /// - `animations_data`: A map containing animation data, keyed by animation name.
    pub fn process_animations(
        &self,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        animations_data: IndexMap<String, NenyrAnimation>,
    ) {
        let sender = self.sender.clone();

        tracing::info!(
            "Starting to process animations for context `{}` with {} inherited contexts and {} animations.",
            context_name,
            inherited_contexts.len(),
            animations_data.len(),
        );

        // Iterate through the animations data.
        animations_data.into_values().for_each(|animation| {
            let animation_name = animation.animation_name;
            let animation_kind = animation.kind.unwrap_or(NenyrAnimationKind::None);

            tracing::debug!(
                "Processing animation `{}` of kind `{:?}` in context `{}`.",
                animation_name,
                animation_kind,
                context_name,
            );

            // Handle each kind of animation separately.
            match animation_kind {
                NenyrAnimationKind::Fraction => {
                    tracing::trace!(
                        "Delegating to `process_fraction_animation` for animation `{}`.",
                        animation_name,
                    );

                    // Process fraction-based animations.
                    self.process_fraction_animation(
                        &animation_name,
                        context_name,
                        inherited_contexts,
                        animation.keyframe,
                    );
                }
                NenyrAnimationKind::Progressive => {
                    // Process progressive animations, using the provided progressive size or defaulting to 0.
                    let progressive_size = animation.progressive_count.unwrap_or(0);

                    tracing::trace!(
                        "Delegating to `process_progressive_animation` for animation `{}` with progressive size `{}`.",
                        animation_name,
                        progressive_size,
                    );

                    self.process_progressive_animation(
                        &animation_name,
                        progressive_size,
                        context_name,
                        inherited_contexts,
                        animation.keyframe,
                    );
                }
                NenyrAnimationKind::Transitive => {
                    tracing::trace!(
                        "Delegating to `process_transitive_animation` for animation `{}`.",
                        animation_name,
                    );

                    // Process transitive animations.
                    self.process_transitive_animation(
                        &animation_name,
                        context_name,
                        inherited_contexts,
                        animation.keyframe,
                    );
                }
                NenyrAnimationKind::None => {
                    tracing::warn!(
                        "Animation `{}` in context `{}` has no kind defined. Applying as an empty animation.",
                        animation_name,
                        context_name,
                    );

                    // Apply an empty animation and issue a warning about it.
                    self.apply_animation_to_stylitron(
                        &animation_name,
                        context_name,
                        IndexMap::new(),
                    );

                    let warning = ShellscapeAlerts::create_warning(
                        Local::now(),
                        &format!(
                            "The animation `{}` within the context `{}` has been detected as empty.",
                            animation_name, self.transform_context_name(context_name)
                        ),
                    );

                    if let Err(err) = sender.send(warning) {
                        tracing::error!("Failed to send warning notification: {:?}", err);
                    }
                }
            }
        });

        tracing::info!(
            "Completed processing animations for context `{}`.",
            context_name
        );
    }

    /// Processes animations defined with fractional stops.
    ///
    /// # Parameters
    /// - `animation_name`: The name of the animation.
    /// - `context_name`: The current context name.
    /// - `inherited_contexts`: A list of inherited context names.
    /// - `keyframes`: A vector of keyframes defining the animation.
    fn process_fraction_animation(
        &self,
        animation_name: &str,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        keyframes: Vec<NenyrKeyframe>,
    ) {
        tracing::info!(
            "Processing fraction animation `{}` in context `{}` with {} keyframes.",
            animation_name,
            context_name,
            keyframes.len(),
        );

        // Transform keyframes into a format suitable for Stylitron.
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

                    tracing::trace!(
                        "Processed keyframe with stops `{}` for animation `{}`.",
                        fraction_stops,
                        animation_name,
                    );

                    return Some((fraction_stops, processed_properties));
                }

                tracing::warn!(
                    "Skipped invalid keyframe for fraction animation `{}` in context `{}`.",
                    animation_name,
                    context_name,
                );

                None
            })
            .collect::<IndexMap<String, IndexMap<String, String>>>();

        // Apply the transformed keyframes to Stylitron.
        self.apply_animation_to_stylitron(animation_name, context_name, transformed_keyframes);

        tracing::info!(
            "Fraction animation `{}` processed and applied to Stylitron.",
            animation_name
        );
    }

    /// Processes animations defined with progressive stops.
    ///
    /// # Parameters
    /// - `animation_name`: The name of the animation.
    /// - `progressive_size`: The number of progressive steps in the animation.
    /// - `context_name`: The current context name.
    /// - `inherited_contexts`: A list of inherited context names.
    /// - `keyframes`: A vector of keyframes defining the animation.
    fn process_progressive_animation(
        &self,
        animation_name: &str,
        progressive_size: i64,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        keyframes: Vec<NenyrKeyframe>,
    ) {
        tracing::info!(
            "Processing progressive animation `{}` in context `{}` with size `{}` and {} keyframes.",
            animation_name,
            context_name,
            progressive_size,
            keyframes.len(),
        );

        // Calculate the percentage increment for each step of the progressive animation.
        let progressive_value = if progressive_size == 1 {
            100.0
        } else if progressive_size > 1 {
            100.0 / progressive_size.saturating_sub(1) as f64
        } else {
            0.0
        };

        // Transform keyframes into a format suitable for Stylitron.
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

                    tracing::trace!(
                        "Processed keyframe `{}` for progressive animation `{}`.",
                        progressive_stop,
                        animation_name,
                    );

                    return Some((progressive_stop, processed_properties));
                }

                tracing::warn!(
                    "Skipped invalid keyframe for progressive animation `{}` in context `{}`.",
                    animation_name,
                    context_name,
                );

                None
            })
            .collect::<IndexMap<String, IndexMap<String, String>>>();

        // Apply the transformed keyframes to Stylitron.
        self.apply_animation_to_stylitron(animation_name, context_name, transformed_keyframes);

        tracing::info!(
            "Progressive animation `{}` processed and applied to Stylitron.",
            animation_name,
        );
    }

    /// Processes and applies a transitive animation by transforming its keyframes
    /// and integrating it into the STYLITRON AST.
    ///
    /// # Arguments
    /// - `animation_name` - The name of the animation to be processed.
    /// - `context_name` - The name of the current context in which the animation resides.
    /// - `inherited_contexts` - A list of inherited contexts used for resolving aliases and variables.
    /// - `keyframes` - A vector of `NenyrKeyframe` objects representing the animation's keyframes.
    fn process_transitive_animation(
        &self,
        animation_name: &str,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        keyframes: Vec<NenyrKeyframe>,
    ) {
        tracing::info!(
            "Starting transitive animation processing: animation_name={}, context_name={}",
            animation_name,
            context_name
        );

        // Transform keyframes into a format suitable for the STYLITRON AST.
        let transformed_keyframes = keyframes
            .into_iter()
            .filter_map(|keyframe| {
                tracing::debug!(
                    "Processing keyframe for animation '{}', context '{}': {:?}",
                    animation_name,
                    context_name,
                    keyframe
                );

                match keyframe {
                    NenyrKeyframe::From(properties) => {
                        // Process "From" keyframe properties.
                        let processed_properties = self.process_animation_properties(
                            animation_name,
                            context_name,
                            inherited_contexts,
                            properties,
                        );

                        tracing::debug!(
                            "Mapped 'From' keyframe to '0%' for animation '{}': {:?}",
                            animation_name,
                            processed_properties
                        );

                        // Map the keyframe to "0%" timing.
                        return Some(("0%".to_string(), processed_properties));
                    }
                    NenyrKeyframe::Halfway(properties) => {
                        // Process "Halfway" keyframe properties.
                        let processed_properties = self.process_animation_properties(
                            animation_name,
                            context_name,
                            inherited_contexts,
                            properties,
                        );

                        tracing::debug!(
                            "Mapped 'Halfway' keyframe to '50%' for animation '{}': {:?}",
                            animation_name,
                            processed_properties
                        );

                        // Map the keyframe to "50%" timing.
                        return Some(("50%".to_string(), processed_properties));
                    }
                    NenyrKeyframe::To(properties) => {
                        // Process "To" keyframe properties.
                        let processed_properties = self.process_animation_properties(
                            animation_name,
                            context_name,
                            inherited_contexts,
                            properties,
                        );

                        tracing::debug!(
                            "Mapped 'To' keyframe to '100%' for animation '{}': {:?}",
                            animation_name,
                            processed_properties
                        );

                        // Map the keyframe to "100%" timing.
                        return Some(("100%".to_string(), processed_properties));
                    }
                    _ => {}
                }

                None
            })
            .collect::<IndexMap<String, IndexMap<String, String>>>();

        // Apply the transformed animation keyframes to the STYLITRON AST.
        self.apply_animation_to_stylitron(animation_name, context_name, transformed_keyframes);

        tracing::info!(
            "Transitive animation `{}` processed and applied to Stylitron.",
            animation_name,
        );
    }

    /// Processes the properties of an animation keyframe by resolving aliases
    /// and variables based on inherited contexts.
    ///
    /// # Arguments
    /// - `animation_name` - The name of the animation.
    /// - `context_name` - The name of the current context.
    /// - `inherited_contexts` - A list of inherited contexts for resolving identifiers.
    /// - `properties` - The keyframe properties to process.
    ///
    /// # Returns
    /// An `IndexMap` containing resolved property-value pairs.
    fn process_animation_properties(
        &self,
        animation_name: &str,
        context_name: &str,
        inherited_contexts: &Vec<String>,
        properties: IndexMap<String, String>,
    ) -> IndexMap<String, String> {
        tracing::info!(
            "Resolving properties for animation '{}', context '{}': {:?}",
            animation_name,
            context_name,
            properties
        );

        properties
            .into_iter()
            .filter_map(|(identifier, value)| {
                // Attempt to resolve the property alias using inherited contexts.
                match resolve_alias_identifier(&identifier, inherited_contexts) {
                    Some(property) => {
                        // Resolve the variable value using inherited contexts.
                        match resolve_variable_from_str(value, false, inherited_contexts) {
                            VariablesOption::Some(resolved_value) => {
                                tracing::debug!(
                                    "Resolved property '{}' with value '{}' for animation '{}'",
                                    property, resolved_value, animation_name
                                );

                                // Return the resolved property and value.
                                return Some((property, resolved_value));
                            }
                            VariablesOption::Unresolved(unresolved_variable) => {
                                let property = camelify(&property);
                                let context_name = self.transform_context_name(context_name);

                                tracing::warn!(
                                    "Unresolved variable in property '{}' for animation '{}' in context '{}'.",
                                    property, animation_name, context_name
                                );

                                // Raise a warning if the variable could not be resolved.
                                self.raise_warning(&format!(
                                    "The `{}` property in the `{}` animation of the `{}` context contains unresolved variable: `{}`. The variable were not found in the current context or any of its extension contexts. As a result, the style corresponding to the `{}` property was not created. Please verify the variable definitions and their scope.",
                                    property, animation_name, context_name, unresolved_variable, property
                                ));
                            }
                        }
                    }
                    None => {
                        let alias = identifier.trim_start_matches("nickname;");
                        let context_name = self.transform_context_name(context_name);

                        tracing::warn!(
                            "Unresolved alias '{}' for animation '{}' in context '{}'.",
                            alias, animation_name, context_name
                        );

                        // Raise a warning if the alias could not be resolved.
                        self.raise_warning(&format!(
                            "Warning: The `{}` alias in the `{}` animation of the `{}` context was not identified in the current context or any of its extension contexts. As a result, the style corresponding to the `{}` alias was not created. Please verify the alias definition and its scope.",
                            alias, animation_name, context_name, alias
                        ));
                    }
                }

                None
            })
            .collect()
    }

    /// Raises a warning by creating and sending a notification.
    ///
    /// # Arguments
    /// - `message` - The warning message to be raised.
    fn raise_warning(&self, message: &str) {
        let sender = self.sender.clone();
        let notification = ShellscapeAlerts::create_warning(Local::now(), message);

        // Attempt to send the warning notification.
        if let Err(err) = sender.send(notification) {
            tracing::error!("Failed to send warning notification: {:?}", err);
        }
    }

    /// Applies the transformed animation keyframes to the STYLITRON AST.
    ///
    /// # Arguments
    /// - `animation_name` - The name of the animation to apply.
    /// - `context_name` - The context to which the animation belongs.
    /// - `keyframes` - The transformed keyframes to apply.
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

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nenyr::types::{
        animations::{NenyrAnimation, NenyrAnimationKind, NenyrKeyframe},
        ast::NenyrAst,
        central::CentralContext,
    };
    use tokio::sync::mpsc;

    use crate::{
        asts::STYLITRON,
        crealion::{
            utils::generates_variable_or_animation_name::generates_variable_or_animation_name,
            Crealion,
        },
        shellscape::alerts::ShellscapeAlerts,
        types::Stylitron,
    };

    fn mock_animations() -> IndexMap<String, NenyrAnimation> {
        IndexMap::from([(
            "testingAnimation".to_string(),
            NenyrAnimation {
                animation_name: "testingAnimation".to_string(),
                kind: Some(NenyrAnimationKind::Progressive),
                progressive_count: Some(2),
                keyframe: vec![
                    NenyrKeyframe::Progressive(IndexMap::from([(
                        "background-color".to_string(),
                        "#FF0000".to_string(),
                    )])),
                    NenyrKeyframe::Progressive(IndexMap::from([(
                        "background-color".to_string(),
                        "#00FFFF".to_string(),
                    )])),
                ],
            },
        )])
    }

    fn transform_animations(
        context_name: &str,
    ) -> IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>> {
        let unique_name =
            generates_variable_or_animation_name(context_name, "testingAnimation", false);

        IndexMap::from([(
            "testingAnimation".to_string(),
            IndexMap::from([(
                unique_name,
                IndexMap::from([
                    (
                        "0%".to_string(),
                        IndexMap::from([("background-color".to_string(), "#FF0000".to_string())]),
                    ),
                    (
                        "100%".to_string(),
                        IndexMap::from([("background-color".to_string(), "#00FFFF".to_string())]),
                    ),
                ]),
            )]),
        )])
    }

    #[test]
    fn test_apply_animations_success() {
        let (sender, _) = mpsc::unbounded_channel();

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let inherits = vec!["myAnimationContextOne".to_string()];
        let _ = crealion.process_animations("myAnimationContextOne", &inherits, mock_animations());

        let result =
            STYLITRON
                .get("animations")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Animation(animations_definitions) => animations_definitions
                        .get("myAnimationContextOne")
                        .and_then(|context_animations| Some(context_animations.to_owned())),
                    _ => None,
                });

        assert!(result.is_some());

        let animations = result.unwrap();
        let expected_animations = transform_animations("myAnimationContextOne");

        assert_eq!(animations, expected_animations);
    }

    #[test]
    fn test_apply_animations_to_existing_context() {
        let (sender, _) = mpsc::unbounded_channel();

        // Pre-populate the STYLITRON AST with existing data.
        let initial_data = IndexMap::from([(
            "newContext".to_string(),
            IndexMap::from([(
                "initialAnimation".to_string(),
                IndexMap::from([(
                    "gs83jd25d28k".to_string(),
                    IndexMap::from([(
                        "0%".to_string(),
                        IndexMap::from([("background-color".to_string(), "#FF0000".to_string())]),
                    )]),
                )]),
            )]),
        )]);

        STYLITRON.insert("animations".to_string(), Stylitron::Animation(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let inherits = vec!["myAnimationContextTwo".to_string()];
        let _ = crealion.process_animations("myAnimationContextTwo", &inherits, mock_animations());

        let result =
            STYLITRON
                .get("animations")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Animation(animations_definitions) => {
                        animations_definitions.get("myAnimationContextTwo").cloned()
                    }
                    _ => None,
                });

        assert!(result.is_some());
        let animations = result.unwrap();
        let expected_animations = transform_animations("myAnimationContextTwo");

        // Verify that the context was updated correctly.
        assert_eq!(animations, expected_animations);
    }

    #[test]
    fn test_apply_animations_to_new_context() {
        let (sender, _) = mpsc::unbounded_channel();

        // Ensure no existing context in the STYLITRON AST.
        let initial_data = IndexMap::new();
        STYLITRON.insert("animations".to_string(), Stylitron::Animation(initial_data));

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let inherits = vec!["myAnimationContextThree".to_string()];
        let _ =
            crealion.process_animations("myAnimationContextThree", &inherits, mock_animations());

        let result =
            STYLITRON
                .get("animations")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Animation(animations_definitions) => animations_definitions
                        .get("myAnimationContextThree")
                        .cloned(),
                    _ => None,
                });

        assert!(result.is_some());
        let animations = result.unwrap();
        let expected_animations = transform_animations("myAnimationContextThree");

        // Verify that the new context was added with correct animations.
        assert_eq!(animations, expected_animations);
    }

    #[test]
    fn test_apply_animations_with_empty_animations_data() {
        let (sender, _) = mpsc::unbounded_channel();

        let crealion = Crealion::new(
            sender,
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let empty_animations: IndexMap<String, NenyrAnimation> = IndexMap::new();
        let inherits = vec!["emptyAnimationContext".to_string()];
        let _ = crealion.process_animations(
            "emptyAnimationContext",
            &inherits,
            empty_animations.clone(),
        );

        let result =
            STYLITRON
                .get("animations")
                .and_then(|stylitron_data| match &*stylitron_data {
                    Stylitron::Animation(animations_definitions) => {
                        animations_definitions.get("emptyAnimationContext").cloned()
                    }
                    _ => None,
                });

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_apply_animations_no_animations_section() {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let (sender, mut receiver) = mpsc::unbounded_channel();

        // Simulate an empty STYLITRON AST to trigger an error.
        STYLITRON.remove("animations");

        let crealion = Crealion::new(
            sender.clone(),
            NenyrAst::CentralContext(CentralContext::new()),
            "".to_string(),
        );

        let inherits = vec!["noAnimationsSection".to_string()];
        let _ = crealion.process_animations("noAnimationsSection", &inherits, mock_animations());

        // Verify that an error notification was sent.
        if let Some(notification) = receiver.recv().await {
            if let ShellscapeAlerts::GaladrielError {
                start_time: _,
                error,
            } = notification
            {
                assert_eq!(
                    error.get_message(),
                    "Failed to access the animations section in STYLITRON AST".to_string()
                );
            }
        } else {
            panic!("Expected an error notification, but none was received.");
        }
    }
}
