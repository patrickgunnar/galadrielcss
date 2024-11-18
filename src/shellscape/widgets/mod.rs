use chrono::{DateTime, Local, TimeDelta};
use nenyr::error::NenyrError;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};

use textwrap::Options;

use crate::{configatron::Configatron, error::GaladrielError};

use super::{
    alerts::{AlertTextType, ShellscapeAlerts},
    app::ShellscapeApp,
    area::ShellscapeArea,
};

/// `ShellscapeWidgets` is a structure that holds color configurations and other settings
/// to style various UI elements in a terminal-based application using `ratatui`.
/// The widget handles the overall appearance and visual styling of different components
/// within the user interface, such as the primary, secondary, and other specialized colors
/// for foreground elements, backgrounds, and highlights.
#[derive(Debug)]
pub struct ShellscapeWidgets {
    primary_color: Color,
    secondary_color: Color,
    foreground_color: Color,
    light_cream_color: Color,
    deep_teal_color: Color,
    off_white_color: Color,
    dark_mustard_color: Color,
}

impl ShellscapeWidgets {
    /// Creates a new instance of `ShellscapeWidgets` with predefined colors.
    ///
    /// # Returns
    /// A new `ShellscapeWidgets` instance with a consistent color scheme.
    pub fn new() -> Self {
        Self {
            primary_color: Color::Rgb(50, 70, 60),
            secondary_color: Color::Rgb(0, 35, 35),
            foreground_color: Color::Rgb(5, 10, 10),
            light_cream_color: Color::Rgb(240, 240, 240),
            deep_teal_color: Color::Rgb(0, 105, 105),
            off_white_color: Color::Rgb(245, 245, 245),
            dark_mustard_color: Color::Rgb(128, 85, 0),
        }
    }

