use std::path::PathBuf;

use chrono::Local;
use nenyr::NenyrParser;
use tokio::sync::broadcast;

use crate::{
    crealion::Crealion, error::GaladrielError, events::GaladrielAlerts,
    utils::resilient_reader::resilient_reader, GaladrielResult,
};

#[derive(Clone, Debug)]
pub struct Formera {
    path: PathBuf,
    auto_naming: bool,
    sender: broadcast::Sender<GaladrielAlerts>,
}

impl Formera {
    pub fn new(
        path: PathBuf,
        auto_naming: bool,
        sender: broadcast::Sender<GaladrielAlerts>,
    ) -> Self {
        Self {
            path,
            auto_naming,
            sender,
        }
    }

    pub async fn start(
        &mut self,
        nenyr_parser: &mut NenyrParser,
    ) -> GaladrielResult<Option<Vec<String>>> {
        let start_time = Local::now();
        let raw_content = resilient_reader(&self.path).await?;
        let raw_content = self.process_names_injection(raw_content)?;

        let path = self.path.to_string_lossy().to_string();

        let parsed_ast = nenyr_parser
            .parse(raw_content, path.to_owned())
            .map_err(|err| GaladrielError::raise_nenyr_error(start_time, err))?;

        let mut crealion = Crealion::new(self.sender.clone(), parsed_ast, path.into());

        crealion.create().await
    }

    pub fn process_names_injection(&self, raw_content: String) -> GaladrielResult<String> {
        if self.auto_naming {
            return Ok(raw_content);
        }

        Ok(raw_content)
    }
}
