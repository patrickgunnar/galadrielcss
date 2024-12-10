use indexmap::IndexMap;

use crate::{
    asts::CLASTRACK, intaker::intaker_contains_context_name::intaker_contains_context_name,
    lothlorien::request::ContextType, types::Clastrack,
};

/// Retrieves a utility class names based on the provided context type, context name, and class name.
///
/// # Arguments
/// - `context_type`: Specifies the type of context (Central, Layout, or Module).
/// - `context_name`: Optional name of the specific context (e.g., layout or module).
/// - `class_name`: The name of the Nenyr class being searched.
///
/// # Returns
/// - A `String` containing the fully resolved utility class names, or an empty string if no match is found.
pub fn get_utility_class_names(
    context_type: ContextType,
    context_name: Option<String>,
    class_name: String,
) -> String {
    match context_type {
        // Handles the Central context type by fetching class names from the central context.
        ContextType::Central => {
            return get_class_names_from_central(class_name);
        }
        // Handles the Layout context type. If a valid context name exists and matches known contexts,
        // retrieves class names from layouts.
        ContextType::Layout => {
            if let Some(layout_name) = context_name {
                if intaker_contains_context_name(&layout_name) {
                    return get_class_names_from_layouts(layout_name, class_name);
                }
            }
        }
        // Handles the Module context type. Similar to Layout, but fetches from module-specific contexts.
        ContextType::Module => {
            if let Some(module_name) = context_name {
                if intaker_contains_context_name(&module_name) {
                    return get_class_names_from_modules(module_name, class_name);
                }
            }
        }
    }

    // Returns an empty string if no valid match is found for the provided context type and name.
    String::new()
}

/// Fetches class names from the central context.
///
/// # Arguments
/// - `class_name`: The name of the Nenyr class.
///
/// # Returns
/// - A `String` containing the class names resolved from the central context or an empty string.
fn get_class_names_from_central(class_name: String) -> String {
    if let Some(clastrack_data) = CLASTRACK.get("central") {
        match &*clastrack_data {
            Clastrack::Central(ref central_maps) => {
                return central_maps
                    .get(&class_name)
                    .unwrap_or(&"".to_string())
                    .replace('\\', ""); // Removes escape characters for the returned class name.
            }
            _ => {}
        }
    }

    String::new()
}

/// Fetches class names from layouts using the provided context and class names.
///
/// # Arguments
/// - `context_name`: The name of the layout context.
/// - `class_name`: The name of the Nenyr class.
///
/// # Returns
/// - A `String` containing the resolved class names or an empty string.
fn get_class_names_from_layouts(context_name: String, class_name: String) -> String {
    if let Some(clastrack_data) = CLASTRACK.get("layouts") {
        match &*clastrack_data {
            Clastrack::Layouts(ref layouts_maps) => {
                return get_from_named_context(context_name, class_name, layouts_maps)
                    .unwrap_or(String::new());
            }
            _ => {}
        }
    }

    String::new()
}

/// Fetches class names from modules using the provided context and class names.
///
/// # Arguments
/// - `context_name`: The name of the module context.
/// - `class_name`: The name of the Nenyr class.
///
/// # Returns
/// - A `String` containing the resolved class names or an empty string.
fn get_class_names_from_modules(context_name: String, class_name: String) -> String {
    if let Some(clastrack_data) = CLASTRACK.get("modules") {
        match &*clastrack_data {
            Clastrack::Modules(ref modules_maps) => {
                return get_from_named_context(context_name, class_name, modules_maps)
                    .unwrap_or(String::new());
            }
            _ => {}
        }
    }

    String::new()
}

/// Resolves class names from a specific named context within a provided map.
///
/// # Arguments
/// - `context_name`: The name of the context (e.g., layout or module).
/// - `class_name`: The name of the Nenyr class.
/// - `maps`: A reference to the map containing context-to-class mappings.
///
/// # Returns
/// - An `Option<String>` containing the resolved class names or `None` if not found.
fn get_from_named_context(
    context_name: String,
    class_name: String,
    maps: &IndexMap<String, IndexMap<String, String>>,
) -> Option<String> {
    maps.get(&context_name).and_then(|context_maps| {
        context_maps
            .get(&class_name)
            .and_then(|entry| Some(entry.replace('\\', ""))) // Removes escape characters if found.
    })
}
