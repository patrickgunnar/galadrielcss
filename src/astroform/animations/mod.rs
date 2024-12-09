use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{
    asts::STYLITRON,
    error::{ErrorAction, ErrorKind},
    types::Stylitron,
};

use super::Astroform;

impl Astroform {
    /// Transforms animation definitions from the AST into CSS keyframe rules.
    ///
    /// # Returns
    /// - A `JoinHandle` wrapping a `String` containing all formatted CSS animations.
    pub fn transform_animations(&self) -> JoinHandle<String> {
        let palantir_sender = self.palantir_sender.clone();
        let tab = self.tab.to_owned();
        let space = self.space.to_owned();
        let newline = self.newline.to_owned();
        let is_minified = self.is_minified;

        // Spawn a blocking task to process the animations in a separate thread.
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting animation transformation process.");

            let mut formatted_css_animations: Vec<String> = vec![];

            // Retrieve the animations map from the global STYLITRON AST.
            let animations_map = match STYLITRON.get("animations") {
                Some(stylitron_data) => match &*stylitron_data {
                    Stylitron::Animation(ref animations) => animations.to_owned(),
                    _ => return String::new(),
                },
                None => {
                    Self::send_palantir_error_notification(
                        ErrorKind::AccessDeniedToStylitronAST,
                        ErrorAction::Restart,
                        "Failed to access the animations section in STYLITRON AST",
                        palantir_sender.clone(),
                    );

                    return String::new();
                }
            };

            // Iterate over each animation context in the retrieved animations map.
            for (context_name, context_animations) in animations_map {
                tracing::info!("Processing animations for context: '{}'", context_name);

                if !is_minified {
                    // Resolve the context name and include a comment in the CSS for non-minified output.
                    let context_name = Self::resolve_context_name(&context_name);

                    formatted_css_animations.push(format!(
                        "/* Animation sourced from the '{}' context */",
                        context_name
                    ));
                }

                // Transform the animations for the current context.
                let animations =
                    Self::transform_context_animation(&tab, &space, &newline, context_animations);

                formatted_css_animations.push(animations);
            }

            tracing::info!("Animation transformation completed.");

            // Combine all animations into a single string separated by newlines.
            formatted_css_animations.join(&newline)
        })
    }

    /// Transforms animations within a specific context into CSS keyframe rules.
    ///
    /// # Arguments
    /// - `tab`: Indentation string (e.g., tabs or spaces).
    /// - `space`: A space character for formatting.
    /// - `newline`: A newline character for formatting.
    /// - `context_animations`: The animation map for a specific context.
    ///
    /// # Returns
    /// - A `String` containing formatted CSS keyframe rules for the context.
    fn transform_context_animation(
        tab: &str,
        space: &str,
        newline: &str,
        context_animations: IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
        >,
    ) -> String {
        tracing::debug!("Starting transformation of context animations.");

        let mut keyframe_rules: Vec<String> = vec![];

        // Iterate over each animation definition in the context.
        context_animations.iter().for_each(|(_, animation_map)| {
            animation_map
                .iter()
                .for_each(|(unique_animation_name, keyframes)| {
                    tracing::debug!(
                        "Transforming keyframes for animation: '{}'",
                        unique_animation_name
                    );

                    // Transform the keyframes for the current animation into CSS rules.
                    let animation_stops = Self::transform_keyframes(tab, space, newline, keyframes);

                    keyframe_rules.push(format!(
                        "@keyframes {}{}{{{}{}{}}}",
                        unique_animation_name, space, newline, animation_stops, newline
                    ));
                });
        });

        tracing::debug!("Completed transformation of context animations.");

        // Combine all keyframe rules into a single string separated by newlines.
        keyframe_rules.join(newline)
    }

    /// Transforms keyframe definitions into CSS rules.
    ///
    /// # Arguments
    /// - `tab`: Indentation string (e.g., tabs or spaces).
    /// - `space`: A space character for formatting.
    /// - `newline`: A newline character for formatting.
    /// - `keyframes`: A map of keyframe stops to their properties and values.
    ///
    /// # Returns
    /// - A `String` containing all formatted CSS keyframe rules.
    fn transform_keyframes(
        tab: &str,
        space: &str,
        newline: &str,
        keyframes: &IndexMap<String, IndexMap<String, String>>,
    ) -> String {
        tracing::debug!("Starting transformation of keyframes.");

        let mut stops_rules: Vec<String> = vec![];

        // Iterate over each keyframe stop and its associated properties.
        keyframes.iter().for_each(|(stops, properties)| {
            tracing::debug!("Processing keyframe stop: '{}'", stops);

            let mut properties_rules: Vec<String> = vec![];

            // Format each property-value pair for the current keyframe stop.
            properties.iter().for_each(|(property, value)| {
                let formatted_prop = format!("{}{}:{}{}", tab.repeat(2), property, space, value);

                properties_rules.push(formatted_prop);
            });

            // Combine the formatted properties into a CSS block for the keyframe stop.
            let formatted_stop = format!(
                "{}{}{}{{{}{}{}{}}}",
                tab,
                stops,
                space,
                newline,
                properties_rules.join(&format!(";{}", newline)),
                newline,
                tab
            );

            stops_rules.push(formatted_stop);
        });

        tracing::debug!("Completed transformation of keyframes.");

        // Combine all keyframe stops into a single string separated by newlines.
        stops_rules.join(newline)
    }
}
