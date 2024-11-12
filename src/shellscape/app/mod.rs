use std::ops::Add;

use rand::Rng;
use ratatui::widgets::ScrollbarState;
use tracing::{debug, info};

use crate::configatron::Configatron;

use super::{metadata::ShellscapeMetadata, notifications::ShellscapeNotifications};

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeApp {
    pub metadata: ShellscapeMetadata,
    pub configs: Configatron,
    pub notifications: Vec<ShellscapeNotifications>,
    pub notifications_offset: (u16, u16),
    pub notification_scroll_vertical: ScrollbarState,
}

#[allow(dead_code)]
impl ShellscapeApp {
    pub fn new(configs: Configatron, version: &str) -> Self {
        let metadata = ShellscapeMetadata::new(
            "Galadriel CSS".to_string(),
            random_subtitle_message(),
            version.to_string(),
            "Patrick Gunnar".to_string(),
            "Galadriel CSS and Nenyr License Agreement".to_string(),
            "\u{00A9} 2024 Galadriel CSS. Crafting modular, efficient, and scalable styles with precision. Built with Rust.".to_string(),
        );

        Self {
            notification_scroll_vertical: ScrollbarState::new(0),
            notifications_offset: (0, 0),
            notifications: vec![],
            configs,
            metadata,
        }
    }

    pub fn tick(&self) {
        info!("ShellscapeApp tick method called.");
    }

    pub fn add_notification(&mut self, notification: ShellscapeNotifications) {
        info!("Adding Galadriel notification in ShellscapeApp.");
        debug!("New notification: {:?}", notification);

        self.notifications.insert(0, notification);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
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

    pub fn reset_scroll_up(&mut self) {
        let (y, _) = self.notifications_offset;

        self.notifications_offset = (y.add(1), 0);
    }

    pub fn reset_scroll_down(&mut self) {
        let (y, _) = self.notifications_offset;

        self.notifications_offset = (y.saturating_sub(1), 0);
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

        let app = ShellscapeApp::new(mock_config, "1.0.0");

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
        let app = ShellscapeApp::new(mock_config, "1.0.0");

        app.tick();
    }

    #[test]
    fn test_shellscape_app_reset_configs_state() {
        let mock_config = get_configatron();
        let new_config = get_configatron();
        let mut app = ShellscapeApp::new(mock_config.clone(), "1.0.0");

        // Check initial configuration
        assert_eq!(app.configs, mock_config);

        // Reset the configuration
        app.reset_configs_state(new_config.clone());

        assert_eq!(app.configs, new_config);
    }
}
