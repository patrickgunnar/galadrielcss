use std::path::PathBuf;

use chrono::Local;
use nenyr::NenyrParser;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    crealion::Crealion, error::GaladrielError, shellscape::alerts::ShellscapeAlerts,
    utils::resilient_reader::resilient_reader, GaladrielResult,
};

#[derive(Clone, Debug)]
pub struct Formera {
    path: PathBuf,
    auto_naming: bool,
    sender: UnboundedSender<ShellscapeAlerts>,
}

impl Formera {
    pub fn new(
        path: PathBuf,
        auto_naming: bool,
        sender: UnboundedSender<ShellscapeAlerts>,
    ) -> Self {
        Self {
            path,
            auto_naming,
            sender,
        }
    }

    pub async fn start(&mut self) -> GaladrielResult<()> {
        let start_time = Local::now();
        let raw_content = resilient_reader(&self.path).await?;
        let raw_content = self.process_names_injection(raw_content)?;

        let path = &self.path.to_string_lossy().to_string();
        let mut parser = NenyrParser::new(&raw_content, path);

        let parsed_ast = parser
            .parse()
            .map_err(|err| GaladrielError::raise_nenyr_error(start_time, err))?;

        let mut crealion = Crealion::new(self.sender.clone(), parsed_ast, path.into());

        match crealion.create().await {
            Ok(None) => {}
            _ => {}
        }

        Ok(())
    }

    pub fn process_names_injection(&self, raw_content: String) -> GaladrielResult<String> {
        if self.auto_naming {
            return Ok(raw_content);
        }

        Ok(raw_content)
    }
}
