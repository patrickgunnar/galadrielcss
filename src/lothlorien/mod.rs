use actix_web::{rt, web, App, HttpResponse, HttpServer, Responder};
use chrono::Local;
use rand::Rng;
use std::{env, net::SocketAddr, path::PathBuf};

use tokio::{
    fs,
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::{GaladrielAlerts, GaladrielEvents},
    utils::{
        get_updated_css::get_updated_css, get_utility_class_names::get_utility_class_names,
        send_palantir_success_notification::send_palantir_success_notification,
        write_file::write_file,
    },
    GaladrielResult,
};

const GALADRIEL_TEMP_FILE_NAME: &str = "galadrielcss_lothlorien_pipeline_port.txt";

/// Represents different contexts in which styles are applied.
#[derive(Clone, PartialEq, Debug)]
pub enum ContextType {
    /// Central context, usually referring to global or main styles.
    Central,
    /// Layout context, typically related to layout-specific styles.
    Layout,
    /// Module context, generally referring to isolated or component-level styles.
    Module,
}

#[derive(Debug)]
pub struct Lothlorien {
    lothlorien_sender: mpsc::UnboundedSender<GaladrielEvents>,
    lothlorien_receiver: mpsc::UnboundedReceiver<GaladrielEvents>,

    palantir_sender: broadcast::Sender<GaladrielAlerts>,
    socket_addr: String,
    temp_folder: PathBuf,
}

#[allow(dead_code)]
impl Lothlorien {
    pub fn new(port: &str, palantir_sender: broadcast::Sender<GaladrielAlerts>) -> Self {
        let (lothlorien_sender, lothlorien_receiver) = mpsc::unbounded_channel();

        Self {
            socket_addr: format!("127.0.0.1:{}", port),
            temp_folder: env::temp_dir(),
            lothlorien_sender,
            lothlorien_receiver,
            palantir_sender,
        }
    }

    pub fn get_sender(&self) -> mpsc::UnboundedSender<GaladrielEvents> {
        self.lothlorien_sender.clone()
    }

    pub async fn next(&mut self) -> GaladrielResult<GaladrielEvents> {
        self.lothlorien_receiver.recv().await.ok_or_else(|| {
            GaladrielError::raise_general_pipeline_error(
                ErrorKind::ServerEventReceiveFailed,
                "Error while receiving response from Lothlórien server sender: No response received.",
                ErrorAction::Notify
            )
        })
    }

    pub async fn register_server_port_in_temp(&self, port: u16) -> GaladrielResult<()> {
        let temp_file = self.temp_folder.join(GALADRIEL_TEMP_FILE_NAME);

        write_file(
            self.temp_folder.clone(),
            temp_file,
            format!("{port}"),
            ErrorAction::Exit,
            ErrorKind::ServerPortRegistrationFailed,
            ErrorKind::ServerPortWriteError,
        )
        .await
    }

    pub async fn remove_server_port_in_temp(&self) -> GaladrielResult<()> {
        let temp_file = self.temp_folder.join(GALADRIEL_TEMP_FILE_NAME);

        if temp_file.exists() {
            fs::remove_file(temp_file).await.map_err(|err| {
                GaladrielError::raise_general_pipeline_error(
                    ErrorKind::ServerPortRemovalFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })?;
        }

        Ok(())
    }

    pub async fn create_socket_addr(&self) -> GaladrielResult<SocketAddr> {
        tokio::net::TcpListener::bind(self.socket_addr.clone())
            .await
            .map_err(|err| {
                GaladrielError::raise_general_pipeline_error(
                    ErrorKind::SocketAddressBindingError,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })?
            .local_addr()
            .map_err(|err| {
                GaladrielError::raise_critical_runtime_error(
                    ErrorKind::ServerLocalAddrFetchFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })
    }

    pub fn stream_sync(&self, socket_addr: SocketAddr) -> JoinHandle<()> {
        let lothlorien_sender = self.lothlorien_sender.clone();
        let palantir_sender = self.palantir_sender.clone();

        rt::spawn(async move {
            let lothlorien_sender = lothlorien_sender.clone();
            let palantir_sender = palantir_sender.clone();

            let http_server = HttpServer::new(|| {
                App::new()
                    .route("/fetch-css", web::get().to(Self::fetch_css))
                    .route(
                        "/collect-utility-class-names/{context_type}:{context_name}::{class_name}",
                        web::get().to(Self::collect_utility_names),
                    )
            });

            match http_server.bind(socket_addr) {
                Ok(server) => {
                    send_palantir_success_notification(
                        &Self::random_server_subheading_message(),
                        Local::now(),
                        palantir_sender.clone(),
                    );

                    match server.run().await {
                        Ok(()) => {}
                        Err(err) => {
                            Self::send_galadriel_error(
                                err,
                                ErrorKind::Other,
                                lothlorien_sender.clone(),
                            );
                        }
                    }
                }
                Err(err) => {
                    Self::send_galadriel_error(
                        err,
                        ErrorKind::ServerBidingError,
                        lothlorien_sender.clone(),
                    );
                }
            }
        })
    }

    async fn fetch_css() -> impl Responder {
        HttpResponse::Ok().body(get_updated_css())
    }

    async fn collect_utility_names(path: web::Path<(String, String, String)>) -> impl Responder {
        let (context_type, context_name, class_name) = path.into_inner();

        let (context_type, context_name, class_name) = match context_type.as_str() {
            "@class" => (ContextType::Central, None, context_name),
            "@layout" => (ContextType::Layout, Some(context_name), class_name),
            "@module" => (ContextType::Module, Some(context_name), class_name),
            _ => return HttpResponse::Ok().body("".to_string()),
        };

        let utility_class_names = get_utility_class_names(context_type, context_name, class_name);

        HttpResponse::Ok().body(utility_class_names)
    }

    fn send_galadriel_error(
        err: std::io::Error,
        error_kind: ErrorKind,
        lothlorien_sender: mpsc::UnboundedSender<GaladrielEvents>,
    ) {
        tracing::error!("{:?}", err);

        let error = GaladrielError::raise_critical_other_error(
            error_kind,
            &err.to_string(),
            ErrorAction::Notify,
        );

        if let Err(err) = lothlorien_sender.send(GaladrielEvents::Error(error)) {
            tracing::error!("{:?}", err);
        }
    }

    fn random_server_subheading_message() -> String {
        let messages = [
            "The light of Eärendil shines. Lothlórien is ready to begin your journey.",
            "The stars of Lothlórien guide your path. The system is fully operational.",
            "As the Mallorn trees bloom, Lothlórien is prepared for your commands.",
            "The Mirror of Galadriel is clear—development is ready to proceed.",
            "Lothlórien is fully operational and ready for development.",
        ];

        let idx = rand::thread_rng().gen_range(0..messages.len());
        let selected_message = messages[idx].to_string();

        tracing::debug!("Selected random subheading message: {}", selected_message);

        selected_message
    }
}
