use crate::{asts::GATEKEEPER, gatekeeper::remove_path_from_gatekeeper};

use super::Crealion;

impl Crealion {
    /// Registers a module-path relationship for a given layout name.
    ///
    /// This function associates a layout name with a module path. It removes the module path
    /// from the `GATEKEEPER` registry (if already present) and then inserts it into the entry for
    /// the specified layout name. The `GATEKEEPER` registry maintains a list of module paths for
    /// each layout name.
    ///
    /// # Arguments
    ///
    /// * `layout_name` - The name of the layout to associate with the module path.
    /// * `module_path` - The path to the module to register for the given layout.
    ///
    /// # Behavior
    ///
    /// - If the layout name already exists in `GATEKEEPER`, the module path will be added to
    ///   the existing list of module paths.
    /// - If the layout name does not exist in `GATEKEEPER`, a new entry is created with the
    ///   module path.
    pub fn register_module_layout_relationship(&self, layout_name: String, module_path: String) {
        tracing::trace!(
            "Removing module path '{}' from the GATEKEEPER registry, if present.",
            module_path
        );

        // Remove the module path from the gatekeeper registry before registering it for the layout.
        remove_path_from_gatekeeper(&module_path);

        tracing::debug!(
            "Inserting module path '{}' into the GATEKEEPER registry for layout '{}'.",
            module_path,
            layout_name
        );

        // Insert the module path into the GATEKEEPER registry for the given layout name.
        // If the layout name doesn't exist, it creates a new entry with a Vec::new.
        GATEKEEPER
            .entry(layout_name)
            .or_insert_with(Vec::new)
            .push(module_path);
    }

    /// Retrieves the list of module paths associated with a given layout name.
    ///
    /// This function looks up the `GATEKEEPER` registry to retrieve the list of module paths
    /// associated with the specified layout name. If no entry exists for the layout name,
    /// it returns `None`.
    ///
    /// # Arguments
    /// * `layout_name` - The name of the layout whose associated module paths should be retrieved.
    ///
    /// # Returns
    /// - `Some(Vec<String>)` - A vector of module paths associated with the given layout name.
    /// - `None` - If the layout name is not found in the `GATEKEEPER` registry.
    pub fn retrieve_module_layout_relationship(&self, layout_name: &str) -> Option<Vec<String>> {
        tracing::debug!("Retrieving module paths for layout '{}'.", layout_name);

        // Retrieve the entry for the layout name from the GATEKEEPER registry and clone the vector.
        GATEKEEPER
            .get(layout_name)
            .map(|entry| entry.value().to_vec())
    }
}
