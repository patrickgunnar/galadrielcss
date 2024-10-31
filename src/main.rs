use std::{env, io};

use galadrielcss::{GaladrielRuntime, GaladrielRuntimeKind, GaladrielRuntimeResult};

/// Main asynchronous function serving as the entry point for Galadriel CSS runtime.
/// This function initializes the runtime based on command-line arguments provided by the user.
///
/// # Returns
///
/// * `GaladrielRuntimeResult<()>` - Returns a result type wrapping a unit (`()`), which will
///   indicate either success (`Ok`) or failure (`Err`) of the runtime initialization.
#[tokio::main]
async fn main() -> GaladrielRuntimeResult<()> {
    // Retrieve the command-line arguments passed to the program.
    let mut args = env::args();

    // Skip the first argument (program name) since it's not required for logic.
    args.next();

    // Match on the next argument to determine the runtime mode (`start` or `build`).
    match args.next() {
        // Check if the mode is valid (either "start" or "build").
        Some(runtime_kind) if runtime_kind == "start" || runtime_kind == "build" => {
            // Get the current working directory to use as the runtime base directory.
            let current_dir = std::env::current_dir()?;

            // Determine runtime mode based on the argument received.
            let runtime_mode = if runtime_kind == "start" {
                GaladrielRuntimeKind::Development
            } else {
                GaladrielRuntimeKind::Build
            };

            // Determine runtime mode based on the argument received.
            let mut runtime = GaladrielRuntime::new(runtime_mode, current_dir);

            // Run the runtime asynchronously and await completion.
            runtime.run().await
        }
        // Handle invalid mode or missing arguments, providing usage information.
        _ => {
            eprintln!("Usage: galadrielcss <mode>\nAvailable modes: 'start' or 'build'");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid Galadriel CSS mode!",
            ))
        }
    }
}
