use std::borrow::Cow;

use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};
use textwrap::Options;

use super::{app::ShellscapeApp, notifications::ShellscapeNotifications};

#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeWidgets {}

/*
let primary_color = Color::Rgb(50, 70, 60);
let settings_color = Color::Rgb(0, 35, 35);
let content_color = Color::Rgb(5, 10, 10);
*/

impl ShellscapeWidgets {
    pub fn paint(frame: &mut Frame, shellscape_app: &mut ShellscapeApp) {
        let primary_color = Color::Rgb(120, 120, 120);
        let settings_color = Color::Rgb(55, 55, 55);
        let content_color = Color::Rgb(10, 10, 10);

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
                shellscape_app.metadata.author,
                shellscape_app.metadata.license,
                shellscape_app.metadata.version
            ),
            Style::default().add_modifier(Modifier::BOLD),
        ));

        let subtitle_lines = textwrap::wrap(
            &shellscape_app.metadata.subtitle,
            Options::new(author_element.width()),
        );

        header_elements.push(author_element);
        header_elements.push(Line::from(Span::styled("", Style::default())));

        for borrowed_line in subtitle_lines {
            if let Cow::Borrowed(line) = borrowed_line {
                header_elements.push(Line::from(Span::styled(line.to_string(), Style::default())));
            }
        }

        let title = format!(" {} ", shellscape_app.metadata.title.to_uppercase());
        let header = Paragraph::new(header_elements)
            .alignment(Alignment::Center)
            .bg(primary_color)
            .block(
                Block::default()
                    .padding(Padding::top(1))
                    .title(Span::styled(
                        title,
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .fg(Color::White)
                    .title_alignment(Alignment::Center),
            );

        frame.render_widget(header, main_layout[0]);

        let content_container = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(main_layout[1]);

        let mut content_elements = vec![];
        let textwrap_width = Options::new(content_container[1].width.saturating_sub(10) as usize);

        for notification in &shellscape_app.notifications {
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
                        Span::styled("        \u{1F6D1} ", Style::default()),
                        Span::styled("TYPE    ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" {:?}", error.get_type()), Style::default()),
                    ]));

                    content_elements.push(Line::from(vec![
                        Span::styled("        \u{1F9EF} ", Style::default()),
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
                ShellscapeNotifications::NenyrError {
                    start_time,
                    error: _,
                } => {
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

        let settings = Paragraph::new("Settings")
            .bg(settings_color)
            .block(Block::default().padding(Padding::new(1, 1, 1, 1)));
        let content = Paragraph::new(content_elements)
            .bg(content_color)
            .scroll(shellscape_app.notifications_offset)
            .block(
                Block::default()
                    .padding(Padding::new(1, 1, 1, 1))
                    .fg(Color::White),
            );

        frame.render_widget(settings, content_container[0]);
        frame.render_widget(content, content_container[1]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            content_container[1],
            &mut shellscape_app.notification_scroll_vertical,
        );

        let footer_element = Span::styled(
            &shellscape_app.metadata.footer,
            Style::default().fg(Color::White),
        );

        let footer_container = Paragraph::new(footer_element)
            .alignment(Alignment::Center)
            .bg(primary_color)
            .block(Block::default().padding(Padding::vertical(1)));

        //frame.render_widget(header_container, main_layout[0]);
        frame.render_widget(footer_container, main_layout[2]);
    }

    fn format_time(time: &DateTime<Local>) -> String {
        time.format("%H:%M:%S.%3f").to_string()
    }
}
