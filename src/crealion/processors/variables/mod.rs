use lazy_static::lazy_static;
use regex::Regex;

use crate::{asts::STYLITRON, types::Stylitron};

/// Schema for themes, defining the available theme types.
const THEMES_SCHEMA: &[&str] = &["light", "dark"];

lazy_static! {
    /// Lazy-static regex to match variables within the format `${variable_name}`.
    static ref RE: Regex = Regex::new(r"\$\{(.*?)\}").unwrap();
}

/// Resolves a variable from a given string input by searching through context-based variables,
/// animations, and themes. If no match is found, it returns `None`.
///
/// # Parameters
/// - `input`: The input string containing potential variables to resolve.
/// - `use_animation`: A boolean flag indicating whether animation nodes should be checked.
/// - `inherited_contexts`: A vector of inherited contexts to search for variable definitions.
///
/// # Returns
/// - `Some(String)` if the input string has resolvable variables.
/// - `None` if no variables can be resolved.
pub fn resolve_variable_from_str(
    input: &str,
    use_animation: bool,
    inherited_contexts: &Vec<String>,
) -> Option<String> {
    tracing::info!("Starting to resolve variable from input: {}", input);

    // Copy the input string for resolution.
    let mut resolved_input = input.to_string();
    // Offset index to adjust for replacement shifts in the string.
    let mut offset_index = 0;

    // Iterate over all regex captures in the input string.
    for capture in RE.captures_iter(input) {
        // Extract the variable name from the capture group.
        let relative_name = &capture[1].to_string();

        tracing::info!("Processing capture: {}", relative_name);

        // Attempt to resolve the variable from context-based variables.
        if let Some(resolved_name) = resolve_from_variables_node(relative_name, inherited_contexts)
        {
            tracing::info!("Resolved '{}' from variables node", resolved_name);

            apply_resolve_variable_to_input(
                &mut resolved_input,
                &mut offset_index,
                &format!("var({resolved_name})"),
                &capture,
            );

            continue;
        }

        // If animations are allowed, attempt to resolve from animation nodes.
        if use_animation {
            if let Some(resolved_name) =
                resolve_from_animations_node(relative_name, inherited_contexts)
            {
                tracing::info!("Resolved '{}' from animations node", resolved_name);

                apply_resolve_variable_to_input(
                    &mut resolved_input,
                    &mut offset_index,
                    &resolved_name,
                    &capture,
                );

                continue;
            }
        }

        // Attempt to resolve the variable from themes.
        if let Some(resolved_name) = resolve_from_themes_node(relative_name, inherited_contexts) {
            tracing::info!("Resolved '{}' from themes node", resolved_name);

            apply_resolve_variable_to_input(
                &mut resolved_input,
                &mut offset_index,
                &format!("var({resolved_name})"),
                &capture,
            );

            continue;
        }

        tracing::warn!(
            "Could not resolve variable '{}' from any source",
            relative_name
        );

        return None;
    }

    tracing::info!(
        "Successfully resolved all variables, final input: {}",
        resolved_input
    );

    Some(resolved_input)
}

/// Replaces a captured variable in the input string with a resolved value,
/// adjusting for positional offsets during replacements.
///
/// # Parameters
/// - `resolved_input`: The string being modified.
/// - `offset_index`: The current positional offset in the string.
/// - `replacement_value`: The resolved value to replace the variable.
/// - `capture`: The regex capture containing the variable match.
fn apply_resolve_variable_to_input(
    resolved_input: &mut String,
    offset_index: &mut usize,
    replacement_value: &str,
    capture: &regex::Captures<'_>,
) {
    // Compute the positions of the variable within the string.
    let (start_pos, end_pos) = get_positions(capture);

    tracing::info!(
        "Replacing variable in input at positions {}..{}",
        start_pos,
        end_pos
    );

    // Replace the variable with its resolved value.
    resolved_input.replace_range(
        start_pos.saturating_add(*offset_index)..end_pos.saturating_add(*offset_index),
        replacement_value,
    );

    // Adjust the offset index based on the replacement length.
    let adjustment = if end_pos <= start_pos {
        end_pos.saturating_sub(start_pos)
    } else {
        0
    };

    *offset_index += replacement_value.len().saturating_sub(adjustment);
}

