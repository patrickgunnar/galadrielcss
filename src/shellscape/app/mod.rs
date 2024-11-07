use rand::Rng;
use tracing::{debug, info};

use crate::configatron::Configatron;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeApp {
    pub subtitle: String,
    pub subheading: String,
    pub current_version: String,
    pub license: String,
    pub author: String,
    pub footer: String,
    pub title: String,
    pub galadriel_configs: Configatron,
}

impl ShellscapeApp {
    pub fn new(
        galadriel_configs: Configatron,
        current_version: &str,
        license: &str,
        author: &str,
        footer: &str,
        title: &str,
    ) -> Self {
        let subtitle = random_subtitle_message();
        let subheading = random_subheading_message();

        info!(
            "Initializing ShellscapeApp with title: {}, version: {}",
            title, current_version
        );
        debug!("ShellscapeApp initial subtitle: {}", subtitle);
        debug!("ShellscapeApp initial subheading: {}", subheading);

        Self {
            current_version: current_version.into(),
            license: license.into(),
            author: author.into(),
            footer: footer.into(),
            title: title.into(),
            galadriel_configs,
            subheading,
            subtitle,
        }
    }

    pub fn tick(&self) {
        info!("ShellscapeApp tick method called.");
    }

    pub fn reset_galadriel_configs_state(&mut self, configs: Configatron) {
        info!("Resetting Galadriel configurations in ShellscapeApp.");
        debug!("Old configurations: {:?}", self.galadriel_configs);
        debug!("New configurations: {:?}", configs);

        self.galadriel_configs = configs;
    }

    pub fn change_title(&mut self, title: String) {
        self.title = title;
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

fn random_subheading_message() -> String {
    let messages = [
        "The light of Eärendil shines. Lothlórien is ready to begin your journey.",
        "The stars of Lothlórien guide your path. The system is fully operational.",
        "As the Mallorn trees bloom, Lothlórien is prepared for your commands.",
        "The Mirror of Galadriel is clear—development is ready to proceed.",
        "Lothlórien is fully operational and ready for development.",
    ];

    let idx = rand::thread_rng().gen_range(0..messages.len());
    let selected_message = messages[idx].to_string();

    debug!("Selected random subheading message: {}", selected_message);

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

        let app = ShellscapeApp::new(
            mock_config,
            "1.0.0",
            "MIT",
            "Author Name",
            "Footer Info",
            "App Title",
        );

        assert_eq!(app.current_version, "1.0.0");
        assert_eq!(app.license, "MIT");
        assert_eq!(app.author, "Author Name");
        assert_eq!(app.footer, "Footer Info");
        assert_eq!(app.title, "App Title");
    }

    #[test]
    fn test_shellscape_app_tick() {
        let mock_config = get_configatron();
        let app = ShellscapeApp::new(
            mock_config,
            "1.0.0",
            "MIT",
            "Author Name",
            "Footer Info",
            "App Title",
        );

        app.tick();
    }

    #[test]
    fn test_shellscape_app_reset_galadriel_configs_state() {
        let mock_config = get_configatron();
        let new_config = get_configatron();
        let mut app = ShellscapeApp::new(
            mock_config.clone(),
            "1.0.0",
            "MIT",
            "Author Name",
            "Footer Info",
            "App Title",
        );

        // Check initial configuration
        assert_eq!(app.galadriel_configs, mock_config);

        // Reset the configuration
        app.reset_galadriel_configs_state(new_config.clone());

        assert_eq!(app.galadriel_configs, new_config);
    }

    // Test for the `change_title` method
    #[test]
    fn test_shellscape_app_change_title() {
        let mock_config = get_configatron();
        let mut app = ShellscapeApp::new(
            mock_config,
            "1.0.0",
            "MIT",
            "Author Name",
            "Footer Info",
            "App Title",
        );

        app.change_title("New Title".to_string());
        assert_eq!(app.title, "New Title");
    }
}
