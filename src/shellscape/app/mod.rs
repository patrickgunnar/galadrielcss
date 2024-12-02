use chrono::Local;
use rand::Rng;
use ratatui::widgets::ScrollbarState;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::{SyntaxReference, SyntaxSet, SyntaxSetBuilder},
};
use tracing::{debug, info};

use crate::{
    configatron::Configatron,
    error::{ErrorAction, ErrorKind, GaladrielError},
    GaladrielResult,
};

use super::{
    alerts::{AlertTextType, ShellscapeAlerts},
    area::ShellscapeArea,
    metadata::ShellscapeMetadata,
};

// The `ShellscapeApp` struct serves as the core representation of the terminal-based application, encapsulating its configuration, UI state, alerts, and various settings that control its behavior and appearance.
// It leverages `ratatui` for rendering the UI, which enables managing interactive terminal-based UIs efficiently.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ShellscapeApp {
    pub metadata: ShellscapeMetadata,
    pub configs: Configatron,
    pub alerts: Vec<ShellscapeAlerts>,
    pub server_running_on_port: u16,

    pub table_scroll_state: ScrollbarState,
    pub dock_scroll_state: ScrollbarState,
    pub table_vertical_axis: u16,
    pub dock_vertical_axis: u16,
    pub table_scroll_len: usize,
    pub dock_scroll_len: usize,
    pub table_area: ShellscapeArea,
    pub dock_area: ShellscapeArea,

    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub syntax: SyntaxReference,
}

#[allow(dead_code)]
impl ShellscapeApp {
    /// Initializes a new instance of `ShellscapeApp` with the provided configurations and version.
    ///
    /// This method sets up the application metadata, loads the necessary syntax set for highlighting,
    /// and prepares the theme and configurations required to run the application.
    ///
    /// # Arguments
    /// * `configs` - The configuration object (`Configatron`) to initialize the application with.
    /// * `version` - A string slice representing the current version of the application.
    ///
    /// # Returns
    /// Returns a `GaladrielResult<Self>`, which contains either a successfully initialized `ShellscapeApp`
    /// or an error if something went wrong (such as missing syntax or configuration errors).
    ///
    /// # Errors
    /// This function can return errors if:
    /// * The syntax set could not be loaded (`NenyrSyntaxIntegrationFailed`).
    /// * The `Nenyr` syntax could not be found in the syntax set (`NenyrSyntaxMissing`).
    pub fn new(configs: Configatron, version: &str) -> GaladrielResult<Self> {
        // Initialize application metadata
        let metadata = ShellscapeMetadata::new(
            "Galadriel CSS".to_string(),
            random_subtitle_message(),
            version.to_string(),
            "Patrick Gunnar".to_string(),
            "Galadriel CSS and Nenyr License Agreement".to_string(),
            "\u{00A9} 2024 Galadriel CSS. Crafting modular, efficient, and scalable styles with precision. Built with Rust.".to_string(),
        );

        // Create a new syntax set builder
        let mut ssb = SyntaxSetBuilder::new();
        // Add syntax from the given folder (recursively)
        ssb.add_from_folder("src/shellscape/syntax", true)
            .map_err(|err| {
                GaladrielError::raise_general_interface_error(
                    ErrorKind::NenyrSyntaxIntegrationFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })?;

        // Build the syntax set
        let syntax_set = ssb.build();
        // Load default theme set
        let theme_set = ThemeSet::load_defaults();
        // Find the "Nenyr" syntax in the syntax set
        let syntax = syntax_set
            .find_syntax_by_name("Nenyr")
            .ok_or(GaladrielError::raise_general_interface_error(
                ErrorKind::NenyrSyntaxMissing,
                "The Nenyr syntax could not be found in the syntax set.",
                ErrorAction::Exit,
            ))?
            .to_owned();

        // Return a new ShellscapeApp instance with the configured values
        Ok(Self {
            alerts: vec![],
            table_scroll_state: ScrollbarState::new(0),
            dock_scroll_state: ScrollbarState::new(0),
            table_area: ShellscapeArea::new(0, 0, 0, 0),
            dock_area: ShellscapeArea::new(0, 0, 0, 0),
            server_running_on_port: 0,
            table_vertical_axis: 0,
            dock_vertical_axis: 0,
            table_scroll_len: 0,
            dock_scroll_len: 0,
            configs,
            metadata,
            syntax_set,
            theme_set,
            syntax,
        })
    }

    /// Highlights the provided code string using the configured syntax and theme.
    ///
    /// # Arguments
    /// * `code` - The code to be highlighted.
    ///
    /// # Returns
    /// Returns a `Vec<(syntect::highlighting::Style, String)>` representing the highlighted
    /// lines of code, where each tuple contains the styling and the highlighted text.
    ///
    /// # Errors
    /// If an error occurs during the highlighting process, an alert is added to notify the user.
    pub fn highlighter(&mut self, code: &str) -> Vec<(syntect::highlighting::Style, String)> {
        // Initialize the highlighter with the selected syntax and theme
        let mut highlighter =
            HighlightLines::new(&self.syntax, &self.theme_set.themes["Solarized (light)"]);

        let mut lines: Vec<(syntect::highlighting::Style, String)> = vec![];
        // Highlight the provided code line by line
        let highlighter_result = highlighter.highlight_line(code, &self.syntax_set);

        match highlighter_result {
            Ok(highlighter_lines) => {
                // On successful highlighting, store the result in lines
                for (style, text) in highlighter_lines {
                    lines.push((style, text.to_string()));
                }
            }
            Err(err) => {
                // On error, add an alert to notify the user
                self.add_alert(ShellscapeAlerts::create_galadriel_error(
                    Local::now(),
                    GaladrielError::raise_general_interface_error(
                        ErrorKind::NenyrSyntaxHighlightingError,
                        &err.to_string(),
                        ErrorAction::Notify,
                    ),
                ));
            }
        }

        lines
    }

    pub fn add_alerts_vec(&mut self, alerts: &mut Vec<ShellscapeAlerts>) {
        alerts.append(&mut self.alerts);

        if alerts.len() > 50 {
            self.alerts = alerts[..49].to_vec();
        } else {
            self.alerts = alerts.to_vec();
        }
    }

    /// Adds a new alert to the application.
    ///
    /// Alerts are added at the beginning of the alerts vector.
    ///
    /// # Arguments
    /// * `alert` - The alert to be added to the application.
    pub fn add_alert(&mut self, alert: ShellscapeAlerts) {
        info!("Adding Galadriel notification in ShellscapeApp.");
        debug!("New notification: {:?}", alert);

        self.alerts.insert(0, alert);

        if self.alerts.len() > 50 {
            self.alerts.pop();
        }
    }

    /// Clears all alerts from the application.
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
        self.table_scroll_state = ScrollbarState::new(0);
        self.table_vertical_axis = 0;
        self.table_scroll_len = 0;
    }

