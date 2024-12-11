use indexmap::IndexMap;

use crate::{asts::CLASTRACK, types::Clastrack};

/// Serializes the tracking data from three different contexts (`central`, `layouts`, and `modules`).
/// It collects the tracking maps from each context and formats them into a JSON-like string.
///
/// # Returns
/// Returns a `String` representing the serialized tracking data from all contexts.
///
/// The string is formatted as a JSON object with keys "central", "layouts", and "modules", each containing
/// the corresponding tracking map data.
pub fn serialize_classes_tracking() -> String {
    tracing::info!("Starting to serialize class tracking data.");

    // Fetch the tracking data from the central context
    let central_map = get_tracking_map_from_central();
    // Fetch the tracking data from the layouts context
    let layouts_map = get_tracking_map_from_layouts();
    // Fetch the tracking data from the modules context
    let modules_map = get_tracking_map_from_modules();

    // Return the serialized string in a JSON-like format containing all three tracking maps
    format!(
        "{{ \"central\": {:?}, \"layouts\": {:?}, \"modules\": {:?}}}",
        central_map, layouts_map, modules_map
    )
}

/// Retrieves the tracking map for the "central" context. If no valid data is found in `CLASTRACK` for
/// the "central" key, an empty `IndexMap` is returned.
///
/// # Returns
/// - `IndexMap<String, String>`: A map representing the tracking data from the central context.
fn get_tracking_map_from_central() -> IndexMap<String, String> {
    tracing::info!("Fetching tracking map for central context.");

    match CLASTRACK.get("central") {
        Some(clastrack_data) => match &*clastrack_data {
            Clastrack::Central(ref central_node) => {
                return central_node.to_owned();
            }
            _ => {}
        },
        None => {}
    }

    IndexMap::new()
}

/// Retrieves the tracking map for the "layouts" context. If no valid data is found in `CLASTRACK` for
/// the "layouts" key, an empty `IndexMap` is returned.
///
/// # Returns
/// - `IndexMap<String, IndexMap<String, String>>`: A map representing the tracking data from the layouts context.
fn get_tracking_map_from_layouts() -> IndexMap<String, IndexMap<String, String>> {
    tracing::info!("Fetching tracking map for layout contexts.");

    match CLASTRACK.get("layouts") {
        Some(clastrack_data) => match &*clastrack_data {
            Clastrack::Layouts(ref layouts_node) => {
                return layouts_node.to_owned();
            }
            _ => {}
        },
        None => {}
    }

    IndexMap::new()
}

/// Retrieves the tracking map for the "modules" context. If no valid data is found in `CLASTRACK` for
/// the "modules" key, an empty `IndexMap` is returned.
///
/// # Returns
/// - `IndexMap<String, IndexMap<String, String>>`: A map representing the tracking data from the modules context.
fn get_tracking_map_from_modules() -> IndexMap<String, IndexMap<String, String>> {
    tracing::info!("Fetching tracking map for module contexts.");

    match CLASTRACK.get("modules") {
        Some(clastrack_data) => match &*clastrack_data {
            Clastrack::Modules(ref modules_node) => {
                return modules_node.to_owned();
            }
            _ => {}
        },
        None => {}
    }

    IndexMap::new()
}
