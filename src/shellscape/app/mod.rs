use chrono::Local;
use rand::Rng;
use ratatui::widgets::ScrollbarState;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet, SyntaxSetBuilder},
};
use tokio::sync;
use tracing::{debug, info};

use crate::{
    asts::PALANTIR_ALERTS,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::{AlertTextType, GaladrielAlerts},
    utils::get_nenyr_syntax::get_nenyr_syntax,
    GaladrielResult,
};

use super::{area::ShellscapeArea, metadata::ShellscapeMetadata};

// The `ShellscapeApp` struct serves as the core representation of the terminal-based application, encapsulating its configuration, UI state, alerts, and various settings that control its behavior and appearance.
// It leverages `ratatui` for rendering the UI, which enables managing interactive terminal-based UIs efficiently.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ShellscapeApp {
    palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    pub metadata: ShellscapeMetadata,
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
    /// * `palantir_sender` - A channel used to send alerts to the terminal UI/User.
    ///
    /// # Returns
    /// Returns a `GaladrielResult<Self>`, which contains either a successfully initialized `ShellscapeApp`
    /// or an error if something went wrong (such as missing syntax or configuration errors).
    ///
    /// # Errors
    /// This function can return errors if:
    /// * The syntax set could not be loaded (`NenyrSyntaxIntegrationFailed`).
    /// * The `Nenyr` syntax could not be found in the syntax set (`NenyrSyntaxMissing`).
    pub fn new(
        version: &str,
        palantir_sender: sync::broadcast::Sender<GaladrielAlerts>,
    ) -> GaladrielResult<Self> {
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
        let nenyr_syntax = SyntaxDefinition::load_from_str(&get_nenyr_syntax(), true, None)
            .map_err(|err| {
                GaladrielError::raise_general_interface_error(
                    ErrorKind::NenyrSyntaxIntegrationFailed,
                    &err.to_string(),
                    ErrorAction::Exit,
                )
            })?;

        // Add syntax.
        ssb.add(nenyr_syntax);

        // Build the syntax set
        let syntax_set = ssb.build();
        // Load default theme set
        let theme_set = ThemeSet::load_defaults();
        // Find the "Nenyr" syntax in the syntax set
        let syntax = syntax_set
            .find_syntax_by_name("Nenyr")
            .ok_or_else(|| {
                GaladrielError::raise_general_interface_error(
                    ErrorKind::NenyrSyntaxMissing,
                    "The Nenyr syntax could not be found in the syntax set.",
                    ErrorAction::Exit,
                )
            })?
            .to_owned();

        // Return a new ShellscapeApp instance with the configured values
        Ok(Self {
            table_scroll_state: ScrollbarState::new(0),
            dock_scroll_state: ScrollbarState::new(0),
            table_area: ShellscapeArea::new(0, 0, 0, 0),
            dock_area: ShellscapeArea::new(0, 0, 0, 0),
            server_running_on_port: 0,
            table_vertical_axis: 0,
            dock_vertical_axis: 0,
            table_scroll_len: 0,
            dock_scroll_len: 0,
            palantir_sender,
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
                self.add_alert(GaladrielAlerts::create_galadriel_error(
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

    pub fn add_alert(&self, notification: GaladrielAlerts) {
        let palantir_sender = self.palantir_sender.clone();

        if let Err(err) = palantir_sender.send(notification) {
            tracing::error!("Failed to send alert: {:?}", err);
        }
    }

    /// Clears all alerts from the application.
    pub fn clear_alerts(&mut self) {
        match PALANTIR_ALERTS.get_mut("alerts") {
            Some(ref mut palantir) => {
                palantir.clear();
            }
            None => {}
        }

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
            ("'Esc' or 'q'".to_string(), "Terminates the application.".to_string()),
            ("'Ctrl' + 'c'".to_string(), "Terminates the application forcefully.".to_string()),
            ("'Shift' + 'w'".to_string(), "Resets all ASTs.".to_string()),
            ("'Shift' + 'r'".to_string(), "Toggles the reset styles.".to_string()),
            ("'Shift' + 'm'".to_string(), "Toggles minified styles.".to_string()),
            ("'Shift' + 'n'".to_string(), "Toggles the auto-naming feature.".to_string()),
            ("'Shift' + 'k'".to_string(), "Clears all alerts.".to_string()),
            ("'Ctrl' + 's'".to_string(), "Displays the shortcut guide.".to_string()),
            ("'Ctrl' + 'l'".to_string(), "Opens license information.".to_string()),
            ("'Ctrl' + 'd'".to_string(), "Displays the donation guide.".to_string()),
            ("'Ctrl' + 't'".to_string(), "Opens contribution information for developers.".to_string()),
            ("'Ctrl' + 'a'".to_string(), "Displays author information.".to_string()),
            ("'Ctrl' + 'Up Arrow'".to_string(), "Scrolls notifications up.".to_string()),
            ("'Ctrl' + 'Down Arrow'".to_string(), "Scrolls notifications down.".to_string()),
            ("'Shift' + 'Up Arrow'".to_string(), "Scrolls the dock up.".to_string()),
            ("'Shift' + 'Down Arrow'".to_string(), "Scrolls the dock down.".to_string()),
        ];

        self.add_alert(GaladrielAlerts::create_shortcuts(Local::now(), shortcuts));
    }

    // Method to add a license alert, describing the terms for Galadriel CSS & Nenyr.
    pub fn add_license_alert(&mut self) {
        let title = "Galadriel CSS & Nenyr License Agreement";
        let content: Vec<String> = vec![
            String::from("Galadriel CSS and Nenyr License Agreement"),
            String::from("Owner: Patrick Gunnar"),
            String::from("Products: Galadriel CSS and Nenyr"),
            String::from("Effective Date: June 28, 2024"),
            String::from("1. Ownership and Rights"),
            String::from("Patrick Gunnar, hereinafter referred to as \"Owner\", retains exclusive ownership and all intellectual property rights over the software products known as \"Galadriel CSS\" and \"Nenyr.\" This includes, but is not limited to, all source code, documentation, design, and associated materials."),
            String::from("2. Grant of License"),
            String::from("The Owner grants a perpetual, worldwide, royalty-free, non-exclusive, and non-transferable license to the end user to use Galadriel CSS and Nenyr for any purpose, including commercial and non-commercial applications, free of charge. This license does not convey ownership rights in Galadriel CSS or Nenyr to the end user."),
            String::from("3. Permitted Uses"),
            String::from("The end user is permitted to:"),
            String::from("3.1 Use Galadriel CSS and Nenyr to develop, deploy, and distribute applications for both commercial and non-commercial purposes."),
            String::from("3.2 Share applications built using Galadriel CSS and Nenyr, provided that such sharing does not misrepresent the ownership of Galadriel CSS or Nenyr."),
            String::from("3.3 Modify Galadriel CSS or Nenyr for internal use, provided such modifications are not redistributed, either as part of a product or independently, without prior written authorization from the Owner. Proper attribution to the Owner must also be maintained for any internal usage."),
            String::from("4. Restrictions"),
            String::from("4.1 The end user may not:"),
            String::from("Claim ownership of Galadriel CSS or Nenyr or their associated intellectual property."),
            String::from("Distribute, sell, lease, or sublicense Galadriel CSS or Nenyr as standalone products, defined as any distribution of the software independent of a functional application."),
            String::from("Reverse engineer, decompile, or disassemble Galadriel CSS or Nenyr."),
            String::from("Modify, adapt, or create derivative works based on Galadriel CSS or Nenyr, except for internal use as outlined in section 3.3."),
            String::from("5. Termination"),
            String::from("This license is effective until terminated. The Owner reserves the right to terminate this license if the end user breaches any term or condition of this agreement. Upon termination, the end user must cease all use of Galadriel CSS and Nenyr and destroy all copies in their possession."),
            String::from("6. No Warranty"),
            String::from("Galadriel CSS and Nenyr are provided \"as is\", without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose, and non-infringement. The Owner shall not be liable for any claim, damages, or liability, whether in contract, tort, or otherwise, arising from or in connection with Galadriel CSS or Nenyr."),
            String::from("7. Governing Law"),
            String::from("This license agreement shall be governed by and construed in accordance with the laws of Brazil, without regard to its conflict of law principles."),
            String::from("8. License Updates"),
            String::from("The Owner reserves the right to update the terms of this license for future versions of Galadriel CSS and Nenyr. However, the license granted for any version obtained by the end user will remain subject to the terms in effect at the time of acquisition."),
            String::from("9. Attribution and Derivative Works"),
            String::from("9.1 Attribution (Optional):"),
            String::from("End users are encouraged, but not required, to include attribution to the Owner when distributing applications built using Galadriel CSS or Nenyr. This could take the form of a statement such as: \"Powered by Galadriel CSS and Nenyr\" in the application's documentation or user interface."),
            String::from("9.2 Derivative Works (Authorization Required):"),
            String::from("Creating tools, frameworks, systems, or any derivative works that are based on or integrate Galadriel CSS or Nenyr requires prior written authorization from the Owner. This ensures proper acknowledgment and compliance with the terms of this agreement."),
            String::from("10. Entire Agreement"),
            String::from("This agreement constitutes the entire understanding between the parties regarding the use of Galadriel CSS and Nenyr and supersedes all prior agreements, negotiations, and discussions."),
            String::from("By using Galadriel CSS or Nenyr, the end user acknowledges that they have read, understood, and agree to be bound by the terms and conditions of this license agreement."),
            String::from("Patrick Gunnar"),
            String::from("Date: June 28, 2024")
        ];

        self.add_alert(GaladrielAlerts::create_text(
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

        self.add_alert(GaladrielAlerts::create_text(
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
            String::from("Contribute as a Developer"),
            String::from("Galadriel CSS is a powerful, innovative framework that I envisioned to bring a highly modular and efficient CSS management system to the developer community. Built using Rust for its core functionality and TypeScript for bundler integrations, Galadriel CSS aims to offer an optimized way of managing CSS in web applications."),
            String::from("The framework is designed to scale, maintain clear and efficient code, and reduce redundant styles across projects. Its flexibility and efficiency offer significant benefits to developers, allowing them to focus more on building great products without worrying about managing bloated CSS."),
            String::from("Until now, I've built and maintained the entire framework on my own, but as the project grows, additional help and expertise are needed to ensure its continued success and evolution. If you're passionate about improving CSS workflows and want to make an impact, your contributions can help Galadriel CSS reach new heights."),
            String::from("Areas to Contribute"),
            String::from("TypeScript Developers:"),
            String::from("Build integration clients (plugins) for popular bundlers such as Vite, ESBuild, Rollup, Parcel, and more. This will help expand the reach of Galadriel CSS into various development environments."),
            String::from("Help maintain and improve the documentation website, which is built with Next.js and styled using Galadriel CSS. The site uses Markdown content to render the documentation, and there are opportunities to enhance both its functionality and content."),
            String::from("Contributions could include improving the website's user interface, adding new sections, or optimizing the website for better performance and accessibility."),
            String::from("Rust Developers:"),
            String::from("Contribute by adding new features to the core of Galadriel CSS, improving its performance, and fixing bugs."),
            String::from("Assist with writing tests and resolving issues to help make the framework more robust and reliable."),
            String::from("Help expand Galadriel CSS by integrating new Rust crates such as tokio, notify, axum, and ratatui, or even implementing new ones."),
            String::from("Contribute to the development and testing of the Nenyr parser. This includes adding advanced parsing features and uncovering and fixing any bugs that might arise."),
            String::from("Both Rust and TypeScript developers have a vital role in enhancing and expanding Galadriel CSS to benefit the broader community."),
            String::from("Get In Touch"),
            String::from("If you're interested in contributing or if you have any questions, please don't hesitate to reach out! You can contact me directly at galadrielcss@gmail.com."),
            String::from("Your feedback and contributions will be highly valued, and together we can make Galadriel CSS an even more powerful tool for the development community.")
        ];

        self.add_alert(GaladrielAlerts::create_text(
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
            String::from("The Vision Behind Galadriel CSS"),
            String::from("Galadriel CSS is a transformative framework designed to revolutionize how developers approach styling in modern web applications. Born out of the desire to break free from the limitations of traditional CSS frameworks, it embodies a vision of innovation, creativity, and practicality."),
            String::from("Where other frameworks often sacrifice flexibility for familiarity, Galadriel CSS seeks to empower developers to craft distinctive, high-performance designs while maintaining clean, efficient, and scalable codebases."),
            String::from("Why Galadriel CSS Was Created"),
            String::from("The inception of Galadriel CSS stems from the challenges faced with existing frameworks—challenges like style redundancy, codebase pollution, and the difficulty of maintaining scalability in increasingly complex applications."),
            String::from("Its purpose is to resolve these issues by introducing a system built around precision, modularity, and innovation."),
            String::from("Key Pillars of the Vision:"),
            String::from("Advanced Styling Capabilities: Galadriel CSS goes beyond the basics, offering tools to create rich, visually compelling designs. It is built for developers who aspire to craft unique experiences that stand out in a crowded digital landscape."),
            String::from("Dynamic and Intelligent CSS: At the core of the framework is Nenyr /ˈnɛ.nɪ.ɑ:/ (NEH-nee-AH), a custom-designed domain-specific language (DSL). Nenyr provides an expressive and intuitive grammar for defining styles, transforming them into optimized utility classes. This intelligent system eliminates redundancy, minimizes bloat, and streamlines projects without compromising creativity."),
            String::from("Modular and Context-Aware Architecture: The framework introduces a context-based hierarchy—Central, Layout, and Module contexts—enabling precise control over styles at every level of an application. This modularity ensures that styles remain organized, cohesive, and adaptable as projects grow."),
            String::from("Guiding Principles of Galadriel CSS"),
            String::from("Clean, Organized Code: By applying utility classes dynamically in production, Galadriel CSS ensures the development phase remains uncluttered. Developers work with clear, declarative Nenyr syntax, free from the visual pollution of traditional utility-first frameworks."),
            String::from("Effortless Scalability: The context-centric design facilitates seamless growth. With a robust inheritance system, styles can be shared, extended, or overridden effortlessly, making even large-scale projects manageable and coherent."),
            String::from("Optimized Performance: The build process transforms Nenyr definitions into lean, utility-first CSS, reducing redundancy and enhancing load times. This focus on efficiency ensures that performance is never sacrificed, no matter the project's scale."),
            String::from("Empowering Developers"),
            String::from("Galadriel CSS represents a shift in how developers approach styling. It doesn't impose rigid patterns or restrict creativity. Instead, it equips developers with tools to build something genuinely unique—whether through dynamic animations, advanced variables, or modular architecture."),
            String::from("Smart Styling in Action:"),
            String::from("Build responsive designs that adapt seamlessly to project needs."),
            String::from("Keep your codebase pristine and scalable, free from the technical debt of outdated methodologies."),
            String::from("Explore limitless possibilities with a DSL designed to empower creativity."),
            String::from("Looking Ahead"),
            String::from("Galadriel CSS is more than a tool; it is a philosophy of innovation and excellence. As the creator, my journey has been about redefining the norms of web styling—challenging stagnation and pushing boundaries."),
            String::from("With Galadriel CSS, developers can embrace a future where style management is efficient, intuitive, and inspired by creativity."),
            String::from("If you're ready to break free from traditional constraints and explore new possibilities, Galadriel CSS offers the flexibility, performance, and clarity to elevate your projects to unprecedented levels."),
            String::from("Patrick Gunnar, Creator of Galadriel CSS")
        ];

        self.add_alert(GaladrielAlerts::create_text(
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
    use tokio::sync;

    use crate::shellscape::app::ShellscapeApp;

    #[test]
    fn test_shellscape_app_new() {
        let (sender, _) = sync::broadcast::channel(10);
        let app = ShellscapeApp::new("1.0.0", sender).unwrap();

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
        let (sender, _) = sync::broadcast::channel(10);
        let app = ShellscapeApp::new("1.0.0", sender).unwrap();

        app.tick();
    }
}
