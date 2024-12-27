use std::{path::PathBuf, sync::Arc};

use chrono::Local;
use ignore::overrides;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use tokio::sync::RwLock;

use crate::{
    asts::CONFIGATRON,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
    GaladrielResult,
};

/// Represents configuration settings for the application, deserialized from a JSON file.
///
/// Fields are deserialized using `serde`, with custom default functions specified for each.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConfigurationJson {
    /// List of paths or identifiers to exclude from the process.
    /// Defaults to an empty vector if not provided.
    #[serde(default = "empty_vector_by_default")]
    pub exclude: Vec<String>,

    /// Boolean flag indicating whether names should be injected during the process.
    /// Renamed in JSON as `autoNaming` and defaults to `true`.
    #[serde(rename = "autoNaming", default = "disenabled_by_default")]
    pub auto_naming: bool,

    /// Boolean flag specifying whether styles should be reset.
    /// Renamed in JSON as `resetStyles` and defaults to `true`.
    #[serde(rename = "resetStyles", default = "enabled_by_default")]
    pub reset_styles: bool,

    /// Boolean flag indicating if the styles should be minified.
    /// Renamed in JSON as `minifiedStyles` and defaults to `true`.
    #[serde(rename = "minifiedStyles", default = "enabled_by_default")]
    pub minified_styles: bool,

    /// Port setting for the application, allowing a wildcard ("0") as default.
    /// If provided, the value is normalized by `normalize_wildcard_port`.
    #[serde(
        default = "default_wildcard_port",
        deserialize_with = "normalize_wildcard_port"
    )]
    pub port: String,
}

/// Returns `true` as the default value, used for fields requiring an enabled default state.
fn enabled_by_default() -> bool {
    tracing::info!("Setting default: true");

    true
}

/// Returns `false` as the default value, used for fields requiring a disenabled default state.
fn disenabled_by_default() -> bool {
    tracing::info!("Setting default: false");

    false
}

/// Returns an empty `Vec<String>` as the default, used for the `exclude` field.
fn empty_vector_by_default() -> Vec<String> {
    tracing::info!("Setting default empty vector for exclude paths");

    vec![]
}

/// Provides "0" as the default port, allowing for a wildcard port assignment.
fn default_wildcard_port() -> String {
    tracing::info!("Setting default wildcard port to '0'");

    "0".to_string()
}

/// Normalize the received port transforming the "*" into "0" - "0" means to the system look for any available port.
fn normalize_wildcard_port<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let port = String::deserialize(deserializer)?;
    let normalized_port = if port == "*" {
        "0".to_string()
    } else {
        port.clone()
    };

    tracing::info!("Normalized port from '{}' to '{}'", port, normalized_port);

    Ok(normalized_port)
}

#[derive(Clone, PartialEq, Debug)]
/// Represents the configuration options for Galadriel CSS.
/// Each variant corresponds to a specific configuration setting.
pub enum GaladrielConfig {
    /// A list of file or directory paths to exclude from processing.
    Exclude(Vec<String>),
    /// Determines whether auto-naming is enabled.
    AutoNaming(bool),
    /// Indicates whether to reset styles to a default state.
    ResetStyles(bool),
    /// Specifies whether styles should be minified.
    MinifiedStyles(bool),
    /// The port to be used by the system.
    Port(String),
}

impl GaladrielConfig {
    /// Toggles the state of the `AutoNaming` configuration.
    pub fn switch_auto_naming(&mut self) {
        if let GaladrielConfig::AutoNaming(ref mut flag) = self {
            *flag = !*flag;
        }
    }

    /// Toggles the state of the `ResetStyles` configuration.
    pub fn switch_reset_styles(&mut self) {
        if let GaladrielConfig::ResetStyles(ref mut flag) = self {
            *flag = !*flag;
        }
    }

    /// Toggles the state of the `MinifiedStyles` configuration.
    pub fn switch_minified_styles(&mut self) {
        if let GaladrielConfig::MinifiedStyles(ref mut flag) = self {
            *flag = !*flag;
        }
    }

    /// Updates the list of paths to exclude in the `Exclude` configuration.
    pub fn _set_exclude(&mut self, exclude: Vec<String>) {
        if let GaladrielConfig::Exclude(ref mut node) = self {
            *node = exclude;
        }
    }

    /// Updates the port in the `Port` configuration.
    pub fn _set_port(&mut self, port: String) {
        if let GaladrielConfig::Port(ref mut node) = self {
            *node = port;
        }
    }

    /// Retrieves the current state of the `AutoNaming` configuration.
    pub fn get_auto_naming(&self) -> bool {
        if let GaladrielConfig::AutoNaming(ref flag) = self {
            return *flag;
        }

        false
    }