    /// Resets the port on which the server is running.
    ///
    /// # Arguments
    /// * `port` - The port number to which the server should be reset.
    pub fn reset_server_running_on_port(&mut self, port: u16) {
        self.server_running_on_port = port;
    }

    /// Resets the application configurations to the provided state.
    ///
    /// This method logs the old and new configurations for debugging purposes.
    ///
    /// # Arguments
    /// * `configs` - The new configuration object (`Configatron`) to reset the state with.
    pub fn reset_configs_state(&mut self, configs: Configatron) {
        info!("Resetting Galadriel configurations in ShellscapeApp.");
        debug!("Old configurations: {:?}", self.configs);
        debug!("New configurations: {:?}", configs);

        self.configs = configs;
    }

    /// Resets the subtitle in the metadata.
    ///
    /// # Arguments
    /// * `subtitle` - The new subtitle to set in the metadata.
    pub fn reset_subtitle(&mut self, subtitle: String) {
        self.metadata.reset_subtitle(subtitle);
    }

    /// Retrieves the current port on which the server is running.
    ///
    /// # Returns
    /// Returns the port number (`u16`) on which the server is currently running.
    pub fn get_server_running_on_port(&self) -> u16 {
        self.server_running_on_port
    }

    /// Retrieves the list of all alerts currently stored in the application.
    ///
    /// # Returns
    /// Returns a vector of `ShellscapeAlerts` containing all the notifications.
    pub fn get_alerts(&self) -> Vec<ShellscapeAlerts> {
        self.alerts.clone()
    }

    /// Retrieves the current configuration of the application.
    ///
    /// # Returns
    /// Returns the current `Configatron` instance containing the application's configurations.
    pub fn get_configs(&self) -> Configatron {
        self.configs.clone()
    }

    /// Returns the author of the application from the metadata.
    ///
    /// # Returns
    /// A `String` containing the author's name.
    pub fn get_author(&self) -> String {
        self.metadata.author.clone()
    }

