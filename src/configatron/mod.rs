use std::{path::PathBuf, sync::Arc};

use chrono::Local;
use ignore::overrides;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::info;

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
    #[serde(rename = "autoNaming", default = "enabled_by_default")]
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
    info!("Setting default: true");

    true
}

/// Returns an empty `Vec<String>` as the default, used for the `exclude` field.
fn empty_vector_by_default() -> Vec<String> {
    info!("Setting default empty vector for exclude paths");

    vec![]
}

/// Provides "0" as the default port, allowing for a wildcard port assignment.
fn default_wildcard_port() -> String {
    info!("Setting default wildcard port to '0'");

    "0".to_string()
}

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

    info!("Normalized port from '{}' to '{}'", port, normalized_port);

    Ok(normalized_port)
}

#[derive(Clone, PartialEq, Debug)]
pub enum GaladrielConfig {
    Exclude(Vec<String>),
    AutoNaming(bool),
    ResetStyles(bool),
    MinifiedStyles(bool),
    Port(String),
}

impl GaladrielConfig {
    pub fn switch_auto_naming(&mut self) {
        if let GaladrielConfig::AutoNaming(ref mut flag) = self {
            *flag = !*flag;
        }
    }

    pub fn switch_reset_styles(&mut self) {
        if let GaladrielConfig::ResetStyles(ref mut flag) = self {
            *flag = !*flag;
        }
    }

    pub fn switch_minified_styles(&mut self) {
        if let GaladrielConfig::MinifiedStyles(ref mut flag) = self {
            *flag = !*flag;
        }
    }

    pub fn _set_exclude(&mut self, exclude: Vec<String>) {
        if let GaladrielConfig::Exclude(ref mut node) = self {
            *node = exclude;
        }
    }

    pub fn _set_port(&mut self, port: String) {
        if let GaladrielConfig::Port(ref mut node) = self {
            *node = port;
        }
    }

    pub fn get_auto_naming(&self) -> bool {
        if let GaladrielConfig::AutoNaming(ref flag) = self {
            return *flag;
        }

        false
    }

    pub fn get_reset_styles(&self) -> bool {
        if let GaladrielConfig::ResetStyles(ref flag) = self {
            return *flag;
        }

        false
    }

    pub fn get_minified_styles(&self) -> bool {
        if let GaladrielConfig::MinifiedStyles(ref flag) = self {
            return *flag;
        }

        false
    }

    pub fn get_exclude(&self) -> Vec<String> {
        if let GaladrielConfig::Exclude(ref exclude) = self {
            return exclude.to_vec();
        }

        vec![]
    }

    pub fn get_port(&self) -> String {
        if let GaladrielConfig::Port(ref port) = self {
            return port.to_owned();
        }

        "0".to_string()
    }
}

pub fn set_configatron(
    exclude: Vec<String>,
    auto_naming: bool,
    reset_styles: bool,
    minified_styles: bool,
    port: String,
) {
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
}

pub fn switch_auto_naming() {
    match CONFIGATRON.get_mut("autoNaming") {
        Some(ref mut auto_naming) => {
            auto_naming.switch_auto_naming();
        }
        None => {}
    }
}

pub fn switch_reset_styles() {
    match CONFIGATRON.get_mut("resetStyles") {
        Some(ref mut reset_styles) => {
            reset_styles.switch_reset_styles();
        }
        None => {}
    }
}

pub fn switch_minified_styles() {
    match CONFIGATRON.get_mut("minifiedStyles") {
        Some(ref mut minified_styles) => {
            minified_styles.switch_minified_styles();
        }
        None => {}
    }
}

pub fn get_auto_naming() -> bool {
    match CONFIGATRON.get("autoNaming") {
        Some(ref auto_naming) => auto_naming.get_auto_naming(),
        None => false,
    }
}

pub fn get_reset_styles() -> bool {
    match CONFIGATRON.get("resetStyles") {
        Some(ref reset_styles) => reset_styles.get_reset_styles(),
        None => false,
    }
}

