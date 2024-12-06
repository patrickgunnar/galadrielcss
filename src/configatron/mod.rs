use serde::{Deserialize, Deserializer, Serialize};
use tracing::info;

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

    /// Version of the Galadriel CSS to be used on build process, initialized to "*" (latest) if unspecified.
    #[serde(default = "initial_version")]
    pub version: String,
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

/// Initializes the version to "*" (latest) if not provided, typically used as an initial version indicator.
fn initial_version() -> String {
    info!("Setting default version to '*' - latest");

    "*".to_string()
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

/// Represents configuration settings for an application or process, with key parameters
/// for controlling behavior such as exclusions, style preferences, and connection settings.
#[derive(Clone, PartialEq, Debug)]
pub struct Configatron {
    /// List of paths or identifiers to exclude from the configuration's scope.
    exclude: Vec<String>,
    /// Flag indicating if names should be injected during configuration processing.
    auto_naming: bool,
    /// Flag specifying if styles should be reset during configuration.
    reset_styles: bool,
    /// Flag indicating if the styles should be minified.
    minified_styles: bool,
    /// Port for network connections, represented as a string to allow flexibility.
    port: String,
    /// Version of Galadriel CSS to be used on build process, generally in "X.Y.Z" format.
    version: String,
}

impl Configatron {
    /// Constructs a new `Configatron` instance with specified configuration parameters.
    ///
    /// # Parameters
    ///
    /// * `exclude` - A vector of paths or identifiers to exclude.
    /// * `auto_naming` - Determines if names should be injected.
    /// * `reset_styles` - Specifies whether styles should be reset.
    /// * `minified_styles` - Indicates if styles should be minified.
    /// * `port` - Network port as a string.
    /// * `version` - Galadriel CSS version to be used on build process.
    ///
    /// # Returns
    ///
    /// * `Self` - A new `Configatron` instance.
    pub fn new(
        exclude: Vec<String>,
        auto_naming: bool,
        reset_styles: bool,
        minified_styles: bool,
        port: String,
        version: String,
    ) -> Self {
        info!(
            "Initializing Galadriel CSS configurations with exclude: {:?}, auto_naming: {}, reset_styles: {}, \
            minified_styles: {}, port: {}, version: {}",
            exclude, auto_naming, reset_styles, minified_styles, port, version
        );

        Self {
            exclude,
            auto_naming,
            reset_styles,
            minified_styles,
            port,
            version,
        }
    }

    /// Retrieves the vector of excluded paths or identifiers.
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A clone of the `exclude` vector.
    pub fn get_exclude(&self) -> Vec<String> {
        // info!("Fetching exclude paths: {:?}", self.exclude);

        self.exclude.clone()
    }

    /// Checks if names should be injected during processing.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if names are to be saved, otherwise `false`.
    pub fn get_auto_naming(&self) -> bool {
        // info!("Fetching auto naming status: {}", self.auto_naming);

        self.auto_naming
    }

    /// Checks if styles should be reset during processing.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if styles are to be reset, otherwise `false`.
    pub fn get_reset_styles(&self) -> bool {
        // info!("Fetching reset styles status: {}", self.reset_styles);

        self.reset_styles
    }

    /// Checks if styles should be minified.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if styles are to be minified, otherwise `false`.
    pub fn get_minified_styles(&self) -> bool {
        // info!("Fetching minified styles status: {}", self.minified_styles);

        self.minified_styles
    }

    /// Retrieves the network port configuration.
    ///
    /// # Returns
    ///
    /// * `String` - A clone of the `port` value.
    pub fn get_port(&self) -> String {
        // info!("Fetching port: {}", self.port);

        self.port.clone()
    }

    /// Retrieves the version of Galadriel CSS to be used in build mode.
    ///
    /// # Returns
    ///
    /// * `String` - A clone of the `version` value.
    pub fn get_version(&self) -> String {
        // info!("Fetching version: {}", self.version);

        self.version.clone()
    }

    pub fn toggle_reset_styles(&mut self) {
        self.reset_styles = !self.reset_styles;
    }

    pub fn toggle_minified_styles(&mut self) {
        self.minified_styles = !self.minified_styles;
    }

    pub fn toggle_auto_naming(&mut self) {
        self.auto_naming = !self.auto_naming;
    }

    /*pub fn reset_version(&mut self, version: String) {
        self.version = version;
    }*/

    pub fn generate_configs_json(&self) -> ConfigurationJson {
        ConfigurationJson {
            exclude: self.exclude.clone(),
            auto_naming: self.auto_naming,
            reset_styles: self.reset_styles,
            minified_styles: self.minified_styles,
            port: self.port.clone(),
            version: self.version.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configatron::{Configatron, ConfigurationJson};

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
        assert_eq!(config.version, "*");
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
        assert_eq!(config.version, "1.0.0");
    }

    #[test]
    fn test_configatron_initialization() {
        let config_json = ConfigurationJson {
            exclude: vec!["path1".to_string(), "path2".to_string()],
            auto_naming: true,
            reset_styles: false,
            minified_styles: true,
            port: "8080".to_string(),
            version: "2.0.0".to_string(),
        };

        let config = Configatron::new(
            config_json.exclude,
            config_json.auto_naming,
            config_json.reset_styles,
            config_json.minified_styles,
            config_json.port,
            config_json.version,
        );

        // Verify initialization
        assert_eq!(config.get_exclude(), vec!["path1", "path2"]);
        assert!(config.get_auto_naming());
        assert!(!config.get_reset_styles());
        assert!(config.get_minified_styles());
        assert_eq!(config.get_port(), "8080");
        assert_eq!(config.get_version(), "2.0.0");
    }
}