    /// Returns the version of the application from the metadata.
    ///
    /// # Returns
    /// A `String` containing the version of the application.
    pub fn get_version(&self) -> String {
        self.metadata.version.clone()
    }

    /// Returns the license type of the application from the metadata.
    ///
    /// # Returns
    /// A `String` containing the license information.
    pub fn get_license(&self) -> String {
        self.metadata.license.clone()
    }

    /// Returns the subtitle of the application from the metadata.
    ///
    /// # Returns
    /// A `String` containing the subtitle.
    pub fn get_subtitle(&self) -> String {
        self.metadata.subtitle.clone()
    }

    /// Returns the title of the application from the metadata.
    ///
    /// # Returns
    /// A `String` containing the title.
    pub fn get_title(&self) -> String {
        self.metadata.title.clone()
    }

    /// Returns the footer text of the application from the metadata.
    ///
    /// # Returns
    /// A `String` containing the footer text.
    pub fn get_footer(&self) -> String {
        self.metadata.footer.clone()
    }

    /// Returns the current dock area as a `ShellscapeArea` object.
    ///
    /// # Returns
    /// A `ShellscapeArea` representing the dock area.
    pub fn get_dock_area(&self) -> ShellscapeArea {
        self.dock_area.clone()
    }

    /// Returns the current table area as a `ShellscapeArea` object.
    ///
    /// # Returns
    /// A `ShellscapeArea` representing the table area.
    pub fn get_alerts_area(&self) -> ShellscapeArea {
        self.table_area.clone()
    }

    /// Returns the vertical axis position for the table scroll.
    ///
    /// # Returns
    /// A `u16` representing the vertical axis position for the table.
    pub fn get_table_vertical_axis(&self) -> u16 {
        self.table_vertical_axis
    }

    /// Returns the vertical axis position for the dock scroll.
    ///
    /// # Returns
    /// A `u16` representing the vertical axis position for the dock.
    pub fn get_dock_vertical_axis(&self) -> u16 {
        self.dock_vertical_axis
    }

    /// Resets the table area with the provided `ShellscapeArea`.
    ///
    /// # Arguments
    /// - `area`: A `ShellscapeArea` to set as the new table area.
    pub fn reset_table_area(&mut self, area: ShellscapeArea) {
        self.table_area = area;
    }

    /// Resets the dock area with the provided `ShellscapeArea`.
    ///
    /// # Arguments
    /// - `area`: A `ShellscapeArea` to set as the new dock area.
    pub fn reset_dock_area(&mut self, area: ShellscapeArea) {
        self.dock_area = area;
    }

    /// Resets the table scroll state based on the provided length.
    ///
    /// # Arguments
    /// - `len`: The length representing the content's size for the table.
    pub fn reset_table_scroll_state(&mut self, len: usize) {
        self.table_scroll_len = len;
        self.table_scroll_state = self.table_scroll_state.content_length(len);
    }

    /// Resets the dock scroll state based on the provided length.
    ///
    /// # Arguments
    /// - `len`: The length representing the content's size for the dock.
    pub fn reset_dock_scroll_state(&mut self, len: usize) {
        self.dock_scroll_len = len;
        self.dock_scroll_state = self.dock_scroll_state.content_length(len);
    }

    pub fn tick(&self) {
        info!("ShellscapeApp tick method called.");
    }

    /// Resets the alerts scroll position upwards by one unit, within valid bounds.
    ///
    /// This method updates the vertical axis for the table, ensuring it doesn't exceed the scrollable content length.
    pub fn reset_alerts_scroll_up(&mut self) {
        let result = self.table_vertical_axis.saturating_add(2);

        if result as usize <= self.table_scroll_len {
            self.table_vertical_axis = result;
            self.table_scroll_state = self.table_scroll_state.position(result as usize);
        }
    }

    /// Resets the alerts scroll position downwards by one unit, within valid bounds.
    ///
    /// This method updates the vertical axis for the table, ensuring it doesn't go below 0.
    pub fn reset_alerts_scroll_down(&mut self) {
        let result = self.table_vertical_axis.saturating_sub(2);

        self.table_vertical_axis = result;
        self.table_scroll_state = self.table_scroll_state.position(result as usize);
    }

    /// Resets the dock scroll position upwards by one unit, within valid bounds.
    ///
    /// This method updates the vertical axis for the dock, ensuring it doesn't exceed the scrollable content length.
    pub fn reset_dock_scroll_up(&mut self) {
        let result = self.dock_vertical_axis.saturating_add(2);

        if result as usize <= self.dock_scroll_len {
            self.dock_vertical_axis = result;
            self.dock_scroll_state = self.dock_scroll_state.position(result as usize);
        }
    }