    /// Paints the entire UI on the terminal frame.
    ///
    /// # Arguments
    /// * `frame` - The terminal frame to render widgets onto.
    /// * `app` - A reference to the application state, containing data for UI components.
    pub fn paint(&self, frame: &mut Frame, app: &mut ShellscapeApp) {
        // Define the layout of the UI in vertical segments: header, table, and footer.
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Header occupies 20% of the screen.
                Constraint::Percentage(72), // Main content (table) takes up 72%.
                Constraint::Percentage(8),  // Footer occupies the remaining 8%.
            ])
            .split(frame.area());

        // Render the header widget.
        let header_width = layout[0].width;
        let header = self.create_header(header_width, app);
        frame.render_widget(header, layout[0]);

        // Render the main table content.
        self.create_table(layout[1], frame, app);

        // Render the footer widget.
        let footer = self.create_footer(app);
        frame.render_widget(footer, layout[2]);
    }

    /// Creates metadata for the header, such as author, license, and version information.
    ///
    /// # Arguments
    /// * `app` - The application state containing metadata values.
    ///
    /// # Returns
    /// A vector of styled spans representing metadata.
    fn create_metadata(&self, app: &mut ShellscapeApp) -> Vec<Span> {
        vec![
            Span::styled("Author: ", Style::default().fg(self.light_cream_color)),
            Span::styled(
                app.get_author(),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.light_cream_color),
            ),
            Span::styled(" \u{25E6} ", Style::default().fg(self.light_cream_color)),
            Span::styled("License: ", Style::default().fg(self.light_cream_color)),
            Span::styled(
                app.get_license(),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.light_cream_color),
            ),
            Span::styled(" \u{25E6} ", Style::default().fg(self.light_cream_color)),
            Span::styled("Version: ", Style::default().fg(self.light_cream_color)),
            Span::styled(
                app.get_version(),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.light_cream_color),
            ),
        ]
    }

    /// Creates the subtitle lines for the header, wrapping text to fit the specified width.
    ///
    /// # Arguments
    /// * `width` - The maximum width of the subtitle text.
    /// * `app` - The application state containing the subtitle text.
    ///
    /// # Returns
    /// A vector of styled lines representing the wrapped subtitle.
    fn create_subtitle(&self, width: usize, app: &mut ShellscapeApp) -> Vec<Line> {
        let subtitle = app.get_subtitle();
        let subtitle_lines = textwrap::wrap(&subtitle, Options::new(width));

        subtitle_lines
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(self.light_cream_color),
                ))
            })
            .collect()
    }

    /// Creates the header widget, including title, metadata, and subtitle.
    ///
    /// # Arguments
    /// * `header_width` - The width of the header area.
    /// * `app` - The application state containing title and metadata.
    ///
    /// # Returns
    /// A `Paragraph` widget representing the header.
    fn create_header(&self, header_width: u16, app: &mut ShellscapeApp) -> Paragraph {
        let title = format!(" {} ", app.get_title().to_uppercase());
        let mut elements = vec![];

        // Add metadata (author, license, version) to the header.
        let author = self.create_metadata(app);
        elements.push(Line::from(author));

        // Add a blank line for spacing.
        let blank_line = Line::from(Span::raw(""));
        elements.push(blank_line);

        // Add the subtitle lines.
        let width = header_width.saturating_div(10).saturating_mul(6) as usize;
        let mut subtitle = self.create_subtitle(width, app);
        elements.append(&mut subtitle);

        // Create and style the header paragraph.
        Paragraph::new(elements)
            .alignment(Alignment::Center)
            .bg(self.primary_color)
            .block(
                Block::default()
                    .padding(Padding::top(1))
                    .title(Span::styled(
                        title,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(self.light_cream_color),
                    ))
                    .borders(Borders::ALL)
                    .fg(self.secondary_color)
                    .title_alignment(Alignment::Center),
            )
    }

    /// Formats a label for configuration items in the settings dock.
    ///
    /// # Arguments
    /// * `icon` - Icon representing the configuration item.
    /// * `title` - The name of the configuration setting.
    /// * `value` - The value of the configuration setting.
    /// * `dock_width` - The width available for rendering the label.
    ///
    /// # Returns
    /// A vector of styled lines representing the formatted label.
    fn format_config_label(
        &self,
        icon: String,
        title: String,
        value: String,
        dock_width: u16,
    ) -> Vec<Line> {
        let length = title.len().saturating_add(value.len()).saturating_add(11);
        let repeat = dock_width.saturating_sub(length as u16) as usize;

        vec![Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default()),
            Span::styled(
                format!(" {} ", title),
                Style::default().bold().fg(self.off_white_color),
            ),
            Span::styled(" ".repeat(repeat), Style::default()),
            Span::styled(
                format!(" {}", value),
                Style::default().fg(Color::White).italic(),
            ),
            Span::styled(" ", Style::default()),
        ])]
    }

    /// Formats a vector of strings (`value`) into multiple lines, applying a specific styling format.
    /// - Each line starts with an emoji and a stylized title.
    /// - Wraps long strings to fit within the `dock_width` with additional formatting.
    ///
    /// # Parameters
    /// - `title`: The title displayed before the list.
    /// - `value`: A vector of strings to format.
    /// - `dock_width`: The width of the dock, used to calculate line lengths.
    ///
    /// # Returns
    /// - A `Vec<Line>` representing the formatted output.
    fn format_exclude_vec(&self, title: String, value: Vec<String>, dock_width: u16) -> Vec<Line> {
        let mut lines = vec![];

        // Add the title line with a styled label and emoji.
        let title = Line::from(vec![
            Span::raw(" \u{1F7E5}  "),
            Span::styled(
                format!("{}:", title,),
                Style::default().bold().fg(self.off_white_color),
            ),
        ]);

        lines.push(title);

        // Process each value in the `value` vector.
        value.iter().for_each(|v| {
            // Calculate the width for text wrapping, subtracting extra spacing.
            let textwrap_width = dock_width.saturating_sub(15) as usize;
            let parts = textwrap::wrap(v, Options::new(textwrap_width));
            let parts_len = parts.len().saturating_sub(1);

            // Iterate through each wrapped part and format lines accordingly.
            parts.iter().enumerate().for_each(|(i, p)| {
                let part = p.to_string();
                let part_len = part.len().saturating_add(16);
                let repeat = dock_width.saturating_sub(part_len as u16) as usize;

                // Determine prefix and suffix for the line based on position.
                let open_str = if i == 0 { "   \u{1F538} \"" } else { "      " };
                let closing_str = if i == parts_len { "\"" } else { "" };

                // Add the formatted line to the output vector.
                lines.push(Line::from(vec![
                    Span::raw(open_str),
                    Span::styled(
                        format!("{}{}", part, closing_str),
                        Style::default().italic().fg(Color::White),
                    ),
                    Span::raw(" ".repeat(repeat)),
                ]));
            });
        });

        lines
    }

    /// Creates a styled title for a block, centered within the available dock width.
    ///
    /// # Parameters
    /// - `title`: The text to be displayed as the block title.
    /// - `dock_width`: The width of the dock for centering the title.
    ///
    /// # Returns
    /// - A `Line` containing the styled block title.
    fn format_block_title(&self, title: String, dock_width: u16) -> Line {
        let block_title_len = title.len().saturating_add(4) as u16;
        let label_width = dock_width.saturating_sub(block_title_len).saturating_div(2) as usize;
        let spaces = " ".repeat(label_width);

        Line::from(Span::styled(
            format!("{}{}{}", spaces, title, spaces),
            Style::default()
                .bg(self.dark_mustard_color)
                .fg(self.light_cream_color)
                .bold(),
        ))
        .alignment(Alignment::Center)
    }

    /// Formats a label and value pair into a single stylized line.
    ///
    /// # Parameters
    /// - `label`: The label to be displayed.
    /// - `value`: The value corresponding to the label.
    /// - `dock_width`: The total width of the dock for alignment calculations.
    ///
    /// # Returns
    /// - A `Line` containing the formatted label and value.
    fn format_dock_option(&self, label: String, value: String, dock_width: u16) -> Line {
        let label_len = label.len();
        let value_len = value.len();
        let option_len = label_len.saturating_add(value_len).saturating_add(7);
        let option_line_len = dock_width.saturating_sub(option_len as u16);
        let spaces = " ".repeat(option_line_len as usize);

        Line::from(vec![
            Span::styled(format!(" \u{27A9} {}", label), Style::default()),
            Span::raw(spaces),
            Span::styled(
                format!("{} ", value),
                Style::default().bold().fg(self.off_white_color),
            ),
        ])
    }

    /// Generates a detailed viewer for configuration settings, formatted as multiple lines.
    ///
    /// # Parameters
    /// - `dock_width`: The width of the dock for alignment and wrapping.
    /// - `configs`: A `Configatron` object containing configuration details.
    /// - `port`: The port number to display.
    ///
    /// # Returns
    /// - A `Vec<Line>` containing the configuration viewer output.
    fn create_configs_viewer(&self, dock_width: u16, configs: Configatron, port: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Add the configuration title.
        let configs_title = self.format_block_title(" Configuration".to_string(), dock_width);
        lines.push(configs_title);
        lines.push(Line::from(Span::raw("")));

        // Format and append each configuration item.
        let mut reset_styles = self.format_config_label(
            "\u{1F7E6}".to_string(),
            "Reset Styles".to_string(),
            format!("{}", configs.get_reset_styles()),
            dock_width,
        );

        lines.append(&mut reset_styles);

        let mut minified_styles = self.format_config_label(
            "\u{1F7EA}".to_string(),
            "Minified Styles".to_string(),
            format!("{}", configs.get_minified_styles()),
            dock_width,
        );

        lines.append(&mut minified_styles);

        let mut auto_naming = self.format_config_label(
            "\u{1F7EB}".to_string(),
            "Auto Naming".to_string(),
            format!("{}", configs.get_auto_naming()),
            dock_width,
        );

        lines.append(&mut auto_naming);

        let mut port_element = self.format_config_label(
            "\u{2B1B}".to_string(),
            "Port".to_string(),
            format!("{}", port),
            dock_width,
        );

        lines.append(&mut port_element);

        let mut version = self.format_config_label(
            "\u{1F7E7}".to_string(),
            "Version".to_string(),
            format!("{}", configs.get_version()),
            dock_width,
        );

        lines.append(&mut version);

        let mut exclude =
            self.format_exclude_vec("Exclude".to_string(), configs.get_exclude(), dock_width);

        lines.append(&mut exclude);

        lines
    }

    /// Creates a list of lines for toggle adjustment options displayed in the dock.
    /// Each toggle has a label and a corresponding shortcut key.
    ///
    /// # Parameters:
    /// - `dock_width`: The width of the dock in terminal cells (u16).
    ///
    /// # Returns:
    /// - `Vec<Line>`: A list of lines representing the toggle adjustment options.
    fn create_adjustment_options(&self, dock_width: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Format the title of the section, centered in the dock.
        let configs_title = self.format_block_title(" Toggle Settings".to_string(), dock_width);

        // Create individual toggle options with labels and key bindings.
        let toggle_reset = self.format_dock_option(
            "Reset Styles".to_string(),
            "'Shift' + 'R'".to_string(),
            dock_width,
        );

        let toggle_minified = self.format_dock_option(
            "Minified Styles".to_string(),
            "'Shift' + 'M'".to_string(),
            dock_width,
        );

        let toggle_naming = self.format_dock_option(
            "Auto Naming".to_string(),
            "'Shift' + 'N'".to_string(),
            dock_width,
        );

        // Add the formatted title and options to the list of lines.
        lines.push(configs_title);
        lines.push(Line::from(Span::raw("")));
        lines.push(toggle_reset);
        lines.push(toggle_minified);
        lines.push(toggle_naming);

        lines
    }

    /// Creates a list of lines for extra options displayed in the dock.
    ///
    /// # Parameters:
    /// - `dock_width`: The width of the dock in terminal cells (u16).
    ///
    /// # Returns:
    /// - `Vec<Line>`: A list of lines representing extra options and their shortcuts.
    fn create_extra_options(&self, dock_width: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Format the title of the section.
        let configs_title = self.format_block_title(" Options".to_string(), dock_width);

        // Create individual options with their respective shortcut keys.
        let clear_alerts = self.format_dock_option(
            "Clear All Alerts".to_string(),
            "'Shift' + 'K'".to_string(),
            dock_width,
        );

        let shortcuts = self.format_dock_option(
            "View Shortcuts".to_string(),
            "'Ctrl' + 'S'".to_string(),
            dock_width,
        );

        let license = self.format_dock_option(
            "View License".to_string(),
            "'Ctrl' + 'L'".to_string(),
            dock_width,
        );

        let about_author = self.format_dock_option(
            "About the Author".to_string(),
            "'Ctrl' + 'A'".to_string(),
            dock_width,
        );

        let donation = self.format_dock_option(
            "Make a Donation".to_string(),
            "'Ctrl' + 'D'".to_string(),
            dock_width,
        );

        let contribute = self.format_dock_option(
            "Contribute as Dev".to_string(),
            "'Ctrl' + 'T'".to_string(),
            dock_width,
        );

        // Add the title and options to the list of lines.
        lines.push(configs_title);
        lines.push(Line::from(Span::raw("")));
        lines.push(clear_alerts);
        lines.push(shortcuts);
        lines.push(license);
        lines.push(about_author);
        lines.push(donation);
        lines.push(contribute);

        lines
    }

    /// Creates a visual separator for dividing sections within the dock.
    ///
    /// # Parameters:
    /// - `dock_width`: The width of the dock in terminal cells (u16).
    ///
    /// # Returns:
    /// - `Vec<Line>`: A list of lines representing the separator.
    fn create_separator(&self, dock_width: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Add an empty line for spacing before the separator.
        lines.push(Line::from(Span::raw("")));

        // Create the separator line using repeated Unicode characters.
        lines.push(Line::from(Span::raw(
            "\u{25E6}".repeat(dock_width.saturating_sub(3) as usize),
        )));

        // Add another empty line for spacing after the separator.
        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Formats the content of the dock, including configuration settings, adjustment toggles,
    /// and extra options.
    ///
    /// # Parameters:
    /// - `dock_width`: The width of the dock in terminal cells (u16).
    /// - `configs`: Configuration settings provided by the `Configatron` object.
    /// - `port`: The port number where the application is running (u16).
    ///
    /// # Returns:
    /// - `Vec<Line>`: A list of lines representing all dock content.
    fn format_dock_settings(&self, dock_width: u16, configs: Configatron, port: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Append configuration viewer lines.
        let mut configs_viewer = self.create_configs_viewer(dock_width, configs, port);
        lines.append(&mut configs_viewer);

        // Append separator lines between sections.
        let mut configs_separator = self.create_separator(dock_width);
        lines.append(&mut configs_separator);

        // Append adjustment option lines.
        let mut adjustment_options = self.create_adjustment_options(dock_width);
        lines.append(&mut adjustment_options);

        let mut adjustment_separator = self.create_separator(dock_width);
        lines.append(&mut adjustment_separator);

        // Append extra options.
        let mut extra_options = self.create_extra_options(dock_width);
        lines.append(&mut extra_options);

        // Add a trailing empty line for spacing.
        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Creates the dock UI component as a `Paragraph` with formatted content.
    ///
    /// # Parameters:
    /// - `dock_width`: The width of the dock in terminal cells (u16).
    /// - `app`: The application state, containing configurations and runtime data.
    ///
    /// # Returns:
    /// - `(Paragraph, usize)`: The dock UI element and the total number of lines.
    fn create_dock(&self, dock_width: u16, app: &mut ShellscapeApp) -> (Paragraph, usize) {
        // Format the dock's content into lines.
        let lines: Vec<Line> = self.format_dock_settings(
            dock_width,
            app.get_configs(),
            app.get_server_running_on_port(),
        );

        // Calculate the total number of lines for scrolling purposes.
        let lines_len = lines.len();

        // Create a Paragraph widget for the dock with background color and padding.
        let element = Paragraph::new(lines)
            .bg(self.secondary_color)
            .scroll((app.get_dock_vertical_axis(), 0))
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 1, 1))
                    .fg(self.off_white_color),
            );

        (element, lines_len)
    }

    /// Formats the title line for an alert message.
    ///
    /// # Arguments
    /// - `time`: A reference to the `DateTime` object representing the current time.
    /// - `icon`: A string representing an icon to prepend to the title.
    /// - `time_fg`: The foreground color for the time display.
    /// - `title`: The title text of the alert.
    /// - `title_bg`: The background color for the title text.
    ///
    /// # Returns
    /// A `Line` object containing the formatted title.
    fn format_alert_title(
        &self,
        time: &DateTime<Local>,
        icon: String,
        time_fg: Color,
        title: String,
        title_bg: Color,
    ) -> Line {
        // Formats the time as a styled string.
        let time = self.date_time_formatter(time);

        // Creates a sequence of styled spans to build the title.
        let spans: Vec<Span> = vec![
            Span::raw(icon),
            Span::styled(
                format!(" {}", time),
                Style::default().add_modifier(Modifier::BOLD).fg(time_fg),
            ),
            Span::styled(" \u{25E6} ", Style::default()),
            Span::styled(
                title,
                Style::default()
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                    .bg(title_bg),
            ),
        ];

        Line::from(spans)
    }

    /// Formats the main message of an alert, wrapping and styling its content.
    ///
    /// # Arguments
    /// - `message`: The alert message to be displayed.
    /// - `textwrap_width`: The maximum width of the text for wrapping.
    ///
    /// # Returns
    /// A vector of `Line` objects representing the formatted message.
    fn format_alert_message(&self, message: String, textwrap_width: usize) -> Vec<Line> {
        // Prefix the message with a decorative icon.
        let message = format!("\u{25C7} {}", message);
        // Wrap the message text to fit the specified width.
        let message_lines = textwrap::wrap(&message, textwrap_width);

        // Process each wrapped line to apply styling.
        message_lines
            .iter()
            .map(|line| {
                let mut spans: Vec<Span> = line
                    .to_string()
                    .split("**")
                    .enumerate()
                    .map(|(idx, part)| {
                        if idx % 2 == 1 {
                            Span::styled(
                                part.to_string(),
                                Style::default().add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::raw(part.to_string())
                        }
                    })
                    .collect();

                spans.insert(0, Span::raw("    ".to_string()));
                Line::from(spans)
            })
            .collect()
    }

    /// Formats a labeled section of an alert with an icon and accompanying text.
    ///
    /// # Arguments
    /// - `icon`: A string representing an icon for the label.
    /// - `label`: The bold label text.
    /// - `message`: The associated message text for the label.
    ///
    /// # Returns
    /// A single `Line` object representing the formatted label.
    fn format_alert_label(&self, icon: String, label: String, message: String) -> Line {
        Line::from(vec![
            Span::raw(format!("        {} ", icon)),
            Span::styled(
                label,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.deep_teal_color),
            ),
            Span::raw(" "),
            Span::styled(message, Style::default().fg(self.light_cream_color)),
        ])
    }

    /// Formats a single line of an alert message, applying syntax highlighting.
    ///
    /// # Arguments
    /// - `line_text`: The text content of the line.
    /// - `line_num`: The line number for display.
    /// - `app`: A mutable reference to the `ShellscapeApp` for syntax highlighting.
    ///
    /// # Returns
    /// A `Line` object with syntax highlighting applied.
    fn format_alert_line(
        &self,
        line_text: String,
        line_num: usize,
        app: &mut ShellscapeApp,
    ) -> Line {
        // Obtain syntax-highlighted ranges for the line.
        let ranges = app.highlighter(&line_text);
        let mut spans: Vec<Span> = ranges
            .iter()
            .map(|(style, line)| {
                Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    )),
                )
            })
            .collect();

        // Add decorative and numerical indicators.
        spans.insert(
            0,
            Span::styled(" \u{2503} ", Style::default().fg(self.deep_teal_color)),
        );

        spans.insert(
            0,
            Span::styled(
                format!("        {}", line_num),
                Style::default().fg(self.light_cream_color),
            ),
        );

        Line::from(spans)
    }

    /// Formats textual content, including a title and additional lines of content.
    ///
    /// # Arguments
    /// - `title`: The title of the content block.
    /// - `content`: A vector of strings containing the content lines.
    /// - `textwrap_width`: The width for wrapping content lines.
    ///
    /// # Returns
    /// A vector of `Line` objects for rendering the content.
    fn format_text_content(
        &self,
        title: String,
        content: Vec<String>,
        textwrap_width: usize,
    ) -> Vec<Line> {
        // Wrap and format the title text.
        let title_lines = textwrap::wrap(&title, textwrap_width.saturating_sub(4));
        let mut lines: Vec<Line> = title_lines
            .iter()
            .map(|t| {
                Line::from(Span::styled(
                    format!("    {}", t.to_string()).to_uppercase(),
                    Style::default().bold(),
                ))
                .alignment(Alignment::Left)
            })
            .collect();

        // Add spacing around the content block.
        lines.insert(0, Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));

        // Process each content item and format it.
        content.iter().for_each(|c| {
            let c_lines = textwrap::wrap(c, textwrap_width.saturating_sub(10));

            c_lines.iter().enumerate().for_each(|(i, l)| {
                let icon = if i == 0 {
                    format!("{}\u{27A9} ", " ".repeat(8))
                } else {
                    " ".repeat(8)
                };

                lines.push(
                    Line::from(Span::styled(
                        format!("{}{}", icon, l.to_string()),
                        Style::default(),
                    ))
                    .alignment(Alignment::Left),
                );
            });

            lines.push(Line::from(Span::raw("")));
        });

        lines
    }

    /// Formats shortcut elements into structured terminal lines.
    ///
    /// # Parameters
    /// - `shortcuts`: A vector of tuples containing shortcut keys and descriptions.
    /// - `textwrap_width`: The maximum width for text wrapping.
    ///
    /// # Returns
    /// A vector of `Line` objects representing the formatted shortcuts.
    fn format_shortcuts_elements(
        &self,
        shortcuts: Vec<(String, String)>,
        textwrap_width: usize,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Iterate through each shortcut and description pair.
        shortcuts.iter().for_each(|(shortcut, description)| {
            // Calculate the available width for description text after accounting for the shortcut.
            let shortcut_len = shortcut.len().saturating_add(9);
            let textwrap_width = textwrap_width.saturating_sub(shortcut_len);

            // Wrap the description text to fit within the calculated width.
            let description_lines = textwrap::wrap(&description, textwrap_width);
            let desc_lines_len = description_lines.len();

            // Insert an empty line for better readability if the description spans multiple lines.
            if desc_lines_len > 1 {
                lines.push(Line::from(Span::raw("")));
            }

            // Process each wrapped line of the description.
            description_lines.iter().enumerate().for_each(|(idx, d)| {
                let formatted_shortcut = if idx == 0 {
                    // Style and format the shortcut for the first line.
                    Span::styled(
                        format!("{}\u{27A9} {}   ", " ".repeat(4), shortcut),
                        Style::default().bold(),
                    )
                } else {
                    // Subsequent lines are indented to align with the shortcut.
                    Span::raw(" ".repeat(shortcut_len))
                };

                // Add the formatted shortcut and description line to the output.
                lines.push(Line::from(vec![
                    formatted_shortcut,
                    Span::styled(d.to_string(), Style::default()),
                ]));
            });

            // Add another empty line if the description had multiple lines.
            if desc_lines_len > 1 {
                lines.push(Line::from(Span::raw("")));
            }
        });

        lines
    }

    /// Creates an alert for a Galadriel-specific error.
    ///
    /// # Parameters
    /// - `time`: Timestamp for when the error occurred.
    /// - `error`: The `GaladrielError` instance containing error details.
    /// - `textwrap_width`: The maximum width for text wrapping.
    ///
    /// # Returns
    /// A vector of `Line` objects representing the error alert.
    fn create_galadriel_error_alert(
        &self,
        time: DateTime<Local>,
        error: GaladrielError,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Format the alert title with an icon and colors.
        let title = self.format_alert_title(
            &time,
            "\u{1F4A5}".to_string(),
            Color::LightRed,
            " GALADRIEL ERROR ".to_string(),
            Color::Red,
        );

        // Format the main error message.
        let mut message = self.format_alert_message(error.get_message(), textwrap_width as usize);

        // Format labels for error type and kind.
        let error_type = self.format_alert_label(
            "\u{1F535}".to_string(),
            "TYPE    ".to_string(),
            format!("{:?}", error.get_type()),
        );

        let error_kind = self.format_alert_label(
            "\u{1F7E0}".to_string(),
            "KIND    ".to_string(),
            format!("{:?}", error.get_kind()),
        );

        // Append title, message, and error details to the output.
        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));
        lines.push(error_type);
        lines.push(error_kind);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Creates a general informational alert.
    ///
    /// # Parameters
    /// - `time`: Timestamp for the alert.
    /// - `message`: The informational message content.
    /// - `textwrap_width`: The maximum width for text wrapping.
    ///
    /// # Returns
    /// A vector of `Line` objects representing the informational alert.
    fn create_information_alert(
        &self,
        time: DateTime<Local>,
        message: String,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Format the alert title with a blue color scheme.
        let title = self.format_alert_title(
            &time,
            "\u{1F535}".to_string(),
            Color::LightBlue,
            " INFORMATION ".to_string(),
            Color::Blue,
        );

        // Format the informational message.
        let mut message = self.format_alert_message(message, textwrap_width as usize);

        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Creates a Nenyr-specific syntax error alert.
    ///
    /// # Parameters
    /// - `time`: Timestamp for the error occurrence.
    /// - `error`: The `NenyrError` instance containing syntax error details.
    /// - `textwrap_width`: The maximum width for text wrapping.
    /// - `app`: The mutable reference to the `ShellscapeApp` instance.
    ///
    /// # Returns
    /// A vector of `Line` objects representing the syntax error alert.
    fn create_nenyr_error_alert(
        &self,
        time: DateTime<Local>,
        error: NenyrError,
        textwrap_width: u16,
        app: &mut ShellscapeApp,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Generate a styled title for the Nenyr error alert.
        let title = self.format_alert_title(
            &time,
            "\u{1F4A2}".to_string(),
            Color::Rgb(255, 99, 71),
            " NENYR SYNTAX ERROR ".to_string(),
            Color::Rgb(139, 0, 0),
        );

        // Retrieve the main error message.
        let mut message =
            self.format_alert_message(error.get_error_message(), textwrap_width as usize);

        // Append the formatted title and message.
        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));

        // Additional context (path, kind, etc.).
        let error_path = self.format_alert_label(
            "\u{1F534}".to_string(),
            "Path           ".to_string(),
            format!("{:?}", error.get_context_path()),
        );

        let error_kind = self.format_alert_label(
            "\u{1F7E0}".to_string(),
            "Kind           ".to_string(),
            format!("{:?}", error.get_error_kind()),
        );

        lines.push(error_path);
        lines.push(error_kind);

        // Add optional context information if present.
        if let Some(context_name) = error.get_context_name() {
            let error_context = self.format_alert_label(
                "\u{1F535}".to_string(),
                "Context Name   ".to_string(),
                format!("{:?}", context_name),
            );

            lines.push(error_context);
        }

        lines.push(Line::from(Span::raw("")));

        // Error position details.
        let error_line = self.format_alert_label(
            "\u{1F7E3}".to_string(),
            "Line           ".to_string(),
            format!("{:?}", error.get_line()),
        );

        let error_column = self.format_alert_label(
            "\u{1F7E1}".to_string(),
            "Column         ".to_string(),
            format!("{:?}", error.get_column()),
        );

        let error_position = self.format_alert_label(
            "\u{1F7E2}".to_string(),
            "Position       ".to_string(),
            format!("{:?}", error.get_position()),
        );

        lines.push(error_line);
        lines.push(error_column);
        lines.push(error_position);
        lines.push(Line::from(Span::raw("")));

        // Some additional information about the line before, error line and line after the error.
        if let Some(line_before) = error.get_line_before_error() {
            let line = self.format_alert_line(line_before, error.get_line().saturating_sub(1), app);

            lines.push(line);
        }

        if let Some(error_line) = error.get_error_line() {
            let line = self.format_alert_line(error_line, error.get_line(), app);

            lines.push(line);
        }

        if let Some(line_after) = error.get_line_after_error() {
            let line = self.format_alert_line(line_after, error.get_line().saturating_add(1), app);

            lines.push(line);
        }

        if let Some(suggestion) = error.get_suggestion() {
            lines.push(Line::from(Span::raw("")));

            let mut suggestion = self.format_alert_message(suggestion, textwrap_width as usize);
            lines.append(&mut suggestion);
        }

        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Creates a success alert with a formatted title, message, and duration information.
    ///
    /// # Arguments
    /// - `start_time`: The start time of the process.
    /// - `ending_time`: The ending time of the process.
    /// - `duration`: The time duration of the process.
    /// - `message`: The main content of the alert.
    /// - `textwrap_width`: Maximum width for text wrapping in the alert.
    ///
    /// # Returns
    /// - A vector of `Line` objects representing the success alert.
    fn create_success_alert(
        &self,
        start_time: DateTime<Local>,
        ending_time: DateTime<Local>,
        duration: TimeDelta,
        message: String,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Format the alert title with a success icon and green coloring.
        let title = self.format_alert_title(
            &start_time,
            "\u{2705}".to_string(),
            Color::LightGreen,
            " SUCCESS ".to_string(),
            Color::Green,
        );

        // Format the main message with text wrapping.
        let mut message = self.format_alert_message(message, textwrap_width as usize);

        // Add the formatted title and message to the list of lines.
        lines.push(title);
        lines.append(&mut message);

        // If a valid duration is provided, add timing information.
        if duration.num_milliseconds() > 0 {
            let end_time = self.date_time_formatter(&ending_time);
            let mut additional = self.format_alert_message(
                format!(
                    "The current process took **{}** ms to complete, finishing at **{}**",
                    duration.num_milliseconds(),
                    end_time
                ),
                textwrap_width as usize,
            );

            lines.append(&mut additional);
        }

        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Creates a warning alert with a formatted title and message.
    ///
    /// # Arguments
    /// - `time`: The timestamp for the warning.
    /// - `message`: The main content of the warning.
    /// - `textwrap_width`: Maximum width for text wrapping in the alert.
    ///
    /// # Returns
    /// - A vector of `Line` objects representing the warning alert.
    fn create_warning_alert(
        &self,
        time: DateTime<Local>,
        message: String,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Format the alert title with a warning icon and yellow coloring.
        let title = self.format_alert_title(
            &time,
            "\u{1F6A8}".to_string(),
            Color::LightYellow,
            " WARNING ".to_string(),
            Color::Yellow,
        );

        // Format the main message with text wrapping.
        let mut message = self.format_alert_message(message, textwrap_width as usize);

        // Add the formatted title and message to the list of lines.
        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Creates a text-based alert with a formatted title, content, and type.
    ///
    /// # Arguments
    /// - `time`: The timestamp for the alert.
    /// - `title`: The main title for the alert.
    /// - `content`: A list of strings representing the content.
    /// - `kind`: The type of alert (e.g., about author, contribute, etc.).
    /// - `textwrap_width`: Maximum width for text wrapping in the alert.
    ///
    /// # Returns
    /// - A vector of `Line` objects representing the text-based alert.
    fn create_text_alert(
        &self,
        time: DateTime<Local>,
        title: String,
        content: Vec<String>,
        kind: AlertTextType,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Define colors and labels based on the alert type.
        let (time_fg, label, title_bg, icon) = match kind {
            AlertTextType::AboutAuthor => (
                Color::Rgb(135, 206, 250),
                "About the Author",
                Color::Rgb(25, 80, 120),
                "\u{1F4DA}",
            ),
            AlertTextType::ContributeAsDev => (
                Color::Rgb(144, 238, 144),
                "Contribute as Dev",
                Color::Rgb(34, 139, 34),
                "\u{1F9F0}",
            ),
            AlertTextType::Donation => (
                Color::Rgb(255, 200, 124),
                "Make a Donation",
                Color::Rgb(210, 105, 30),
                "\u{1F49B}",
            ),
            AlertTextType::License => (
                Color::Rgb(186, 85, 211),
                "License",
                Color::Rgb(75, 0, 130),
                "\u{1F511}",
            ),
        };

        // Format the content with text wrapping.
        let label_element = self.format_alert_title(
            &time,
            icon.to_string(),
            time_fg,
            format!(" {} ", label),
            title_bg,
        );

        // Format the content with text wrapping.
        let mut text = self.format_text_content(title, content, textwrap_width as usize);

        // Add the formatted title and content to the list of lines.
        lines.push(label_element);
        lines.append(&mut text);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// Creates an alert listing all keyboard shortcuts.
    ///
    /// # Arguments
    /// - `time`: The timestamp for the alert.
    /// - `shortcuts`: A list of tuples, each representing a shortcut and its description.
    /// - `textwrap_width`: Maximum width for text wrapping in the alert.
    ///
    /// # Returns
    /// - A vector of `Line` objects representing the shortcuts alert.
    fn create_shortcuts_alert(
        &self,
        time: DateTime<Local>,
        shortcuts: Vec<(String, String)>,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Format the title with a shortcuts icon and gray coloring.
        let title = self.format_alert_title(
            &time,
            "\u{1F4BB}".to_string(),
            Color::Rgb(245, 245, 245),
            " SHORTCUTS ".to_string(),
            Color::Rgb(129, 129, 128),
        );

        // Format the list of shortcuts with text wrapping.
        let mut shortcuts = self.format_shortcuts_elements(shortcuts, textwrap_width as usize);

        // Add the formatted title and shortcuts to the list of lines.
        lines.push(title);
        lines.append(&mut shortcuts);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    /// This method processes a list of alerts and returns a vector of `Line` elements that
    /// can be displayed in the terminal UI. It handles different types of alerts by formatting
    /// them accordingly and adding them to the `lines` vector.
    ///
    /// The `textwrap_width` is used to determine the maximum width of the text in each alert,
    /// ensuring that the content fits within the specified width.
    ///
    /// # Arguments
    /// - `textwrap_width: u16` - The width to which the alert texts will be wrapped.
    /// - `alerts: Vec<ShellscapeAlerts>` - A vector containing the alerts to be processed.
    /// - `app: &mut ShellscapeApp` - The application instance which provides alert data.
    ///
    /// # Returns
    /// - `Vec<Line>` - A vector of `Line` elements containing the formatted alerts to be displayed.
    fn process_alerts(
        &self,
        textwrap_width: u16,
        alerts: Vec<ShellscapeAlerts>,
        app: &mut ShellscapeApp,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        // Check if there are no alerts to display, and if so, show a default message
        if alerts.len() == 0 {
            lines.push(
                Line::from(vec![Span::styled(
                    "There are currently no alerts to display at this time.",
                    Style::default().fg(self.deep_teal_color),
                )])
                .alignment(Alignment::Center),
            );
        }

        // Loop through each alert and process it depending on its type
        for alert in alerts {
            match alert {
                // Process a GaladrielError alert (likely related to a Galadriel-related process)
                ShellscapeAlerts::GaladrielError { start_time, error } => {
                    let mut elements =
                        self.create_galadriel_error_alert(start_time, error, textwrap_width);

                    lines.append(&mut elements);
                }
                // Process an Information alert (general information message)
                ShellscapeAlerts::Information {
                    start_time,
                    message,
                } => {
                    let mut elements =
                        self.create_information_alert(start_time, message, textwrap_width);

                    lines.append(&mut elements);
                }
                // Process a NenyrError alert (likely related to the Nenyr parsing)
                ShellscapeAlerts::NenyrError { start_time, error } => {
                    let mut elements =
                        self.create_nenyr_error_alert(start_time, error, textwrap_width, app);

                    lines.append(&mut elements);
                }
                // Process a Success alert (successful operation with details)
                ShellscapeAlerts::Success {
                    start_time,
                    ending_time,
                    duration,
                    message,
                } => {
                    let mut elements = self.create_success_alert(
                        start_time,
                        ending_time,
                        duration,
                        message,
                        textwrap_width,
                    );

                    lines.append(&mut elements);
                }
                // Process a Warning alert (general warning message)
                ShellscapeAlerts::Warning {
                    start_time,
                    message,
                } => {
                    let mut elements =
                        self.create_warning_alert(start_time, message, textwrap_width);

                    lines.append(&mut elements);
                }
                // Process a Text alert (custom message with a title and content)
                ShellscapeAlerts::Text {
                    start_time,
                    title,
                    content,
                    kind,
                } => {
                    let mut elements =
                        self.create_text_alert(start_time, title, content, kind, textwrap_width);

                    lines.append(&mut elements);
                }
                // Process a Shortcuts alert (displays a list of keyboard shortcuts)
                ShellscapeAlerts::Shortcuts {
                    start_time,
                    shortcuts,
                } => {
                    let mut elements =
                        self.create_shortcuts_alert(start_time, shortcuts, textwrap_width);

                    lines.append(&mut elements);
                }
            }
        }

        lines
    }

    /// This method creates and returns a `Paragraph` containing the alerts formatted as `Line` elements.
    /// It uses `process_alerts` to retrieve the alert lines, and it sets up the UI component for displaying them.
    ///
    /// It also returns the total number of alert lines for potential use elsewhere in the UI.
    ///
    /// # Arguments
    /// - `textwrap_width: u16` - The width to which the alert texts will be wrapped.
    /// - `app: &mut ShellscapeApp` - The application instance from which alerts will be fetched.
    ///
    /// # Returns
    /// - `(Paragraph, usize)` - A tuple containing:
    ///     - A `Paragraph` UI component ready for display.
    ///     - The number of lines of alerts created.
    fn create_alerts_table(
        &self,
        textwrap_width: u16,
        app: &mut ShellscapeApp,
    ) -> (Paragraph, usize) {
        // Retrieve alerts from the application instance
        let alerts = app.get_alerts();
        // Process the alerts to get a vector of formatted lines
        let lines = self.process_alerts(textwrap_width, alerts, app);
        // Get the total number of lines
        let lines_len = lines.len();

        // Create a Paragraph widget to display the alerts with custom styling
        let element = Paragraph::new(lines)
            .bg(self.foreground_color)
            .scroll((app.get_table_vertical_axis(), 0))
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 1, 1))
                    .fg(self.off_white_color),
            );

        // Return the paragraph element and the total number of lines
        (element, lines_len)
    }

    /// Creates the main table layout for the terminal UI.
    /// The table is split into two main sections: `dock` and `table`.
    /// The `dock` is allocated 25% of the available width, while the `table` takes up the remaining 75%.
    ///
    /// # Arguments
    /// * `table` - A `Rect` representing the full area where the table will be rendered.
    /// * `frame` - A mutable reference to the `Frame` used for rendering widgets.
    /// * `app` - A mutable reference to the `ShellscapeApp`, which holds the app state.
    ///
    /// This method:
    /// 1. Splits the terminal area into two parts: `dock` and `table`.
    /// 2. Defines a specific width for the `dock` and adjusts the `table` width.
    /// 3. Creates two areas: one for the `dock` and another for the `table`.
    /// 4. Resets the areas and scroll states before rendering the widgets.
    /// 5. Renders the dock and table, along with their corresponding scrollbars.
    fn create_table(&self, table: Rect, frame: &mut Frame, app: &mut ShellscapeApp) {
        // Layout is split into two parts: 25% for the dock and 75% for the table.
        let container = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(table);

        // `dock` is the first part, taking up 25% of the available space.
        let dock = container[0];
        // `table` takes the remaining 75% space.
        let table = container[1];

        // Set the width for dock and adjust text wrap width for table (leaving space for scrollbars).
        let dock_width = dock.width;
        let textwrap_width = table.width.saturating_sub(10);

        // Create areas for both the dock and the table
        let dock_are = ShellscapeArea::new(dock.left(), dock.right(), dock.top(), dock.bottom());
        let table_area =
            ShellscapeArea::new(table.left(), table.right(), table.top(), table.bottom());

        // Reset the area definitions in the app state
        app.reset_dock_area(dock_are);
        app.reset_table_area(table_area);

        // Generate the dock widget and calculate its length
        let (dock_element, dock_len) = self.create_dock(dock_width, app);

        // Reset scroll state for the dock and render the dock widget
        app.reset_dock_scroll_state(dock_len);
        frame.render_widget(dock_element, dock);

        // Render a vertical scrollbar for the dock
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("\u{25B4}"))
                .end_symbol(Some("\u{25BE}"))
                .track_symbol(Some("\u{2503}"))
                .end_style(self.primary_color)
                .begin_style(self.primary_color)
                .track_style(self.primary_color)
                .thumb_style(self.deep_teal_color),
            dock,
            &mut app.dock_scroll_state,
        );

        // Generate the table widget and calculate the alerts length
        let (table_element, alerts_len) = self.create_alerts_table(textwrap_width, app);

        // Reset scroll state for the table and render the table widget
        app.reset_table_scroll_state(alerts_len);
        frame.render_widget(table_element, table);

        // Render a vertical scrollbar for the table
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("\u{25B4}"))
                .end_symbol(Some("\u{25BE}"))
                .track_symbol(Some("\u{2503}"))
                .end_style(self.primary_color)
                .begin_style(self.primary_color)
                .track_style(self.primary_color)
                .thumb_style(self.deep_teal_color),
            table,
            &mut app.table_scroll_state,
        );
    }

    /// Creates a footer widget for the terminal UI, which displays some footer text.
    ///
    /// # Arguments
    /// * `app` - A mutable reference to the `ShellscapeApp` where the footer text is retrieved from.
    ///
    /// Returns a `Paragraph` widget displaying the footer text in the terminal.
    fn create_footer(&self, app: &mut ShellscapeApp) -> Paragraph {
        // Retrieve the footer text from the app state.
        let footer_text = app.get_footer();
        // Apply styling to the footer text (light cream color).
        let footer = Span::styled(footer_text, Style::default().fg(self.light_cream_color));

        // Create a Paragraph widget with the footer text and apply styling (alignment, background, padding).
        Paragraph::new(footer)
            .alignment(Alignment::Center)
            .bg(self.primary_color)
            .block(
                Block::default()
                    .padding(Padding::vertical(1))
                    .fg(self.light_cream_color),
            )
    }

    /// Formats a given `DateTime` into a string with the format `HH:MM:SS.mmm`.
    ///
    /// # Arguments
    /// * `time` - A reference to a `DateTime<Local>` object that represents the current date and time.
    ///
    /// Returns a formatted string representing the time in hours, minutes, seconds, and milliseconds
    fn date_time_formatter(&self, time: &DateTime<Local>) -> String {
        time.format("%H:%M:%S.%3f").to_string()
    }
}
