use std::path::PathBuf;

use chrono::{DateTime, Local};
use nenyr::NenyrParser;
use tokio::sync::broadcast;

use crate::{
    crealion::{Crealion, CrealionContextType},
    error::GaladrielError,
    events::GaladrielAlerts,
    utils::{
        resilient_reader::resilient_reader,
        send_palantir_error_notification::send_palantir_error_notification,
        send_palantir_notification::send_palantir_notification,
        send_palantir_success_notification::send_palantir_success_notification,
    },
    GaladrielResult,
};

/// This function handles parsing a Nenyr context file and processing it to create styles.
///
/// It performs the following steps:
/// 1. Attempts to parse the Nenyr file specified by `current_path`.
/// 2. On successful parsing, sends a success notification and returns the parsed context data.
/// 3. In case of a Nenyr-specific error, sends an error notification.
/// 4. For any other error, it notifies Palantir of the issue.
///
/// # Parameters
/// - `current_path`: The file path of the Nenyr context file to parse.
/// - `nenyr_parser`: A mutable reference to the `NenyrParser` that will be used for parsing the file.
/// - `starting_time`: The starting time of the parsing process, used for logging purposes.
/// - `palantir_sender`: The `broadcast::Sender` that will be used to send notifications.
///
/// # Returns
/// A tuple with two elements:
/// - An `Option<CrealionContextType>`, which is `Some(context_type)` if parsing is successful, or `None` if there is an error.
/// - An `Option<Vec<String>>`, which contains layout relations if parsing is successful, or `None` if there is an error.
pub async fn formera(
    current_path: PathBuf,
    nenyr_parser: &mut NenyrParser,
    starting_time: DateTime<Local>,
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
) -> (Option<CrealionContextType>, Option<Vec<String>>) {
    // Convert path to a string.
    let stringified_path = current_path.to_string_lossy().to_string();

    tracing::info!("Starting to parse Nenyr file: {:?}", stringified_path);

    // Attempt to start parsing the Nenyr file.
    let parsing_result = run_parsing(
        current_path,
        &stringified_path,
        nenyr_parser,
        palantir_sender.clone(),
    )
    .await;

    match parsing_result {
        // Notify Palantir on successful parsing.
        Ok((context_type, layout_relation)) => {
            tracing::info!("Successfully parsed Nenyr file: {:?}", stringified_path);

            send_palantir_success_notification(
                &format!("The Nenyr file located at {:?} has been successfully parsed and processed without any errors. All relevant data has been extracted and is ready for further operations.", stringified_path),
                starting_time,
                palantir_sender.clone(),
            );

            return (Some(context_type), layout_relation);
        }
        // Handle Nenyr-specific errors.
        Err(GaladrielError::NenyrError { start_time, error }) => {
            tracing::error!(
                "Nenyr error occurred while parsing file: {:?}",
                stringified_path
            );
            tracing::error!("Error details: {:?}", error);

            let mut error = error.to_owned();
            error.error_message = error.get_error_message().replace("nickname;", "");

            let notification = GaladrielAlerts::create_nenyr_error(start_time.to_owned(), error);

            send_palantir_notification(notification, palantir_sender.clone());
        }
        // Handle other errors and notify Palantir.
        Err(error) => {
            tracing::error!(
                "Unexpected error occurred while processing Nenyr file: {:?}",
                stringified_path
            );
            tracing::error!("Error details: {:?}", error);

            send_palantir_error_notification(error, Local::now(), palantir_sender.clone());
        }
    }

    (None, None)
}

/// This helper function handles the actual parsing of the Nenyr file content.
///
/// It reads the raw content from the file, parses it using the `NenyrParser`,
/// and processes it to generate the necessary styles.
///
/// # Parameters
/// - `path`: The path to the Nenyr file to be parsed.
/// - `stringified_path`: The string representation of the file path for logging purposes.
/// - `nenyr_parser`: A mutable reference to the `NenyrParser` that will perform the parsing.
/// - `palantir_sender`: The `broadcast::Sender` to send any notifications.
///
/// # Returns
/// A `GaladrielResult` containing a tuple:
/// - A `CrealionContextType` representing the parsed context.
/// - An optional vector of layout relations (if any) extracted from the parsed file.
async fn run_parsing(
    path: PathBuf,
    stringified_path: &str,
    nenyr_parser: &mut NenyrParser,
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
) -> GaladrielResult<(CrealionContextType, Option<Vec<String>>)> {
    tracing::info!("Reading raw content of Nenyr file: {:?}", stringified_path);

    // Read the raw content of the Nenyr file.
    let raw_content = resilient_reader(&path).await?;
    let start_time = Local::now();

    tracing::info!(
        "Starting to parse the content of Nenyr file: {:?}",
        stringified_path
    );

    // Attempt to parse the raw content.
    let parsed_ast = nenyr_parser
        .parse(raw_content, stringified_path.to_string())
        .map_err(|error| GaladrielError::raise_nenyr_error(start_time, error))?;

    tracing::info!("Successfully parsed Nenyr file: {:?}", stringified_path);

    // Create a new instance of Crealion for further processing.
    let mut crealion = Crealion::new(
        palantir_sender.clone(),
        parsed_ast,
        stringified_path.to_string(),
    );

    tracing::info!(
        "Creating styles from the parsed context: {:?}",
        stringified_path
    );

    // Create the required styles.
    crealion.create().await
}
