use nenyr::types::variables::NenyrVariables;
use tokio::task::JoinHandle;

use super::Crealion;

impl Crealion {
    pub fn process_themes(
        &self,
        context_name: String,
        light_data: Option<NenyrVariables>,
        dark_data: Option<NenyrVariables>,
    ) -> JoinHandle<()> {
        let sender = self.sender.clone();

        tokio::spawn(async move {})
    }
}