/// Retrieves the start and end positions of a regex capture group.
///
/// # Parameters
/// - `capture`: The regex capture to extract positions from.
///
/// # Returns
/// - `(usize, usize)` tuple representing the start and end positions.
fn get_positions(capture: &regex::Captures<'_>) -> (usize, usize) {
    // Retrieve the positions for the first capture group (index 0).
    capture
        .get(0)
        .and_then(|cap| Some((cap.start(), cap.end())))
        .unwrap_or((0, 0))
}

/// Resolves a variable from the "variables" node in the STYLITRON.
///
/// # Parameters
/// - `relative_name`: The variable name to resolve.
/// - `inherited_contexts`: Contexts to search for the variable.
///
/// # Returns
/// - `Some(String)` if the variable is resolved.
/// - `None` if the variable cannot be found.
fn resolve_from_variables_node(
    relative_name: &str,
    inherited_contexts: &Vec<String>,
) -> Option<String> {
    STYLITRON
        .get("variables")
        .and_then(|stylitron_data| match &*stylitron_data {
            Stylitron::Variables(ref variables_definitions) => {
                inherited_contexts.iter().find_map(|context_name| {
                    variables_definitions
                        .get(context_name)
                        .and_then(|context_variables| {
                            context_variables
                                .get(relative_name)
                                .and_then(|variable_entry| Some(variable_entry[0].to_owned()))
                        })
                })
            }
            _ => None,
        })
}

/// Resolves a variable from the "animations" node in the STYLITRON.
///
/// # Parameters
/// - `relative_name`: The animation name to resolve.
/// - `inherited_contexts`: Contexts to search for the animation.
///
/// # Returns
/// - `Some(String)` if the animation is resolved.
/// - `None` if the animation cannot be found.
fn resolve_from_animations_node(
    relative_name: &str,
    inherited_contexts: &Vec<String>,
) -> Option<String> {
    STYLITRON
        .get("animations")
        .and_then(|stylitron_data| match &*stylitron_data {
            Stylitron::Animation(ref animation_definitions) => {
                inherited_contexts.iter().find_map(|context_name| {
                    animation_definitions
                        .get(context_name)
                        .and_then(|context_animations| {
                            context_animations
                                .get(relative_name)
                                .and_then(|animation_entry| {
                                    animation_entry
                                        .get_index(0)
                                        .and_then(|(unique_name, _)| Some(unique_name.to_owned()))
                                })
                        })
                })
            }
            _ => None,
        })
}

/// Resolves a variable from the "themes" node in the STYLITRON.
///
/// # Parameters
/// - `relative_name`: The theme variable name to resolve.
/// - `inherited_contexts`: Contexts to search for the theme variable.
///
/// # Returns
/// - `Some(String)` if the theme variable is resolved.
/// - `None` if the theme variable cannot be found.
fn resolve_from_themes_node(
    relative_name: &str,
    inherited_contexts: &Vec<String>,
) -> Option<String> {
    STYLITRON
        .get("themes")
        .and_then(|stylitron_data| match &*stylitron_data {
            Stylitron::Themes(ref themes_definitions) => {
                inherited_contexts.iter().find_map(|context_name| {
                    themes_definitions
                        .get(context_name)
                        .and_then(|context_themes| {
                            THEMES_SCHEMA.iter().find_map(|schema_type| {
                                context_themes.get(schema_type.to_owned()).and_then(
                                    |schema_variables| {
                                        schema_variables.get(relative_name).and_then(
                                            |variable_entry| Some(variable_entry[0].to_owned()),
                                        )
                                    },
                                )
                            })
                        })
                })
            }
            _ => None,
        })
}
