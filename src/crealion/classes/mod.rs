use indexmap::IndexMap;
use nenyr::types::class::NenyrStyleClass;

use super::Crealion;

impl Crealion {
    pub async fn process_classes(
        &self,
        _context_name: String,
        _inherited_contexts: Vec<String>,
        _classes_data: IndexMap<String, NenyrStyleClass>,
    ) {
        let _sender = self.sender.clone();
    }
}
