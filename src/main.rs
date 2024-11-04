use std::env;

use galadrielcss::{GaladrielResult, GaladrielRuntime, GaladrielRuntimeKind};

fn get_usage_message() -> String {
    "Usage:\n    galadrielcss <mode>\n\nAvailable modes:\n    'start'   - Launches the development server\n    'build'   - Compiles the project for production\n    'update'  - Updates Galadriel CSS to the latest version".to_string()
}

/// Main asynchronous function serving as the entry point for Galadriel CSS runtime.
/// This function initializes the runtime based on command-line arguments provided by the user.
///
/// # Returns
///
/// * `GaladrielRuntimeResult<()>` - Returns a result type wrapping a unit (`()`), which will
///   indicate either success (`Ok`) or failure (`Err`) of the runtime initialization.
#[tokio::main]
async fn main() -> GaladrielResult<()> {
    // Retrieve the command-line arguments passed to the program.
    let mut args = env::args();

    // Skip the first argument (program name) since it's not required for logic.
    args.next();

    // Match on the next argument to determine the runtime mode (`start` or `build`).
    match args.next() {
        // Check if the mode is valid (either "start" or "build").
        Some(runtime_kind)
            if runtime_kind == "start" || runtime_kind == "build" || runtime_kind == "update" =>
        {
            // Get the current working directory to use as the runtime base directory.
            let current_dir = std::env::current_dir()?;

            // Determine runtime mode based on the argument received.
            let runtime_mode = if runtime_kind == "start" {
                GaladrielRuntimeKind::Development
            } else if runtime_kind == "build" {
                GaladrielRuntimeKind::Build
            } else {
                GaladrielRuntimeKind::Update
            };

            // Determine runtime mode based on the argument received.
            let mut runtime = GaladrielRuntime::new(runtime_mode, current_dir);

            // Run the runtime asynchronously and await completion.
            runtime.run().await
        }
        // Handle invalid mode or missing arguments, providing usage information.
        Some(input) => {
            eprintln!("Error: Invalid mode `{}`", input);
            eprintln!();
            eprintln!("{}", get_usage_message());
            eprintln!();
            eprintln!("Please enter a valid mode and try again.");

            Err(Box::<dyn std::error::Error>::from(format!(
                "`{}` is not a valid Galadriel CSS mode.",
                input
            )))
        }
        None => {
            eprintln!("Error: No mode specified.");
            eprintln!();
            eprintln!("{}", get_usage_message());
            eprintln!();
            eprintln!("Please specify one of the valid modes and try again.");

            Err(Box::<dyn std::error::Error>::from(
                "No mode specified. Please provide a valid mode ('start', 'build', or 'update').",
            ))
        }
    }
}
