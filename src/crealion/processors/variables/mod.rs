use lazy_static::lazy_static;
use regex::Regex;

use crate::{asts::STYLITRON, types::Stylitron};

/// Schema for themes, defining the available theme types.
const THEMES_SCHEMA: &[&str] = &["light", "dark"];

lazy_static! {
    /// Lazy-static regex to match variables within the format `${variable_name}`.
    static ref RE: Regex = Regex::new(r"\$\{(.*?)\}").unwrap();
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum VariablesOption<T> {
    Some(T),
    Unresolved(T),
}

#[allow(dead_code)]
impl<T> VariablesOption<T> {
    pub fn is_some(&self) -> bool {
        match self {
            VariablesOption::Some(_) => true,
            _ => false,
        }
    }
}

/// Resolves variables from a string input by attempting to resolve it from various sources
/// such as variables and animation nodes. It also handles the possibility of unresolved variables.
///
/// This function uses regular expressions to extract variable names from the input string and
/// attempts to resolve each one from the available contexts. If a variable cannot be resolved,
/// it is returned as is. If the variable is found within the animation node context, the resolved
/// name is returned. Otherwise, the function returns the resolved string with all variables
/// replaced by their resolved values.
///
/// # Parameters
/// - `input`: A `String` representing the input that may contain variables to be resolved.
/// - `use_animation`: A `bool` indicating whether to attempt resolving variables from animation nodes.
/// - `inherited_contexts`: A reference to a `Vec<String>` containing the inherited contexts to search for variables.
///
/// # Returns
/// - `Option<String>`: The resolved string with variables replaced, or `None` if a variable could not be resolved.
pub fn resolve_variable_from_str(
    input: String,
    use_animation: bool,
    inherited_contexts: &Vec<String>,
) -> VariablesOption<String> {
    tracing::info!("Starting to resolve variable from input: {}", input);

    // Variable to hold the unresolved variable name, if any.
    let mut variable_not_found: Option<String> = None;

    // Process the input string using the regex RE to replace captured variable names.
    let resolved_input = RE
        .replace_all(&input, |caps: &regex::Captures| {
            // Extract the variable name from the regex capture group.
            let relative_name = &caps[1].to_string();

            tracing::info!("Processing capture: {}", relative_name);

            // First, attempt to resolve the variable from the themes node.
            match resolve_from_themes_node(relative_name, inherited_contexts) {
                Some(resolved_name) => {
                    tracing::info!("Resolved '{}' from variables node", resolved_name);

                    return format!("var({resolved_name})");
                }
                None => {}
            }

            // Second attempt to resolve the variable from the variables node.
            match resolve_from_variables_node(relative_name, inherited_contexts) {
                Some(resolved_name) => {
                    tracing::info!("Resolved '{}' from variables node", resolved_name);

                    return format!("var({resolved_name})");
                }
                None => {}
            }

            // If using animation, attempt to resolve the variable from animation nodes.
            if use_animation {
                match resolve_from_animations_node(relative_name, inherited_contexts) {
                    Some(resolved_name) => {
                        tracing::info!("Resolved '{}' from animations node", resolved_name);

                        return resolved_name;
                    }
                    None => {}
                }
            }

            // If variable was not resolved, store its name and return the original.
            variable_not_found = Some(relative_name.to_owned());

            return relative_name.to_owned();
        })
        .to_string();

    // If a variable could not be resolved, log a warning and return `None`.
    match variable_not_found {
        Some(name) => {
            tracing::warn!("Could not resolve variable '{}' from any source", name);

            return VariablesOption::Unresolved(name);
        }
        // If no variables were unresolved, return the fully resolved input.
        None => VariablesOption::Some(resolved_input),
    }
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

#[cfg(test)]
mod processors_tests {
    use indexmap::IndexMap;

    use crate::{
        asts::STYLITRON, crealion::processors::variables::VariablesOption, types::Stylitron,
    };

    use super::resolve_variable_from_str;

    fn mock_variables() {
        let map = IndexMap::from([
            (
                "justAnotherContext".to_string(),
                IndexMap::from([
                    (
                        "varNameOne".to_string(),
                        vec!["--jd5dj3h4e7".to_string(), "#000000".to_string()],
                    ),
                    (
                        "varNameTwo".to_string(),
                        vec!["--o34s54e83e".to_string(), "#FFFFFF".to_string()],
                    ),
                ]),
            ),
            (
                "oneExtraContext".to_string(),
                IndexMap::from([
                    (
                        "varNameOne".to_string(),
                        vec!["--jd5dj3h4e7".to_string(), "#000000".to_string()],
                    ),
                    (
                        "varNameTwo".to_string(),
                        vec!["--o34s54e83e".to_string(), "#FFFFFF".to_string()],
                    ),
                    (
                        "varNameThree".to_string(),
                        vec!["--y7637dj35e".to_string(), "rgb(0, 255, 0)".to_string()],
                    ),
                ]),
            ),
        ]);

        STYLITRON.insert("variables".to_string(), Stylitron::Variables(map));
    }

    fn mock_themes() {
        let map = IndexMap::from([
            (
                "justAnotherContext".to_string(),
                IndexMap::from([
                    (
                        "light".to_string(),
                        IndexMap::from([(
                            "themesVarOne".to_string(),
                            vec!["--jd5dj3h4e7".to_string(), "rgb(255, 0, 255)".to_string()],
                        )]),
                    ),
                    (
                        "dark".to_string(),
                        IndexMap::from([(
                            "themesVarOne".to_string(),
                            vec!["--jd5dj3h4e7".to_string(), "rgb(0, 255, 0)".to_string()],
                        )]),
                    ),
                ]),
            ),
            (
                "oneExtraContext".to_string(),
                IndexMap::from([
                    (
                        "light".to_string(),
                        IndexMap::from([
                            (
                                "themesVarOne".to_string(),
                                vec!["--jd5dj3h4e7".to_string(), "#FFFFFF".to_string()],
                            ),
                            (
                                "themesVarTwo".to_string(),
                                vec!["--ywd5drj73h".to_string(), "#000000".to_string()],
                            ),
                        ]),
                    ),
                    (
                        "dark".to_string(),
                        IndexMap::from([
                            (
                                "themesVarOne".to_string(),
                                vec!["--jd5dj3h4e7".to_string(), "#000000".to_string()],
                            ),
                            (
                                "themesVarTwo".to_string(),
                                vec!["--ywd5drj73h".to_string(), "#FFFFFF".to_string()],
                            ),
                        ]),
                    ),
                ]),
            ),
        ]);

        STYLITRON.insert("themes".to_string(), Stylitron::Themes(map));
    }

    fn mock_animations() {
        let map = IndexMap::from([
            (
                "animationsContextOne".to_string(),
                IndexMap::from([(
                    "myAnimation".to_string(),
                    IndexMap::from([(
                        "g39jd4dkh3k7".to_string(),
                        IndexMap::from([
                            (
                                "0%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "blue".to_string(),
                                )]),
                            ),
                            (
                                "50%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "green".to_string(),
                                )]),
                            ),
                            (
                                "100%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "red".to_string(),
                                )]),
                            ),
                        ]),
                    )]),
                )]),
            ),
            (
                "animationsContextTwo".to_string(),
                IndexMap::from([(
                    "simpleAnimation".to_string(),
                    IndexMap::from([(
                        "g4duf74dju3".to_string(),
                        IndexMap::from([
                            (
                                "0%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "blue".to_string(),
                                )]),
                            ),
                            (
                                "50%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "green".to_string(),
                                )]),
                            ),
                            (
                                "100%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "red".to_string(),
                                )]),
                            ),
                        ]),
                    )]),
                )]),
            ),
        ]);

        STYLITRON.insert("animations".to_string(), Stylitron::Animation(map));
    }

    #[test]
    fn variables_exists_in_variable_node() {
        mock_variables();

        let input = "${varNameThree} ${varNameOne} ${varNameTwo}".to_string();
        let inherits = vec![
            "justAnotherContext".to_string(),
            "oneExtraContext".to_string(),
        ];

        let resolved_input = resolve_variable_from_str(input, false, &inherits);
        let expected_result = "var(--y7637dj35e) var(--jd5dj3h4e7) var(--o34s54e83e)".to_string();

        assert!(resolved_input.is_some());
        assert_eq!(resolved_input, VariablesOption::Some(expected_result));
    }

    #[test]
    fn variables_exists_in_themes_node() {
        mock_themes();

        let input = "${themesVarOne} ${themesVarTwo}".to_string();
        let inherits = vec![
            "justAnotherContext".to_string(),
            "oneExtraContext".to_string(),
        ];

        let resolved_input = resolve_variable_from_str(input, false, &inherits);
        let expected_result = "var(--jd5dj3h4e7) var(--ywd5drj73h)".to_string();

        assert!(resolved_input.is_some());
        assert_eq!(resolved_input, VariablesOption::Some(expected_result));
    }

    #[test]
    fn animations_exists_in_animation_node() {
        mock_animations();

        let input = "${simpleAnimation} ${myAnimation}".to_string();
        let inherits = vec![
            "animationsContextOne".to_string(),
            "animationsContextTwo".to_string(),
        ];

        let resolved_input = resolve_variable_from_str(input, true, &inherits);
        let expected_result = "g4duf74dju3 g39jd4dkh3k7".to_string();

        assert!(resolved_input.is_some());
        assert_eq!(resolved_input, VariablesOption::Some(expected_result));
    }
}
