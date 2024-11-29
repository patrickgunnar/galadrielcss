use indexmap::IndexMap;
use tokio::task::JoinHandle;

use super::Crealion;

impl Crealion {
    pub fn process_breakpoints(
        &self,
        mobile_data: Option<IndexMap<String, String>>,
        desktop_data: Option<IndexMap<String, String>>,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();

        tokio::task::spawn_blocking(move || {})
    }
}