    /// Retrieves the current state of the `ResetStyles` configuration.
    pub fn get_reset_styles(&self) -> bool {
        if let GaladrielConfig::ResetStyles(ref flag) = self {
            return *flag;
        }

        true
    }

    /// Retrieves the current state of the `MinifiedStyles` configuration.
    pub fn get_minified_styles(&self) -> bool {
        if let GaladrielConfig::MinifiedStyles(ref flag) = self {
            return *flag;
        }

        true
    }

    /// Retrieves the current list of excluded paths from the `Exclude` configuration.
    pub fn get_exclude(&self) -> Vec<String> {
        if let GaladrielConfig::Exclude(ref exclude) = self {
            return exclude.to_vec();
        }

        vec![]
    }

    /// Retrieves the current port from the `Port` configuration.
    pub fn get_port(&self) -> String {
        if let GaladrielConfig::Port(ref port) = self {
            return port.to_owned();
        }

        "0".to_string()
    }
}

/// Updates the global configuration map `CONFIGATRON` with new configuration values.
///
/// # Parameters
/// - `exclude`: Paths to exclude from processing.
/// - `auto_naming`: Whether to enable auto-naming.
/// - `reset_styles`: Whether to reset styles to defaults.
/// - `minified_styles`: Whether styles should be minified.
/// - `port`: The port to use for the system.
pub fn set_configatron(
    exclude: Vec<String>,
    auto_naming: bool,
    reset_styles: bool,
    minified_styles: bool,
    port: String,
) {
    tracing::trace!(
        "Entering set_configatron with parameters: exclude={:?}, auto_naming={}, reset_styles={}, minified_styles={}, port={}",
        exclude, auto_naming, reset_styles, minified_styles, port
    );

    CONFIGATRON.insert("exclude".to_string(), GaladrielConfig::Exclude(exclude));
    CONFIGATRON.insert(
        "autoNaming".to_string(),
        GaladrielConfig::AutoNaming(auto_naming),
    );
    CONFIGATRON.insert(
        "resetStyles".to_string(),
        GaladrielConfig::ResetStyles(reset_styles),
    );
    CONFIGATRON.insert(
        "minifiedStyles".to_string(),
        GaladrielConfig::MinifiedStyles(minified_styles),
    );
    CONFIGATRON.insert("port".to_string(), GaladrielConfig::Port(port));

    tracing::info!("Updated CONFIGATRON with new configuration values.");
    tracing::debug!("Current CONFIGATRON state: {:?}", *CONFIGATRON);
}

/// Toggles the state of the `AutoNaming` configuration in `CONFIGATRON`.
pub fn switch_auto_naming() {
    match CONFIGATRON.get_mut("autoNaming") {
        Some(ref mut auto_naming) => {
            auto_naming.switch_auto_naming();
            tracing::info!("Toggled 'autoNaming' configuration.");
        }
        None => {}
    }
}

/// Toggles the state of the `ResetStyles` configuration in `CONFIGATRON`.
pub fn switch_reset_styles() {
    match CONFIGATRON.get_mut("resetStyles") {
        Some(ref mut reset_styles) => {
            reset_styles.switch_reset_styles();
            tracing::info!("Toggled 'resetStyles' configuration.");
        }
        None => {}
    }
}

/// Toggles the state of the `MinifiedStyles` configuration in `CONFIGATRON`.
pub fn switch_minified_styles() {
    match CONFIGATRON.get_mut("minifiedStyles") {
        Some(ref mut minified_styles) => {
            minified_styles.switch_minified_styles();
            tracing::info!("Toggled 'minifiedStyles' configuration.");
        }
        None => {}
    }
}

/// Retrieves the current state of the `AutoNaming` configuration.
/// Returns `true` if enabled, or `false` if not found or disabled.
/// Defaults to `true`.
pub fn get_auto_naming() -> bool {
    match CONFIGATRON.get("autoNaming") {
        Some(ref auto_naming) => auto_naming.get_auto_naming(),
        None => false,
    }
}

/// Retrieves the current state of the `ResetStyles` configuration.
/// Returns `true` if enabled, or `false` if not found or disabled.
/// Defaults to `true`.
pub fn get_reset_styles() -> bool {
    match CONFIGATRON.get("resetStyles") {
        Some(ref reset_styles) => reset_styles.get_reset_styles(),
        None => true,
    }
}

/// Retrieves the current state of the `MinifiedStyles` configuration.
/// Returns `true` if enabled, or `false` if not found or disabled.
/// Defaults to `true`.
pub fn get_minified_styles() -> bool {
    match CONFIGATRON.get("minifiedStyles") {
        Some(ref minified_styles) => minified_styles.get_minified_styles(),
        None => true,
    }
}

