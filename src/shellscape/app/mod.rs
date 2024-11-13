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
    area::ShellscapeArea, metadata::ShellscapeMetadata, notifications::ShellscapeNotifications,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ShellscapeApp {
    pub metadata: ShellscapeMetadata,
    pub configs: Configatron,
    pub notifications: Vec<ShellscapeNotifications>,
    pub notifications_scroll_offset: (u16, u16),
    pub notifications_area: ShellscapeArea,
    pub notification_vertical_scroll_state: ScrollbarState,
    pub notifications_scroll_len: usize,
    pub dock_scroll_offset: (u16, u16),
    pub dock_area: ShellscapeArea,
    pub dock_vertical_scroll_state: ScrollbarState,
    pub dock_scroll_len: usize,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub syntax: SyntaxReference,
}

#[allow(dead_code)]
impl ShellscapeApp {
    pub fn new(configs: Configatron, version: &str) -> GaladrielResult<Self> {
        let metadata = ShellscapeMetadata::new(
            "Galadriel CSS".to_string(),
            random_subtitle_message(),
            version.to_string(),
            "Patrick Gunnar".to_string(),
            "Galadriel CSS and Nenyr License Agreement".to_string(),
            "\u{00A9} 2024 Galadriel CSS. Crafting modular, efficient, and scalable styles with precision. Built with Rust.".to_string(),
        );

        let mut ssb = SyntaxSetBuilder::new();
        ssb.add_from_folder("src/shellscape/syntax", true)
            .map_err(|err| {
                GaladrielError::raise_general_interface_error(
                    ErrorKind::NenyrSyntaxIntegrationFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })?;

        let syntax_set = ssb.build();
        let theme_set = ThemeSet::load_defaults();
        let syntax = syntax_set
            .find_syntax_by_name("Nenyr")
            .ok_or(GaladrielError::raise_general_interface_error(
                ErrorKind::NenyrSyntaxMissing,
                "The Nenyr syntax could not be found in the syntax set.",
                ErrorAction::Exit,
            ))?
            .to_owned();

        Ok(Self {
            notification_vertical_scroll_state: ScrollbarState::new(0),
            notifications_scroll_offset: (0, 0),
            notifications_area: ShellscapeArea::new(0, 0, 0, 0),
            notifications_scroll_len: 0,
            dock_vertical_scroll_state: ScrollbarState::new(0),
            dock_scroll_offset: (0, 0),
            dock_area: ShellscapeArea::new(0, 0, 0, 0),
            dock_scroll_len: 0,
            notifications: vec![],
            configs,
            metadata,
            syntax_set,
            theme_set,
            syntax,
        })
    }

    pub fn highlighter(&mut self, code: &str) -> Vec<(syntect::highlighting::Style, String)> {
        let mut highlighter =
            HighlightLines::new(&self.syntax, &self.theme_set.themes["Solarized (light)"]);

        let mut lines: Vec<(syntect::highlighting::Style, String)> = vec![];
        let highlighter_result = highlighter.highlight_line(code, &self.syntax_set);

        match highlighter_result {
            Ok(highlighter_lines) => {
                for (style, text) in highlighter_lines {
                    lines.push((style, text.to_string()));
                }
            }
            Err(err) => {
                self.add_notification(ShellscapeNotifications::create_galadriel_error(
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

    pub fn tick(&self) {
        info!("ShellscapeApp tick method called.");
    }

    pub fn get_author(&self) -> String {
        self.metadata.author.clone()
    }

    pub fn get_version(&self) -> String {
        self.metadata.version.clone()
    }

    pub fn get_license(&self) -> String {
        self.metadata.license.clone()
    }

    pub fn get_subtitle(&self) -> String {
        self.metadata.subtitle.clone()
    }

    pub fn get_title(&self) -> String {
        self.metadata.title.clone()
    }

    pub fn get_footer(&self) -> String {
        self.metadata.footer.clone()
    }

    pub fn get_notifications(&self) -> Vec<ShellscapeNotifications> {
        self.notifications.clone()
    }

    pub fn get_notifications_offset(&self) -> (u16, u16) {
        self.notifications_scroll_offset.clone()
    }

    pub fn get_dock_offset(&self) -> (u16, u16) {
        self.dock_scroll_offset.clone()
    }

    pub fn get_dock_area(&self) -> ShellscapeArea {
        self.dock_area.clone()
    }

    pub fn get_notifications_area(&self) -> ShellscapeArea {
        self.notifications_area.clone()
    }

    pub fn add_notification(&mut self, notification: ShellscapeNotifications) {
        info!("Adding Galadriel notification in ShellscapeApp.");
        debug!("New notification: {:?}", notification);

        self.notifications.insert(0, notification);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    pub fn reset_notifications_scroll_state(&mut self, len: usize) {
        self.notifications_scroll_len = len;
        self.notification_vertical_scroll_state =
            self.notification_vertical_scroll_state.content_length(len);
    }

    pub fn reset_dock_scroll_state(&mut self, len: usize) {
        self.dock_scroll_len = len;
        self.dock_vertical_scroll_state = self.dock_vertical_scroll_state.content_length(len);
    }

    pub fn reset_notifications_area(&mut self, area: ShellscapeArea) {
        self.notifications_area = area;
    }

    pub fn reset_dock_area(&mut self, area: ShellscapeArea) {
        self.dock_area = area;
    }

    pub fn reset_configs_state(&mut self, configs: Configatron) {
        info!("Resetting Galadriel configurations in ShellscapeApp.");
        debug!("Old configurations: {:?}", self.configs);
        debug!("New configurations: {:?}", configs);

        self.configs = configs;
    }

    pub fn reset_subtitle(&mut self, subtitle: String) {
        self.metadata.reset_subtitle(subtitle);
    }

    pub fn reset_notifications_scroll_up(&mut self) {
        let (y, _) = self.notifications_scroll_offset;
        let result = y.saturating_add(1);

        if result as usize <= self.notifications_scroll_len {
            self.notifications_scroll_offset = (result, 0);
            self.notification_vertical_scroll_state = self
                .notification_vertical_scroll_state
                .position(result as usize);
        }
    }

    pub fn reset_notifications_scroll_down(&mut self) {
        let (y, _) = self.notifications_scroll_offset;
        let result = y.saturating_sub(1);

        self.notifications_scroll_offset = (result, 0);
        self.notification_vertical_scroll_state = self
            .notification_vertical_scroll_state
            .position(result as usize);
    }

    pub fn reset_dock_scroll_up(&mut self) {
        let (y, _) = self.dock_scroll_offset;
        let result = y.saturating_add(1);

        if result as usize <= self.dock_scroll_len {
            self.dock_scroll_offset = (result, 0);
            self.dock_vertical_scroll_state =
                self.dock_vertical_scroll_state.position(result as usize);
        }
    }

    pub fn reset_dock_scroll_down(&mut self) {
        let (y, _) = self.dock_scroll_offset;
        let result = y.saturating_sub(1);

        self.dock_scroll_offset = (result, 0);
        self.dock_vertical_scroll_state = self.dock_vertical_scroll_state.position(result as usize);
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
