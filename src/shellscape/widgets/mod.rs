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

use super::{alerts::ShellscapeAlerts, app::ShellscapeApp, area::ShellscapeArea};

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

    pub fn paint(&self, frame: &mut Frame, app: &mut ShellscapeApp) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(72),
                Constraint::Percentage(8),
            ])
            .split(frame.area());

        let header_width = layout[0].width;
        let header = self.create_header(header_width, app);
        frame.render_widget(header, layout[0]);

        self.create_table(layout[1], frame, app);

        let footer = self.create_footer(app);
        frame.render_widget(footer, layout[2]);
    }

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

    fn create_header(&self, header_width: u16, app: &mut ShellscapeApp) -> Paragraph {
        let title = format!(" {} ", app.get_title().to_uppercase());
        let mut elements = vec![];

        let author = self.create_metadata(app);
        elements.push(Line::from(author));

        let blank_line = Line::from(Span::raw(""));
        elements.push(blank_line);

        let width = header_width.saturating_div(10).saturating_mul(6) as usize;
        let mut subtitle = self.create_subtitle(width, app);
        elements.append(&mut subtitle);

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

    fn format_exclude_vec(&self, title: String, value: Vec<String>, dock_width: u16) -> Vec<Line> {
        let mut lines = vec![];
        let title = Line::from(vec![
            Span::raw(" \u{1F7E5}  "),
            Span::styled(
                format!("{}:", title,),
                Style::default().bold().fg(self.off_white_color),
            ),
        ]);

        lines.push(title);

        value.iter().for_each(|v| {
            let textwrap_width = dock_width.saturating_sub(15) as usize;
            let parts = textwrap::wrap(v, Options::new(textwrap_width));
            let parts_len = parts.len().saturating_sub(1);

            parts.iter().enumerate().for_each(|(i, p)| {
                let part = p.to_string();
                let part_len = part.len().saturating_add(16);
                let repeat = dock_width.saturating_sub(part_len as u16) as usize;

                let open_str = if i == 0 { "   \u{1F538} \"" } else { "      " };
                let closing_str = if i == parts_len { "\"" } else { "" };

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

    fn create_configs_viewer(&self, dock_width: u16, configs: Configatron, port: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let configs_title = self.format_block_title(" Configuration".to_string(), dock_width);
        lines.push(configs_title);
        lines.push(Line::from(Span::raw("")));

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

    fn create_adjustment_options(&self, dock_width: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let configs_title = self.format_block_title("Adjustments".to_string(), dock_width);

        let toggle_reset = self.format_dock_option(
            "Toggle R. Styles".to_string(),
            "'Shift' + 'R'".to_string(),
            dock_width,
        );

        let toggle_minified = self.format_dock_option(
            "Toggle M. Styles".to_string(),
            "'Shift' + 'M'".to_string(),
            dock_width,
        );

        let toggle_naming = self.format_dock_option(
            "Toggle A. Naming".to_string(),
            "'Shift' + 'N'".to_string(),
            dock_width,
        );

        let change_version = self.format_dock_option(
            "Modify Version".to_string(),
            "'Shift' + 'V'".to_string(),
            dock_width,
        );

        let exclude = self.format_dock_option(
            "Adjust Exclude".to_string(),
            "'Shift' + 'E'".to_string(),
            dock_width,
        );

        lines.push(configs_title);
        lines.push(Line::from(Span::raw("")));
        lines.push(toggle_reset);
        lines.push(toggle_minified);
        lines.push(toggle_naming);
        lines.push(change_version);
        lines.push(exclude);

        lines
    }

    fn create_extra_options(&self, dock_width: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let configs_title = self.format_block_title(" Options".to_string(), dock_width);

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

    fn create_separator(&self, dock_width: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw(
            "\u{25E6}".repeat(dock_width.saturating_sub(3) as usize),
        )));
        lines.push(Line::from(Span::raw("")));

        lines
    }

    fn format_dock_settings(&self, dock_width: u16, configs: Configatron, port: u16) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let mut configs_viewer = self.create_configs_viewer(dock_width, configs, port);
        lines.append(&mut configs_viewer);

        let mut configs_separator = self.create_separator(dock_width);
        lines.append(&mut configs_separator);

        let mut adjustment_options = self.create_adjustment_options(dock_width);
        lines.append(&mut adjustment_options);

        let mut adjustment_separator = self.create_separator(dock_width);
        lines.append(&mut adjustment_separator);

        let mut extra_options = self.create_extra_options(dock_width);
        lines.append(&mut extra_options);

        lines.push(Line::from(Span::raw("")));

        lines
    }

    fn create_dock(&self, dock_width: u16, app: &mut ShellscapeApp) -> (Paragraph, usize) {
        let lines: Vec<Line> = self.format_dock_settings(
            dock_width,
            app.get_configs(),
            app.get_server_running_on_port(),
        );
        let lines_len = lines.len();
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

    fn format_alert_title(
        &self,
        time: &DateTime<Local>,
        icon: String,
        time_fg: Color,
        title: String,
        title_bg: Color,
    ) -> Line {
        let time = self.date_time_formatter(time);
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

    fn format_alert_message(&self, message: String, textwrap_width: usize) -> Vec<Line> {
        let message = format!("\u{25C7} {}", message);
        let message_lines = textwrap::wrap(&message, textwrap_width);

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

    fn format_alert_line(
        &self,
        line_text: String,
        line_num: usize,
        app: &mut ShellscapeApp,
    ) -> Line {
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

    fn create_galadriel_error_alert(
        &self,
        time: DateTime<Local>,
        error: GaladrielError,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let title = self.format_alert_title(
            &time,
            "\u{1F4A5}".to_string(),
            Color::LightRed,
            " GALADRIEL ERROR ".to_string(),
            Color::Red,
        );

        let mut message = self.format_alert_message(error.get_message(), textwrap_width as usize);
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

        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));
        lines.push(error_type);
        lines.push(error_kind);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    fn create_information_alert(
        &self,
        time: DateTime<Local>,
        message: String,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let title = self.format_alert_title(
            &time,
            "\u{1F535}".to_string(),
            Color::LightBlue,
            " INFORMATION ".to_string(),
            Color::Blue,
        );

        let mut message = self.format_alert_message(message, textwrap_width as usize);

        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    fn create_nenyr_error_alert(
        &self,
        time: DateTime<Local>,
        error: NenyrError,
        textwrap_width: u16,
        app: &mut ShellscapeApp,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let title = self.format_alert_title(
            &time,
            "\u{1F4A2}".to_string(),
            Color::Rgb(255, 99, 71),
            " NENYR SYNTAX ERROR ".to_string(),
            Color::Rgb(139, 0, 0),
        );

        let mut message =
            self.format_alert_message(error.get_error_message(), textwrap_width as usize);

        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));

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

        if let Some(context_name) = error.get_context_name() {
            let error_context = self.format_alert_label(
                "\u{1F535}".to_string(),
                "Context Name   ".to_string(),
                format!("{:?}", context_name),
            );

            lines.push(error_context);
        }

        lines.push(Line::from(Span::raw("")));

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

    fn create_success_alert(
        &self,
        start_time: DateTime<Local>,
        ending_time: DateTime<Local>,
        duration: TimeDelta,
        message: String,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let title = self.format_alert_title(
            &start_time,
            "\u{2705}".to_string(),
            Color::LightGreen,
            " SUCCESS ".to_string(),
            Color::Green,
        );

        let mut message = self.format_alert_message(message, textwrap_width as usize);

        lines.push(title);
        lines.append(&mut message);

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

    fn create_warning_alert(
        &self,
        time: DateTime<Local>,
        message: String,
        textwrap_width: u16,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let title = self.format_alert_title(
            &time,
            "\u{1F6A8}".to_string(),
            Color::LightYellow,
            " WARNING ".to_string(),
            Color::Yellow,
        );

        let mut message = self.format_alert_message(message, textwrap_width as usize);

        lines.push(title);
        lines.append(&mut message);
        lines.push(Line::from(Span::raw("")));

        lines
    }

    fn process_alerts(
        &self,
        textwrap_width: u16,
        alerts: Vec<ShellscapeAlerts>,
        app: &mut ShellscapeApp,
    ) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        if alerts.len() == 0 {
            lines.push(
                Line::from(vec![Span::styled(
                    "There are currently no alerts to display at this time.",
                    Style::default().fg(self.deep_teal_color),
                )])
                .alignment(Alignment::Center),
            );
        }

        for alert in alerts {
            match alert {
                ShellscapeAlerts::GaladrielError { start_time, error } => {
                    let mut elements =
                        self.create_galadriel_error_alert(start_time, error, textwrap_width);

                    lines.append(&mut elements);
                }
                ShellscapeAlerts::Information {
                    start_time,
                    message,
                } => {
                    let mut elements =
                        self.create_information_alert(start_time, message, textwrap_width);

                    lines.append(&mut elements);
                }
                ShellscapeAlerts::NenyrError { start_time, error } => {
                    let mut elements =
                        self.create_nenyr_error_alert(start_time, error, textwrap_width, app);

                    lines.append(&mut elements);
                }
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
                ShellscapeAlerts::Warning {
                    start_time,
                    message,
                } => {
                    let mut elements =
                        self.create_warning_alert(start_time, message, textwrap_width);

                    lines.append(&mut elements);
                }
            }
        }

        lines
    }

    fn create_alerts_table(
        &self,
        textwrap_width: u16,
        app: &mut ShellscapeApp,
    ) -> (Paragraph, usize) {
        let alerts = app.get_alerts();
        let lines = self.process_alerts(textwrap_width, alerts, app);
        let lines_len = lines.len();
        let element = Paragraph::new(lines)
            .bg(self.foreground_color)
            .scroll((app.get_table_vertical_axis(), 0))
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 1, 1))
                    .fg(self.off_white_color),
            );

        (element, lines_len)
    }

    fn create_table(&self, table: Rect, frame: &mut Frame, app: &mut ShellscapeApp) {
        let container = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(table);

        let dock = container[0];
        let table = container[1];

        let dock_width = dock.width;
        let textwrap_width = table.width.saturating_sub(10);

        let dock_are = ShellscapeArea::new(dock.left(), dock.right(), dock.top(), dock.bottom());
        let table_area =
            ShellscapeArea::new(table.left(), table.right(), table.top(), table.bottom());

        app.reset_dock_area(dock_are);
        app.reset_table_area(table_area);

        let (dock_element, dock_len) = self.create_dock(dock_width, app);

        app.reset_dock_scroll_state(dock_len);
        frame.render_widget(dock_element, dock);
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

        let (table_element, alerts_len) = self.create_alerts_table(textwrap_width, app);

        app.reset_table_scroll_state(alerts_len);
        frame.render_widget(table_element, table);
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

    fn create_footer(&self, app: &mut ShellscapeApp) -> Paragraph {
        let footer_text = app.get_footer();
        let footer = Span::styled(footer_text, Style::default().fg(self.light_cream_color));

        Paragraph::new(footer)
            .alignment(Alignment::Center)
            .bg(self.primary_color)
            .block(
                Block::default()
                    .padding(Padding::vertical(1))
                    .fg(self.light_cream_color),
            )
    }

    fn date_time_formatter(&self, time: &DateTime<Local>) -> String {
        time.format("%H:%M:%S.%3f").to_string()
    }
}
