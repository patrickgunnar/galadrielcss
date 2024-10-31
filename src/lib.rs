use std::{io, path::PathBuf};

use chrono::Local;
use configatron::{Configatron, ConfigurationJson};
use error::GaladrielError;
use kickstartor::Kickstartor;
use tracing::{level_filters::LevelFilter, Level};
use tracing_appender::{non_blocking::NonBlocking, rolling};
use tracing_subscriber::{
    fmt::format::{DefaultFields, Format},
    FmtSubscriber,
};

mod asts;
mod baraddur;
mod configatron;
mod error;
mod kickstartor;
mod lothlorien;
mod shellscape;

#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielRuntimeKind {
    Development,
    Build,
}

pub type GaladrielRuntimeResult<T> = io::Result<T>;
pub type GaladrielResult<T> = Result<T, GaladrielError>;

#[derive(Clone, PartialEq, Debug)]
pub struct GaladrielRuntime {
    runtime_mode: GaladrielRuntimeKind,
    current_dir: PathBuf,
    configatron: Configatron,
}

impl GaladrielRuntime {
    pub fn new(runtime_mode: GaladrielRuntimeKind, current_dir: PathBuf) -> Self {
        Self {
            runtime_mode,
            current_dir,
            configatron: Configatron::new(
                vec![],
                true,
                true,
                true,
                "0".to_string(),
                "0.0.0".to_string(),
            ),
        }
    }

    pub async fn run(&mut self) -> GaladrielRuntimeResult<()> {
        match self.runtime_mode {
            GaladrielRuntimeKind::Development => self.start_development_mode().await,
            GaladrielRuntimeKind::Build => self.start_build_mode().await,
        }
    }

    async fn start_development_mode(&mut self) -> GaladrielRuntimeResult<()> {
        // Creates the development logs subscriber.
        let subscriber = self.generate_log_subscriber();

        match tracing::subscriber::set_global_default(subscriber) {
            Ok(_) => {}
            Err(_) => {}
        }

        // Load the galadriel configurations.
        self.load_galadriel_config()?;

        let mut kickstartor = Kickstartor::new(
            self.configatron.get_exclude(),
            self.configatron.get_auto_naming(),
        );

        // TODO: Set an initial state for the UI.

        match kickstartor.process_nyr_files().await {
            Ok(_) => {}
            Err(_) => {}
        }

        // TODO: Pass the initial UI state for the dev runtime.
        self.development_runtime().await
    }

    async fn start_build_mode(&mut self) -> GaladrielRuntimeResult<()> {
        self.load_galadriel_config()?;

        println!("Build process not implemented yet.");

        Ok(())
    }

    async fn development_runtime(&mut self) -> GaladrielRuntimeResult<()> {
        Ok(())
    }

    fn load_galadriel_config(&mut self) -> GaladrielRuntimeResult<()> {
        let config_path = self.current_dir.join("galadriel.config.json");

        match std::fs::read_to_string(config_path) {
            Ok(raw_config) => {
                let config_json: ConfigurationJson = serde_json::from_str(&raw_config)?;
                let configatron = Configatron::new(
                    config_json.exclude,
                    config_json.auto_naming,
                    config_json.reset_styles,
                    config_json.minified_styles,
                    config_json.port,
                    config_json.version,
                );

                self.configatron = configatron;

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn generate_log_filename(&self) -> String {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

        format!("galadrielcss_log_{}.log", timestamp)
    }

    fn generate_log_subscriber(
        &self,
    ) -> FmtSubscriber<DefaultFields, Format, LevelFilter, NonBlocking> {
        let file_name = self.generate_log_filename();
        let file_appender = rolling::never("logs", file_name);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .with_writer(non_blocking)
            .finish()
    }
}