pub fn get_minified_styles() -> bool {
    match CONFIGATRON.get("minifiedStyles") {
        Some(ref minified_styles) => minified_styles.get_minified_styles(),
        None => false,
    }
}

pub fn get_exclude() -> Vec<String> {
    match CONFIGATRON.get("exclude") {
        Some(ref exclude) => exclude.get_exclude(),
        None => vec![],
    }
}

pub fn get_port() -> String {
    match CONFIGATRON.get("port") {
        Some(ref port) => port.get_port(),
        None => "0".to_string(),
    }
}

pub async fn load_galadriel_configs(working_dir: &PathBuf) -> GaladrielResult<()> {
    let config_path = working_dir.join("galadriel.config.json");

    if config_path.exists() {
        match tokio::fs::read_to_string(config_path).await {
            Ok(raw_content) => {
                // Deserialize the JSON string into the ConfigurationJson struct.
                let configs_json: ConfigurationJson =
                    serde_json::from_str(&raw_content).map_err(|err| {
                        GaladrielError::raise_general_other_error(
                            ErrorKind::ConfigFileParsingError,
                            &err.to_string(),
                            ErrorAction::Notify,
                        )
                    })?;

                set_configatron(
                    configs_json.exclude,
                    configs_json.auto_naming,
                    configs_json.reset_styles,
                    configs_json.minified_styles,
                    configs_json.port,
                );
            }
            Err(err) => {
                return Err(GaladrielError::raise_general_other_error(
                    ErrorKind::ConfigFileReadError,
                    &err.to_string(),
                    ErrorAction::Notify,
                ));
            }
        }
    }

    Ok(())
}

pub fn construct_exclude_matcher(working_dir: &PathBuf) -> GaladrielResult<overrides::Override> {
    // Initialize the override builder with the working directory.
    let mut overrides = overrides::OverrideBuilder::new(working_dir);
    let exclude = get_exclude();

    // Iterate through the list of excludes from the configuration and add them to the matcher.
    for exclude in &exclude {
        overrides
            .add(&format!("!/{}", exclude.trim_start_matches("/")))
            .map_err(|err| {
                GaladrielError::raise_general_other_error(
                    ErrorKind::ExcludeMatcherCreationError,
                    &err.to_string(),
                    ErrorAction::Notify,
                )
            })?;
    }

    tracing::info!(
        "Exclude matcher constructed with {} patterns.",
        exclude.len()
    );

    // Return the built override object.
    overrides.build().map_err(|err| {
        GaladrielError::raise_general_other_error(
            ErrorKind::ExcludeMatcherBuildFailed,
            &err.to_string(),
            ErrorAction::Notify,
        )
    })
}

pub async fn reconstruct_exclude_matcher(
    working_dir: &PathBuf,
    atomically_matcher: Arc<RwLock<overrides::Override>>,
) -> GaladrielResult<GaladrielAlerts> {
    let starting_time = Local::now();
    let mut matcher = atomically_matcher.write().await;
    let new_matcher = construct_exclude_matcher(working_dir)?;

    tracing::info!("Successfully applied new exclude matcher configuration.");
    *matcher = new_matcher;

    let notification = GaladrielAlerts::create_information(
        starting_time,
        "Exclude matcher reconstructed successfully",
    );

    Ok(notification)
}

pub fn transform_configatron_to_json() -> GaladrielResult<String> {
    serde_json::to_string_pretty(
        &CONFIGATRON
            .iter()
            .map(|entry| {
                let entry_value = match entry.value() {
                    GaladrielConfig::Exclude(value) => json!(value),
                    GaladrielConfig::AutoNaming(value) => json!(value),
                    GaladrielConfig::ResetStyles(value) => json!(value),
                    GaladrielConfig::MinifiedStyles(value) => json!(value),
                    GaladrielConfig::Port(value) => json!(value),
                };

                (entry.key().to_owned(), entry_value)
            })
            .collect::<std::collections::HashMap<String, serde_json::Value>>(),
    )
    .map_err(|err| {
        GaladrielError::raise_general_other_error(
            ErrorKind::GaladrielConfigSerdeSerializationError,
            &err.to_string(),
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
        assert!(config.auto_naming);
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