/// Retrieves the list of excluded paths from the `Exclude` configuration.
/// Returns a vector of excluded paths or an empty vector if not found.
pub fn get_exclude() -> Vec<String> {
    match CONFIGATRON.get("exclude") {
        Some(ref exclude) => exclude.get_exclude(),
        None => vec![],
    }
}

/// Retrieves the port value from the `Port` configuration.
/// Returns the port as a string, or "0" if not found.
pub fn get_port() -> String {
    match CONFIGATRON.get("port") {
        Some(ref port) => port.get_port(),
        None => "0".to_string(),
    }
}

/// Loads Galadriel configurations from the specified `galadriel.config.json` file.
///
/// # Parameters
/// - `working_dir`: A reference to the working directory path where the configuration file is located.
///
/// # Returns
/// - `Ok(())`: If the configuration file is successfully loaded and applied.
/// - `Err(GaladrielError)`: If an error occurs during file reading or parsing.
pub async fn load_galadriel_configs(working_dir: &PathBuf) -> GaladrielResult<()> {
    // Construct the full path to the configuration file.
    let config_path = working_dir.join("galadriel.config.json");

    tracing::info!(
        "Attempting to load Galadriel configuration file from {:?}",
        config_path
    );

    // Check if the configuration file exists.
    if config_path.exists() {
        match tokio::fs::read_to_string(config_path).await {
            // If the file is successfully read, deserialize its content.
            Ok(raw_content) => {
                if raw_content.is_empty() {
                    return Ok(());
                }

                tracing::info!("Configuration file read successfully. Deserializing content.");

                // Deserialize the JSON string into the ConfigurationJson struct.
                let configs_json: ConfigurationJson =
                    serde_json::from_str(&raw_content).map_err(|err| {
                        tracing::error!("Error parsing configuration file: {}", err);

                        // Raise an error if deserialization fails.
                        GaladrielError::raise_general_other_error(
                            ErrorKind::ConfigFileParsingError,
                            &format!("Something went wrong while parsing the `galadriel.config.json` file. Err: {}", err.to_string()),
                            ErrorAction::Notify,
                        )
                    })?;

                tracing::info!("Configuration successfully parsed. Applying settings.");

                // Apply the deserialized configurations to CONFIGATRON.
                set_configatron(
                    configs_json.exclude,
                    configs_json.auto_naming,
                    configs_json.reset_styles,
                    configs_json.minified_styles,
                    configs_json.port,
                );

                tracing::info!("Configuration settings successfully applied.");
            }
            // Raise an error if the file cannot be read.
            Err(err) => {
                tracing::error!("Error reading configuration file: {}", err);

                return Err(GaladrielError::raise_general_other_error(
                    ErrorKind::ConfigFileReadError,
                    &format!("Something went wrong while reading the `galadriel.config.json` file. Err: {}", err.to_string()),
                    ErrorAction::Notify,
                ));
            }
        }
    }

    Ok(())
}

/// Constructs an exclude matcher based on the configuration's exclude patterns.
///
/// # Parameters
/// - `working_dir`: A reference to the working directory path.
///
/// # Returns
/// - `GaladrielResult<overrides::Override>`: The built exclude matcher or an error if construction fails.
pub fn construct_exclude_matcher(working_dir: &PathBuf) -> GaladrielResult<overrides::Override> {
    tracing::info!("Constructing exclude matcher using patterns from configuration.");

    // Initialize the override builder with the working directory.
    let mut overrides = overrides::OverrideBuilder::new(working_dir);
    let exclude = get_exclude();

    // Iterate through the list of excludes from the configuration and add them to the matcher.
    for exclude in &exclude {
        tracing::info!("Adding exclude pattern: {}", exclude);

        // Add each pattern, ensuring proper format.
        overrides
            .add(&format!("!/{}", exclude.trim_start_matches("/")))
            .map_err(|err| {
                tracing::error!("Error adding exclude pattern: {}", err);

                // Handle errors that occur while adding patterns.
                GaladrielError::raise_general_other_error(
                    ErrorKind::ExcludeMatcherCreationError,
                    &format!(
                        "Something went wrong while constructing the exclude matcher. Err: {}",
                        err.to_string()
                    ),
                    ErrorAction::Notify,
                )
            })?;
    }

    tracing::info!(
        "Exclude matcher constructed with {} patterns.",
        exclude.len()
    );

    // Build the override object and return it, handling any errors that occur during the build process.
    overrides.build().map_err(|err| {
        tracing::error!("Error building exclude matcher: {}", err);

        GaladrielError::raise_general_other_error(
            ErrorKind::ExcludeMatcherBuildFailed,
            &format!("Something went wrong while reconstructing the exclude matcher with new values. Err: {}", err.to_string()),
            ErrorAction::Notify,
        )
    })
}

