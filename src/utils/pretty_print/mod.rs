use chrono::{DateTime, Local, TimeDelta};
use crossterm::style::Stylize;
use nenyr::error::NenyrError;

use crate::{error::GaladrielError, events::GaladrielAlerts};

/// The `pretty_print` function is the main entry point for displaying different types of notifications
/// in a formatted way. It takes a `GaladrielAlerts` enum, matches it to its variant, and delegates the
/// printing to the respective function based on the type of alert.
pub fn pretty_print(notification: GaladrielAlerts) {
    // Match on the different types of alerts (errors, success, information, warnings)
    match notification {
        // Handle GaladrielError by calling the `pretty_print_galadriel_error` function
        GaladrielAlerts::GaladrielError { start_time, error } => {
            pretty_print_galadriel_error(start_time, error);
        }
        // Handle general informational alerts
        GaladrielAlerts::Information {
            start_time,
            message,
        } => {
            pretty_print_information(message, start_time);
        }
        // Handle NenyrError specifically with its own formatting
        GaladrielAlerts::NenyrError { start_time, error } => {
            pretty_print_nenyr_error(start_time, error);
        }
        // Handle success alerts, displaying additional info such as duration
        GaladrielAlerts::Success {
            start_time,
            ending_time,
            duration,
            message,
        } => {
            pretty_print_success(message, start_time, ending_time, duration);
        }
        // Handle warning alerts
        GaladrielAlerts::Warning {
            start_time,
            message,
        } => {
            pretty_print_warning(message, start_time);
        }
        _ => {}
    }
}

/// Prints a formatted success message with details like the duration and ending time.
///
/// # Arguments
/// * `message` - A string representing the success message.
/// * `start_time` - The start time of the process.
/// * `ending_time` - The end time of the process.
/// * `duration` - The time duration of the process.
fn pretty_print_success(
    message: String,
    start_time: DateTime<Local>,
    ending_time: DateTime<Local>,
    duration: TimeDelta,
) {
    // Convert the duration to milliseconds
    let duration = duration.num_milliseconds();

    // Format an additional message if the duration is greater than zero
    let additional_msg = if duration > 0 {
        format!(
            " The current process took {} ms to complete, finishing at {}",
            duration.to_string().bold(),
            date_time_formatter(&ending_time).bold()
        )
    } else {
        "".to_string()
    };

    // Build the success message in a structured format
    let formatted_message = format!(
        "\t\u{2705} {} {} {}{}",
        format!("[{}]", date_time_formatter(&start_time))
            .green()
            .bold(),
        " SUCCESS ".on_dark_green().bold().italic(),
        message,
        additional_msg
    );

    // Apply text wrapping to the formatted message
    apply_textwrap(&formatted_message, false);
}

/// Prints a formatted error message for Nenyr-related errors, including details about the context and suggestion.
///
/// # Arguments
/// * `start_time` - The time when the error occurred.
/// * `error` - The error details specific to Nenyr.
fn pretty_print_nenyr_error(start_time: DateTime<Local>, error: NenyrError) {
    // Format the basic error message with time and error type
    let formatted_message = format!(
        "\t\u{1F4A2} {} {} {}",
        format!("[{}]", date_time_formatter(&start_time))
            .magenta()
            .bold(),
        " NENYR ERROR ".on_dark_magenta().bold().italic(),
        error.get_error_message()
    );

    // Wrap the error message if necessary
    apply_textwrap(&formatted_message, true);

    // Optionally print the context name if available
    let context_name = if let Some(name) = error.get_context_name() {
        format!("\n\t\t{}: {}", "CONTEXT NAME".bold(), name)
    } else {
        "".to_string()
    };

    // Print the detailed error context, path, kind, and other info
    let formatted_message = format!(
        "\t\t{}: {:?}\n\t\t{}: {:?}{}",
        "PATH".bold(),
        error.get_context_path(),
        "KIND".bold(),
        error.get_error_kind(),
        context_name
    );

    eprintln!("{}", formatted_message);

    // Print the error's line and column details
    let formatted_message = format!(
        "\t\t{}: {}\n\t\t{}: {}\n\t\t{}: {}",
        "LINE".bold(),
        error.get_line(),
        "COLUMN".bold(),
        error.get_column(),
        "POSITION".bold(),
        error.get_position()
    );

    eprintln!("{}", formatted_message);

    // Optionally print lines before, during, and after the error if available
    if let Some(line_before) = error.get_line_before_error() {
        eprintln!("\n\t\t{}", line_before);
    }

    if let Some(line_error) = error.get_error_line() {
        if let None = error.get_line_before_error() {
            eprintln!("");
        }

        eprintln!("\t\t{}", line_error);
    }

    if let Some(line_after) = error.get_line_after_error() {
        eprintln!("\t\t{}", line_after);
    }

    // Print the suggestion if provided
    if let Some(suggestion) = error.get_suggestion() {
        let formatted_message = format!("\n\u{25C7} {}:{}", "SUGGESTION".bold(), suggestion);

        apply_textwrap(&formatted_message, true);
    } else {
        println!("");
    }
}

