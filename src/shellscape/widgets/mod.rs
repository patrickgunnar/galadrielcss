use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};

use textwrap::Options;

use super::{app::ShellscapeApp, area::ShellscapeArea, notifications::ShellscapeNotifications};

#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeWidgets {}

/*
let primary_color = Color::Rgb(120, 120, 120);
let secondary_color = Color::Rgb(55, 55, 55);
let foreground_color = Color::Rgb(10, 10, 10);
let thumb_color = Color::Rgb(145, 145, 145);
*/

impl ShellscapeWidgets {
    pub fn paint(frame: &mut Frame, shellscape_app: &mut ShellscapeApp) {
        let primary_color = Color::Rgb(50, 70, 60);
        let secondary_color = Color::Rgb(0, 35, 35);
        let foreground_color = Color::Rgb(5, 10, 10);
        let light_cream = Color::Rgb(240, 240, 240);
        let deep_teal = Color::Rgb(0, 95, 95);
        let off_white = Color::Rgb(250, 250, 250);

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(72),
                Constraint::Percentage(8),
            ])
            .split(frame.area());

        let mut header_elements = vec![];
        let author_element = Line::from(Span::styled(
            format!(
                "Author: {} | License: {} | Version: {}",
                shellscape_app.get_author(),
                shellscape_app.get_license(),
                shellscape_app.get_version()
            ),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(light_cream),
        ));

        let subtitle = shellscape_app.get_subtitle();

        let subtitle_lines = textwrap::wrap(&subtitle, Options::new(author_element.width()));

        header_elements.push(author_element);
        header_elements.push(Line::from(Span::styled("", Style::default())));

        for borrowed_line in subtitle_lines {
            let line = borrowed_line.to_string();

            header_elements.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(light_cream),
            )));
        }

        let title = format!(" {} ", shellscape_app.get_title().to_uppercase());
        let header = Paragraph::new(header_elements)
            .alignment(Alignment::Center)
            .bg(primary_color)
            .block(
                Block::default()
                    .padding(Padding::top(1))
                    .title(Span::styled(
                        title,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(light_cream),
                    ))
                    .borders(Borders::ALL)
                    .fg(secondary_color)
                    .title_alignment(Alignment::Center),
            );

        frame.render_widget(header, main_layout[0]);

        let content_container = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(main_layout[1]);

        let dock = content_container[0];
        let notify = content_container[1];

        let dock_area = ShellscapeArea::new(dock.left(), dock.right(), dock.top(), dock.bottom());
        let notifications_area =
            ShellscapeArea::new(notify.left(), notify.right(), notify.top(), notify.bottom());

        shellscape_app.reset_dock_area(dock_area);
        shellscape_app.reset_notifications_area(notifications_area);

        let mut content_elements = vec![];
        let textwrap_width = Options::new(content_container[1].width.saturating_sub(10) as usize);

        let notifications = shellscape_app.get_notifications();

        for notification in &notifications {
            match notification {
                ShellscapeNotifications::GaladrielError { start_time, error } => {
                    let time = Self::format_time(start_time);
                    let time_element = Line::from(vec![
                        Span::raw("\u{1F4A5}"),
                        Span::styled(
                            format!(" {}", time),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::LightRed),
                        ),
                        Span::styled(" \u{2022} ", Style::default()),
                        Span::styled(
                            " GALADRIEL ERROR ",
                            Style::default()
                                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                                .bg(Color::Red),
                        ),
                    ]);

                    content_elements.push(time_element);

                    let message = format!("\u{25C6} {}", error.get_message());
                    let message_lines = textwrap::wrap(&message, &textwrap_width);

                    for borrowed_line in message_lines {
                        let mut spans = vec![Span::raw("    ")];
                        let line = borrowed_line.to_string();

                        let parts: Vec<String> = line.split("**").map(|v| v.to_string()).collect();

                        for (idx, part) in parts.into_iter().enumerate() {
                            if idx % 2 == 1 {
                                spans.push(Span::styled(
                                    part,
                                    Style::default().add_modifier(Modifier::BOLD),
                                ));
                            } else {
                                spans.push(Span::raw(part));
                            }
                        }

                        content_elements.push(Line::from(spans));
                    }

                    content_elements.push(Line::from(Span::styled("", Style::default())));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F539} ", Style::default()),
                        Span::styled("TYPE    ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" {:?}", error.get_type()), Style::default()),
                    ]));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F538} ", Style::default()),
                        Span::styled("KIND    ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" {:?}", error.get_kind()), Style::default()),
                    ]));
                }
                ShellscapeNotifications::Information {
                    start_time,
                    message,
                } => {
                    let time = Self::format_time(start_time);
                    let time_element = Line::from(vec![
                        Span::raw("\u{1F535}"),
                        Span::styled(
                            format!(" {}", time),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::LightBlue),
                        ),
                        Span::styled(" \u{2022} ", Style::default()),
                        Span::styled(
                            " INFORMATION ",
                            Style::default()
                                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                                .bg(Color::Blue),
                        ),
                    ]);

                    content_elements.push(time_element);

                    let message = format!("\u{25C6} {}", message);
                    let message_lines = textwrap::wrap(&message, &textwrap_width);

                    for borrowed_line in message_lines {
                        let mut spans = vec![Span::raw("    ")];
                        let line = borrowed_line.to_string();

                        let parts: Vec<String> = line.split("**").map(|v| v.to_string()).collect();

                        for (idx, part) in parts.into_iter().enumerate() {
                            if idx % 2 == 1 {
                                spans.push(Span::styled(
                                    part,
                                    Style::default().add_modifier(Modifier::BOLD),
                                ));
                            } else {
                                spans.push(Span::raw(part));
                            }
                        }

                        content_elements.push(Line::from(spans));
                    }
                }
                ShellscapeNotifications::NenyrError { start_time, error } => {
                    let time = Self::format_time(start_time);
                    let time_element = Line::from(vec![
                        Span::raw("\u{1F4A2}"),
                        Span::styled(
                            format!(" {}", time),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Rgb(255, 99, 71)),
                        ),
                        Span::styled(" \u{2022} ", Style::default()),
                        Span::styled(
                            " NENYR SYNTAX ERROR ",
                            Style::default()
                                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                                .bg(Color::Rgb(139, 0, 0)),
                        ),
                    ]);

                    content_elements.push(time_element);

                    let message = format!("\u{25C6} {}", error.get_error_message());
                    let message_lines = textwrap::wrap(&message, &textwrap_width);

                    for borrowed_line in message_lines {
                        let mut spans = vec![Span::raw("    ")];
                        let line = borrowed_line.to_string();

                        let parts: Vec<String> = line.split("**").map(|v| v.to_string()).collect();

                        for (idx, part) in parts.into_iter().enumerate() {
                            if idx % 2 == 1 {
                                spans.push(Span::styled(
                                    part,
                                    Style::default().add_modifier(Modifier::BOLD),
                                ));
                            } else {
                                spans.push(Span::raw(part));
                            }
                        }

                        content_elements.push(Line::from(spans));
                    }

                    content_elements.push(Line::from(Span::styled("", Style::default())));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F534} ", Style::default()),
                        Span::styled(
                            "Path           ",
                            Style::default().add_modifier(Modifier::BOLD).fg(deep_teal),
                        ),
                        Span::styled(
                            format!(" {:?}", error.get_context_path()),
                            Style::default().fg(light_cream),
                        ),
                    ]));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F7E0} ", Style::default()),
                        Span::styled(
                            "Kind           ",
                            Style::default().add_modifier(Modifier::BOLD).fg(deep_teal),
                        ),
                        Span::styled(
                            format!(" {:?}", error.get_error_kind()),
                            Style::default().fg(light_cream),
                        ),
                    ]));

                    if let Some(context_name) = error.get_context_name() {
                        content_elements.push(Line::from(vec![
                            Span::styled("        \u{1F535} ", Style::default()),
                            Span::styled(
                                "Context Name   ",
                                Style::default().add_modifier(Modifier::BOLD).fg(deep_teal),
                            ),
                            Span::styled(
                                format!(" {:?}", context_name),
                                Style::default().fg(light_cream),
                            ),
                        ]));
                    }

                    content_elements.push(Line::from(Span::styled("", Style::default())));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F7E3} ", Style::default()),
                        Span::styled(
                            "Line           ",
                            Style::default().add_modifier(Modifier::BOLD).fg(deep_teal),
                        ),
                        Span::styled(
                            format!(" {:?}", error.get_line()),
                            Style::default().fg(light_cream),
                        ),
                    ]));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F7E1} ", Style::default()),
                        Span::styled(
                            "Column         ",
                            Style::default().add_modifier(Modifier::BOLD).fg(deep_teal),
                        ),
                        Span::styled(
                            format!(" {:?}", error.get_column()),
                            Style::default().fg(light_cream),
                        ),
                    ]));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F7E2} ", Style::default()),
                        Span::styled(
                            "Position       ",
                            Style::default().add_modifier(Modifier::BOLD).fg(deep_teal),
                        ),
                        Span::styled(
                            format!(" {:?}", error.get_position()),
                            Style::default().fg(light_cream),
                        ),
                    ]));

                    content_elements.push(Line::from(Span::styled("", Style::default())));

                    if let Some(line_before) = error.get_line_before_error() {
                        let ranges = shellscape_app.highlighter(&line_before);
                        let mut spans = vec![
                            Span::styled(
                                format!("        {}", error.get_line().saturating_sub(1)),
                                Style::default().fg(light_cream),
                            ),
                            Span::styled(" \u{2503} ", Style::default().fg(deep_teal)),
                        ];

                        for (style, text) in ranges {
                            spans.push(Span::styled(
                                text,
                                Style::default().fg(Color::Rgb(
                                    style.foreground.r,
                                    style.foreground.g,
                                    style.foreground.b,
                                )),
                            ));
                        }

                        content_elements.push(Line::from(spans));
                    }

                    if let Some(error_line) = error.get_error_line() {
                        let ranges = shellscape_app.highlighter(&error_line);
                        let mut spans = vec![
                            Span::styled(
                                format!("        {}", error.get_line()),
                                Style::default().fg(light_cream),
                            ),
                            Span::styled(" \u{2503} ", Style::default().fg(deep_teal)),
                        ];

                        for (style, text) in ranges {
                            spans.push(Span::styled(
                                text,
                                Style::default().fg(Color::Rgb(
                                    style.foreground.r,
                                    style.foreground.g,
                                    style.foreground.b,
                                )),
                            ));
                        }

                        content_elements.push(Line::from(spans));
                    }

                    if let Some(line_after) = error.get_line_after_error() {
                        let ranges = shellscape_app.highlighter(&line_after);
                        let mut spans = vec![
                            Span::styled(
                                format!("        {}", error.get_line().saturating_add(1)),
                                Style::default().fg(light_cream),
                            ),
                            Span::styled(" \u{2503} ", Style::default().fg(deep_teal)),
                        ];

                        for (style, text) in ranges {
                            spans.push(Span::styled(
                                text,
                                Style::default().fg(Color::Rgb(
                                    style.foreground.r,
                                    style.foreground.g,
                                    style.foreground.b,
                                )),
                            ));
                        }

                        content_elements.push(Line::from(spans));
                    }

                    if let Some(suggestion) = error.get_suggestion() {
                        content_elements.push(Line::from(Span::styled("", Style::default())));

                        let message = format!("\u{1F4A1} {}", suggestion);
                        let message_lines = textwrap::wrap(&message, &textwrap_width);

                        for borrowed_line in message_lines {
                            let mut spans = vec![Span::raw("    ")];
                            let line = borrowed_line.to_string();

                            let parts: Vec<String> =
                                line.split("**").map(|v| v.to_string()).collect();

                            for (idx, part) in parts.into_iter().enumerate() {
                                if idx % 2 == 1 {
                                    spans.push(Span::styled(
                                        part,
                                        Style::default().add_modifier(Modifier::BOLD),
                                    ));
                                } else {
                                    spans.push(Span::raw(part));
                                }
                            }

                            content_elements.push(Line::from(spans));
                        }
                    }
                }
                ShellscapeNotifications::Success {
                    start_time,
                    ending_time,
                    duration,
                    message,
                } => {
                    let start_time = Self::format_time(start_time);
                    let end_time = Self::format_time(ending_time);

                    let start_time_element = Line::from(vec![
                        Span::raw("\u{2705}"),
                        Span::styled(
                            format!(" {}", start_time),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::LightGreen),
                        ),
                        Span::styled(" \u{2022} ", Style::default()),
                        Span::styled(
                            " SUCCESS ",
                            Style::default()
                                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                                .bg(Color::Green),
                        ),
                    ]);

                    content_elements.push(start_time_element);

                    let message = format!("\u{25C6} {}", message);
                    let message_lines = textwrap::wrap(&message, &textwrap_width);

                    for borrowed_line in message_lines {
                        let mut spans = vec![Span::raw("    ")];
                        let line = borrowed_line.to_string();

                        let parts: Vec<String> = line.split("**").map(|v| v.to_string()).collect();

                        for (idx, part) in parts.into_iter().enumerate() {
                            if idx % 2 == 1 {
                                spans.push(Span::styled(
                                    part,
                                    Style::default().add_modifier(Modifier::BOLD),
                                ));
                            } else {
                                spans.push(Span::raw(part));
                            }
                        }

                        content_elements.push(Line::from(spans));
                    }

                    let message = format!(
                        "\u{25C6} The current process took **{}** ms to complete, finishing at **{}**",
                        duration.num_milliseconds(),
                        end_time
                    );

                    let message_lines = textwrap::wrap(message.as_str(), &textwrap_width);

                    for borrowed_line in message_lines {
                        let mut spans = vec![Span::raw("    ")];
                        let line = borrowed_line.to_string();
                        let parts: Vec<String> = line.split("**").map(|v| v.to_string()).collect();

                        for (idx, part) in parts.into_iter().enumerate() {
                            if idx % 2 == 1 {
                                spans.push(Span::styled(
                                    part,
                                    Style::default().add_modifier(Modifier::BOLD),
                                ));
                            } else {
                                spans.push(Span::raw(part));
                            }
                        }

                        content_elements.push(Line::from(spans));
                    }
                }
                ShellscapeNotifications::Warning {
                    start_time,
                    message,
                } => {
                    let time = Self::format_time(start_time);
                    let time_element = Line::from(vec![
                        Span::raw("\u{1F6A8}"),
                        Span::styled(
                            format!(" {}", time),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::LightYellow),
                        ),
                        Span::styled(" \u{2022} ", Style::default()),
                        Span::styled(
                            " WARNING ",
                            Style::default()
                                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                                .bg(Color::Yellow),
                        ),
                    ]);

                    content_elements.push(time_element);

                    let message = format!("\u{25C6} {}", message);
                    let message_lines = textwrap::wrap(&message, &textwrap_width);

                    for borrowed_line in message_lines {
                        let mut spans = vec![Span::raw("    ")];
                        let line = borrowed_line.to_string();

                        let parts: Vec<String> = line.split("**").map(|v| v.to_string()).collect();

                        for (idx, part) in parts.into_iter().enumerate() {
                            if idx % 2 == 1 {
                                spans.push(Span::styled(
                                    part,
                                    Style::default().add_modifier(Modifier::BOLD),
                                ));
                            } else {
                                spans.push(Span::raw(part));
                            }
                        }

                        content_elements.push(Line::from(spans));
                    }
                }
            }

            content_elements.push(Line::from(Span::styled("", Style::default())));
        }

        shellscape_app.reset_dock_scroll_state(content_elements.len());
        shellscape_app.reset_notifications_scroll_state(content_elements.len());

        let settings = Paragraph::new("Settings")
            .bg(secondary_color)
            .scroll(shellscape_app.get_dock_offset())
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 1, 1))
                    .fg(off_white),
            );

        let content = Paragraph::new(content_elements)
            .bg(foreground_color)
            .scroll(shellscape_app.get_notifications_offset())
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 1, 1))
                    .fg(off_white),
            );

        frame.render_widget(settings, content_container[0]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("\u{25B4}"))
                .end_symbol(Some("\u{25BE}"))
                .track_symbol(Some("\u{2503}"))
                .end_style(primary_color)
                .begin_style(primary_color)
                .track_style(primary_color)
                .thumb_style(deep_teal),
            content_container[0],
            &mut shellscape_app.dock_vertical_scroll_state,
        );

        frame.render_widget(content, content_container[1]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("\u{25B4}"))
                .end_symbol(Some("\u{25BE}"))
                .track_symbol(Some("\u{2503}"))
                .end_style(primary_color)
                .begin_style(primary_color)
                .track_style(primary_color)
                .thumb_style(deep_teal),
            content_container[1],
            &mut shellscape_app.notification_vertical_scroll_state,
        );

        let footer = shellscape_app.get_footer();
        let footer_element = Span::styled(&footer, Style::default().fg(light_cream));

        let footer_container = Paragraph::new(footer_element)
            .alignment(Alignment::Center)
            .bg(primary_color)
            .block(
                Block::default()
                    .padding(Padding::vertical(1))
                    .fg(light_cream),
            );

        //frame.render_widget(header_container, main_layout[0]);
        frame.render_widget(footer_container, main_layout[2]);
    }

    fn format_time(time: &DateTime<Local>) -> String {
        time.format("%H:%M:%S.%3f").to_string()
    }
}