/// Reconstructs the exclude matcher by replacing the existing matcher.
///
/// # Parameters
/// - `working_dir`: A reference to the working directory path.
/// - `atomically_matcher`: A thread-safe reference to the current exclude matcher.
///
/// # Returns
/// - `GaladrielResult<GaladrielAlerts>`: An alert indicating the operation's success or an error if it fails.
pub async fn reconstruct_exclude_matcher(
    working_dir: &PathBuf,
    atomically_matcher: Arc<RwLock<overrides::Override>>,
) -> GaladrielResult<GaladrielAlerts> {
    let starting_time = Local::now(); // Record the start time for the operation.
    let mut matcher = atomically_matcher.write().await; // Acquire a write lock to modify the current matcher.
    let new_matcher = construct_exclude_matcher(working_dir)?; // Construct a new exclude matcher based on the updated configuration.

    // Replace the current matcher with the new one.
    *matcher = new_matcher;

    tracing::info!("Successfully applied new exclude matcher configuration.");

    // Create an informational alert indicating successful reconstruction.
    let notification = GaladrielAlerts::create_information(
        starting_time,
        "Exclude matcher reconstructed successfully",
    );

    Ok(notification)
}

/// Converts the current `CONFIGATRON` configuration into a pretty-printed JSON string.
///
/// # Returns
/// - `GaladrielResult<String>`: The JSON representation of the configuration, or an error if serialization fails.
pub fn transform_configatron_to_json() -> GaladrielResult<String> {
    tracing::info!("Converting current configuration (CONFIGATRON) to pretty-printed JSON.");

    // Serialize the CONFIGATRON key-value pairs into a JSON object.
    serde_json::to_string_pretty(
        &CONFIGATRON
            .iter()
            .map(|entry| {
                // Match each configuration entry type and serialize its value to JSON.
                let entry_value = match entry.value() {
                    GaladrielConfig::Exclude(value) => json!(value),
                    GaladrielConfig::AutoNaming(value) => json!(value),
                    GaladrielConfig::ResetStyles(value) => json!(value),
                    GaladrielConfig::MinifiedStyles(value) => json!(value),
                    GaladrielConfig::Port(value) => json!(value),
                };

                // Return the key-value pair for the serialized configuration entry.
                (entry.key().to_owned(), entry_value)
            })
            .collect::<std::collections::HashMap<String, serde_json::Value>>(),
    )
    .map_err(|err| {
        tracing::error!("Error serializing configuration to JSON: {}", err);

        // Handle errors that occur during JSON serialization.
        GaladrielError::raise_general_other_error(
            ErrorKind::GaladrielConfigSerdeSerializationError,
            &format!("Something went wrong while transforming the current Galadriel CSS configurations into JSON. Err: {}", err.to_string()),
            ErrorAction::Notify,
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::configatron::ConfigurationJson;

    #[test]
    fn test_default_configuration() {
        let json_data = "{}"; // Empty JSON object
        let config: ConfigurationJson = serde_json::from_str(json_data).unwrap();

        // Check default values
        assert_eq!(config.exclude, Vec::<String>::new());
        assert!(!config.auto_naming);
        assert!(config.reset_styles);
        assert!(config.minified_styles);
        assert_eq!(config.port, "0");
    }

    #[test]
    fn test_custom_configuration() {
        let json_data = r#"
        {
            "exclude": ["path/to/exclude"],
            "autoNaming": false,
            "resetStyles": true,
            "minifiedStyles": false,
            "port": "*",
            "version": "1.0.0"
        }"#;

        let config: ConfigurationJson = serde_json::from_str(json_data).unwrap();

        // Check custom values
        assert_eq!(config.exclude, vec!["path/to/exclude"]);
        assert!(!config.auto_naming);
        assert!(config.reset_styles);
        assert!(!config.minified_styles);
        assert_eq!(config.port, "0"); // normalize_wildcard_port should convert "*" to "0"
    }

    #[test]
    fn test_configatron_initialization() {
        let config = ConfigurationJson {
            exclude: vec!["path1".to_string(), "path2".to_string()],
            auto_naming: true,
            reset_styles: false,
            minified_styles: true,
            port: "8080".to_string(),
        };

        // Verify initialization
        assert_eq!(config.exclude, vec!["path1", "path2"]);
        assert!(config.auto_naming);
        assert!(!config.reset_styles);
        assert!(config.minified_styles);
        assert_eq!(config.port, "8080");
    }
}