    /// Resets the dock scroll position downwards by one unit, within valid bounds.
    ///
    /// This method updates the vertical axis for the dock, ensuring it doesn't go below 0.
    pub fn reset_dock_scroll_down(&mut self) {
        let result = self.dock_vertical_axis.saturating_sub(2);

        self.dock_vertical_axis = result;
        self.dock_scroll_state = self.dock_scroll_state.position(result as usize);
    }

    // Method to add an alert for keyboard shortcuts.
    pub fn add_shortcut_alert(&mut self) {
        // A list of keyboard shortcuts and their corresponding descriptions.
        let shortcuts: Vec<(String, String)> = vec![
            (
                "'Ctrl' + 'p'".to_string(),
                "Opens the print dialog box, allowing you to configure print settings and print the current document, webpage, or image.".to_string()
            ),
            ("'Ctrl' + 'c'".to_string(), "Copy the selected text or item.".to_string()),
            ("'Ctrl' + 'v'".to_string(), "Paste the copied text or item.".to_string()),
            ("'Ctrl' + 'x'".to_string(), "Cut the selected text or item.".to_string()),
            ("'Ctrl' + 'z'".to_string(), "Undo the last action.".to_string()),
            (
                "'Ctrl' + 'p'".to_string(),
                "Opens the print dialog box, allowing you to configure print settings and print the current document, webpage, or image.".to_string()
            ),
            ("'Ctrl' + 's'".to_string(), "Save the current document or file.".to_string()),
            ("'Ctrl' + 'o'".to_string(), "Open a file or document.".to_string()),
            ("'alt' + 'tab'".to_string(), "Switch between open applications.".to_string()),
            ("'Ctrl' + 'alt' + 'del'".to_string(), "Open the security options menu.".to_string()),
            ("'Ctrl' + 'f'".to_string(), "Find a word or phrase in the document.".to_string()),
            ("'Ctrl' + 'p'".to_string(), "Print the current document or page.".to_string()),
            (
                "'Ctrl' + 'alt' + 'del'".to_string(),
                "Displays a system-level menu with options such as locking the computer, switching users, opening the Task Manager, or restarting the system.".to_string()
            ),
        ];

        self.add_alert(ShellscapeAlerts::create_shortcuts(Local::now(), shortcuts));
    }

    // Method to add a license alert, describing the terms for Galadriel CSS & Nenyr.
    pub fn add_license_alert(&mut self) {
        let title = "Galadriel CSS & Nenyr License Agreement";
        let content: Vec<String> = vec![
            String::from("This is the first paragraph of dummy content. It is filled with placeholder text to simulate a real paragraph. Let's add some emojis for fun: \u{1F600}, \u{1F603}, \u{1F604}."),
            String::from("Here is another paragraph, showcasing how \u{1F4AF} awesome this dummy text can be! Feel free to replace this text with actual content later."),
            String::from("Adding some more lines of text here just for testing purposes. Check out these emojis: \u{1F680}, \u{1F4A1}, \u{1F3C1}."),
            String::from("The final paragraph wraps it all up nicely. Don't forget to smile: \u{1F642}, \u{1F60A}, \u{1F60D}.")
        ];

        self.add_alert(ShellscapeAlerts::create_text(
            AlertTextType::License,
            Local::now(),
            title,
            content,
        ));
    }

    // Method to add a donation alert, encouraging users to support Galadriel CSS.
    pub fn add_donation_alert(&mut self) {
        let title = "Help Galadriel CSS Grow";
        let content: Vec<String> = vec![
            String::from("This is the first paragraph of dummy content. It is filled with placeholder text to simulate a real paragraph. Let's add some emojis for fun: \u{1F600}, \u{1F603}, \u{1F604}."),
            String::from("Here is another paragraph, showcasing how \u{1F4AF} awesome this dummy text can be! Feel free to replace this text with actual content later."),
            String::from("Adding some more lines of text here just for testing purposes. Check out these emojis: \u{1F680}, \u{1F4A1}, \u{1F3C1}."),
            String::from("The final paragraph wraps it all up nicely. Don't forget to smile: \u{1F642}, \u{1F60A}, \u{1F60D}.")
        ];

        self.add_alert(ShellscapeAlerts::create_text(
            AlertTextType::Donation,
            Local::now(),
            title,
            content,
        ));
    }

