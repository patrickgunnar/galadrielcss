use crate::crealion::utils::generate_prefix::generate_prefix;

/// Generates a variable or animation name based on the current context and name.
///
/// This function creates a standardized name for a variable or animation by incorporating
/// the current context and name, along with a prefix that differentiates between variables
/// and animations. The length of the generated prefix depends on whether the name is for
/// a variable or an animation.
///
/// # Parameters
///
/// - `current_context`: A string slice representing the current context in which the name is used.
/// - `current_name`: A string slice representing the base name to be used.
/// - `is_variable`: A boolean flag indicating whether the name is for a variable (true) or an animation (false).
///
/// # Returns
///
/// Returns a `String` containing the generated name for the variable or animation.
pub fn generates_variable_or_animation_name(
    current_context: &str,
    current_name: &str,
    is_variable: bool,
) -> String {
    // Determine the size of the prefix based on whether the name is for a variable or an animation.
    let current_size = if is_variable { 10 } else { 14 };
    // Create a context-based name by combining the context and the base name.
    let context_based_name = format!("{}-{}", current_context, current_name);
    // Generate a prefix for the name based on the context-based name and size.
    let generated_name = generate_prefix(&context_based_name, false, current_size);
    // Format and return the name with the appropriate prefix based on the type (variable or animation).
    let prefixed_var = if is_variable { "--" } else { "" };

    format!("{}g{}", prefixed_var, generated_name)
}