/// Prints a formatted Galadriel error message with specific error details.
///
/// # Arguments
/// * `start_time` - The time when the error occurred.
/// * `error` - The Galadriel error details.
fn pretty_print_galadriel_error(start_time: DateTime<Local>, error: GaladrielError) {
    // Format the error message with the appropriate color and bold/italic styling
    let formatted_message = format!(
        "\t\u{1F4A5} {} {} {} | {}: {:?} - {}: {:?}",
        format!("[{}]", date_time_formatter(&start_time))
            .red()
            .bold(),
        " GALADRIEL ERROR ".on_dark_red().bold().italic(),
        error.get_message(),
        "ERROR TYPE".bold(),
        error.get_type(),
        "ERROR KIND".bold(),
        error.get_kind(),
    );

    // Wrap the error message
    apply_textwrap(&formatted_message, true);
}

/// Prints a formatted informational message.
///
/// # Arguments
/// * `message` - The informational message to print.
/// * `start_time` - The time the information was logged.
fn pretty_print_information(message: String, start_time: DateTime<Local>) {
    // Format the informational message with the time and message
    let formatted_message = format!(
        "\t\u{1F535} {} {} {}",
        format!("[{}]", date_time_formatter(&start_time))
            .blue()
            .bold(),
        " INFORMATION ".on_dark_blue().bold().italic(),
        message
    );

    // Wrap the message if necessary
    apply_textwrap(&formatted_message, false);
}

/// Prints a formatted warning message.
///
/// # Arguments
/// * `message` - The warning message to print.
/// * `start_time` - The time the warning was logged.
fn pretty_print_warning(message: String, start_time: DateTime<Local>) {
    // Format the warning message with appropriate styling
    let formatted_message = format!(
        "\t\u{1F6A8} {} {} {}",
        format!("[{}]", date_time_formatter(&start_time))
            .yellow()
            .bold(),
        " WARNING ".on_dark_yellow().bold().italic(),
        message
    );

    // Wrap the message if necessary
    apply_textwrap(&formatted_message, false);
}

// Formats a `DateTime` to a specific time format (HH:MM:SS.mmm).
///
/// # Arguments
/// * `time` - The time to format.
fn date_time_formatter(time: &DateTime<Local>) -> String {
    time.format("%H:%M:%S.%3f").to_string()
}

/// Applies text wrapping to a message, printing it either to standard output or error.
/// It adjusts the width based on terminal size and ensures proper formatting.
///
/// # Arguments
/// * `message` - The message to print.
/// * `is_error` - A boolean flag indicating if the message is an error (true) or not (false).
fn apply_textwrap(message: &str, is_error: bool) {
    // Try to get the terminal's width and adjust the wrapping accordingly
    if let Ok((width, _)) = crossterm::terminal::size() {
        // Subtract 16 characters for margin
        let width = width.saturating_sub(16);

        // Wrap the message text to fit within the terminal width
        for (idx, msg) in textwrap::wrap(message, width as usize).iter().enumerate() {
            let tab = if idx > 0 { "\t" } else { "" };

            // Print the message either to stdout or stderr based on is_error
            if is_error {
                eprintln!("{}{}", tab, msg.to_string());
            } else {
                println!("{}{}", tab, msg.to_string());
            }
        }
    } else {
        // Print the message either to stdout or stderr based on is_error
        if is_error {
            eprintln!("{}", message);
        } else {
            println!("{}", message);
        }
    }

    println!("");
}