    // Method to add an alert for users to contribute to the project as developers.
    pub fn add_contribute_alert(&mut self) {
        let title = "Empower Galadriel CSS with Your Skills";
        let content: Vec<String> = vec![
            String::from("This is the first paragraph of dummy content. It is filled with placeholder text to simulate a real paragraph. Let's add some emojis for fun: \u{1F600}, \u{1F603}, \u{1F604}."),
            String::from("Here is another paragraph, showcasing how \u{1F4AF} awesome this dummy text can be! Feel free to replace this text with actual content later."),
            String::from("Adding some more lines of text here just for testing purposes. Check out these emojis: \u{1F680}, \u{1F4A1}, \u{1F3C1}."),
            String::from("The final paragraph wraps it all up nicely. Don't forget to smile: \u{1F642}, \u{1F60A}, \u{1F60D}.")
        ];

        self.add_alert(ShellscapeAlerts::create_text(
            AlertTextType::ContributeAsDev,
            Local::now(),
            title,
            content,
        ));
    }

    // Method to add an alert with information about the author of Galadriel CSS.
    pub fn add_about_author_alert(&mut self) {
        let title = "Crafting Galadriel CSS: The Author\'s Insight";
        let content: Vec<String> = vec![
            String::from("This is the first paragraph of dummy content. It is filled with placeholder text to simulate a real paragraph. Let's add some emojis for fun: \u{1F600}, \u{1F603}, \u{1F604}."),
            String::from("Here is another paragraph, showcasing how \u{1F4AF} awesome this dummy text can be! Feel free to replace this text with actual content later."),
            String::from("Adding some more lines of text here just for testing purposes. Check out these emojis: \u{1F680}, \u{1F4A1}, \u{1F3C1}."),
            String::from("The final paragraph wraps it all up nicely. Don't forget to smile: \u{1F642}, \u{1F60A}, \u{1F60D}.")
        ];

        self.add_alert(ShellscapeAlerts::create_text(
            AlertTextType::AboutAuthor,
            Local::now(),
            title,
            content,
        ));
    }
}

fn random_subtitle_message() -> String {
    let messages = [
        "Galadriel CSS was not designed to be merely simple; it was crafted to be a powerful, advanced and robust solution.",
        "Galadriel CSS transcends simplicity; it is a high-performance, scalable framework designed to handle the most complex styling challenges.",
        "Galadriel CSS redefines styling with precision, merging modularity and power to deliver a framework built for advanced, real-world applications.",
        "Galadriel CSS empowers developers with unparalleled control, offering a scalable, context-driven approach to CSS that adapts to any project’s needs.",
        "Galadriel CSS isn't just another tool; it's a comprehensive, modern solution designed for developers who demand efficiency, flexibility, and performance."
    ];

    let idx = rand::thread_rng().gen_range(0..messages.len());
    let selected_message = messages[idx].to_string();

    debug!("Selected random subtitle message: {}", selected_message);

    selected_message
}

#[cfg(test)]
mod tests {
    use crate::{configatron::Configatron, shellscape::app::ShellscapeApp};

    fn get_configatron() -> Configatron {
        Configatron::new(
            vec![],
            true,
            true,
            true,
            "8080".to_string(),
            "1.0.0".to_string(),
        )
    }

    #[test]
    fn test_shellscape_app_new() {
        let mock_config = get_configatron();

        let app = ShellscapeApp::new(mock_config, "1.0.0").unwrap();

        assert_eq!(app.metadata.title, "Galadriel CSS");
        assert_eq!(app.metadata.author, "Patrick Gunnar");
        assert_eq!(app.metadata.version, "1.0.0");
        assert_eq!(
            app.metadata.license,
            "Galadriel CSS and Nenyr License Agreement"
        );
        assert_eq!(app.metadata.footer, "© 2024 Galadriel CSS. Crafting modular, efficient, and scalable styles with precision. Built with Rust.");
    }

    #[test]
    fn test_shellscape_app_tick() {
        let mock_config = get_configatron();
        let app = ShellscapeApp::new(mock_config, "1.0.0").unwrap();

        app.tick();
    }

    #[test]
    fn test_shellscape_app_reset_configs_state() {
        let mock_config = get_configatron();
        let new_config = get_configatron();
        let mut app = ShellscapeApp::new(mock_config.clone(), "1.0.0").unwrap();

        // Check initial configuration
        assert_eq!(app.configs, mock_config);

        // Reset the configuration
        app.reset_configs_state(new_config.clone());

        assert_eq!(app.configs, new_config);
    }
}
