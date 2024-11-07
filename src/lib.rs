use std::path::PathBuf;

use baraddur::BaraddurObserver;
use chrono::Local;
use configatron::{Configatron, ConfigurationJson};
use error::GaladrielError;
use ignore::overrides;
use kickstartor::Kickstartor;
use lothlorien::LothlorienPipeline;
use shellscape::{commands::ShellscapeCommands, Shellscape};
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
    Update,
}

pub type GaladrielCustomResult<T> = Result<T, GaladrielError>;
pub type GaladrielResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type GaladrielFuture<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, PartialEq, Debug)]
pub struct GaladrielRuntime {
    runtime_mode: GaladrielRuntimeKind,
    working_dir: PathBuf,
    configatron: Configatron,
}

impl GaladrielRuntime {
    pub fn new(runtime_mode: GaladrielRuntimeKind, working_dir: PathBuf) -> Self {
        Self {
            runtime_mode,
            working_dir,
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

    pub async fn run(&mut self) -> GaladrielResult<()> {
        match self.runtime_mode {
            GaladrielRuntimeKind::Development => self.start_development_mode().await,
            GaladrielRuntimeKind::Build => self.start_build_mode().await,
            GaladrielRuntimeKind::Update => Ok(()),
        }
    }

    async fn start_development_mode(&mut self) -> GaladrielResult<()> {
        // Creates the development logs subscriber.
        let subscriber = self.generate_log_subscriber();

        // Set logs subscriber.
        tracing::subscriber::set_global_default(subscriber).map_err(|err| {
            tracing::error!("Failed to set log subscriber: {:?}", err.to_string());

            Box::<dyn std::error::Error>::from(err.to_string())
        })?;

        tracing::info!("Log subscriber set successfully.");

        // Load the galadriel configurations.
        self.load_galadriel_config().map_err(|err| {
            tracing::error!("Failed to load Galadriel configuration: {:?}", err);

            Box::<dyn std::error::Error>::from(err)
        })?;

        tracing::info!("Galadriel CSS configuration loaded successfully.");

        let mut kickstartor = Kickstartor::new(
            self.configatron.get_exclude(),
            self.configatron.get_auto_naming(),
        );

        // TODO: Set an initial state for the UI.
        // TODO: Handle the initial parsing error.
        // Processing Nenyr files for initial setup.
        kickstartor.process_nyr_files().await.map_err(|err| {
            tracing::error!("Error processing Nenyr files: {:?}", err);

            Box::<dyn std::error::Error>::from(err.to_string())
        })?;

        tracing::info!("Nenyr files processed successfully. Initial styles AST set successfully.");

        // TODO: Pass the initial UI state for the dev runtime.
        // Transition to development runtime.
        self.development_runtime().await
    }

    async fn start_build_mode(&mut self) -> GaladrielResult<()> {
        self.load_galadriel_config()?;

        println!("Build process not implemented yet.");

        Ok(())
    }

    async fn development_runtime(&mut self) -> GaladrielResult<()> {
        // Setting Shellscape terminal interface.
        let mut shellscape = Shellscape::new();
        let mut _shellscape_events = shellscape.create_events(250);
        let mut interface = shellscape.create_interface()?;
        let mut shellscape_app = shellscape.create_app(self.configatron.clone());

        // Setting Lothlórien pipeline stream.
        let mut pipeline = LothlorienPipeline::new(self.configatron.get_port());
        let pipeline_listener = pipeline.create_listener().await?;
        let local_addr = pipeline_listener.local_addr()?;
        let running_on_port = local_addr.port();
        let _listener_handler = pipeline.create_pipeline(pipeline_listener);
        let mut _runtime_sender = pipeline.get_runtime_sender();

        // Setting Barad-dûr observer.
        let mut observer = BaraddurObserver::new();
        let exclude_matcher = self.construct_exclude_matcher()?;
        let _observer_handler = observer.start(exclude_matcher, self.working_dir.clone(), 250);

        pipeline.register_server_port_in_temp(running_on_port)?;
        interface.invoke()?;

        tracing::info!("Galadriel CSS development runtime initiated.");

        loop {
            if let Err(err) = interface.render(&mut shellscape_app) {
                println!("{:?}", err);
            }

            tokio::select! {
                // Receives events from the Lothlórien pipeline.
                pipeline_res = pipeline.next() => {
                    match pipeline_res {
                        Ok(event) => {
                            println!("{:?}", event);
                        }
                        Err(err) => {
                            println!("{:?}", err);
                        }
                    }
                }
                // Receives events from the Baraddur observer.
                baraddur_res = observer.next() => {
                    match baraddur_res {
                        Ok(event) => {
                            println!("{:?}", event);
                        }
                        Err(err) => {
                            println!("{:?}", err);
                        }
                    }
                }
                // Receives events from the shellscape/terminal interface.
                shellscape_res = shellscape.next() => {
                    match shellscape_res {
                        Ok(event) => {
                            let token = shellscape.match_shellscape_event(event);

                            if token == ShellscapeCommands::Terminate {
                                break;
                            }
                        }
                        Err(err) => {
                            println!("{:?}", err);
                        }
                    }
                }
            }
        }

        pipeline.remove_server_port_in_temp()?;
        interface.abort()?;

        Ok(())
    }

    fn construct_exclude_matcher(&self) -> GaladrielResult<overrides::Override> {
        let mut overrides = overrides::OverrideBuilder::new(self.working_dir.clone());

        for exclude in self.configatron.get_exclude().iter() {
            overrides.add(&format!("!/{}", exclude.trim_start_matches("/")))?;
        }

        Ok(overrides.build()?)
    }

    fn load_galadriel_config(&mut self) -> GaladrielResult<()> {
        let config_path = self.working_dir.join("galadriel.config.json");

        tracing::debug!("Loading Galadriel CSS configuration from {:?}", config_path);

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
                tracing::info!("Galadriel CSS configuration loaded and applied successfully.");

                Ok(())
            }
            Err(err) => {
                tracing::error!("Failed to read Galadriel CSS configuration file: {:?}", err);

                Err(Box::<dyn std::error::Error>::from(err.to_string()))
            }
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
