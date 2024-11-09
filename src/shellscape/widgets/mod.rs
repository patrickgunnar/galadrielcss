use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::ShellscapeApp;

#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeWidgets {}

impl ShellscapeWidgets {
    pub fn paint(frame: &mut Frame, shellscape_app: &mut ShellscapeApp) {
        // Layout for the main sections
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(frame.area());

        // Render Title
        let title = Paragraph::new(Span::styled(
            &shellscape_app.metadata.title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Title"));
        frame.render_widget(title, chunks[0]);

        // Render Subtitle
        let subtitle = Paragraph::new(Span::styled(
            &shellscape_app.metadata.subtitle,
            Style::default().fg(Color::Cyan),
        ))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Subtitle"));
        frame.render_widget(subtitle, chunks[1]);

        // Render Galadriel Configurations
        let galadriel_config_text = vec![
            Line::from(vec![Span::styled(
                "Galadriel CSS Configuration:",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )]),
            Line::from(format!(
                "Exclude Paths: {:?}",
                shellscape_app.configs.get_exclude()
            )),
            Line::from(format!(
                "Auto Naming: {}",
                shellscape_app.configs.get_auto_naming()
            )),
            Line::from(format!(
                "Reset Styles: {}",
                shellscape_app.configs.get_reset_styles()
            )),
            Line::from(format!(
                "Minified Styles: {}",
                shellscape_app.configs.get_minified_styles()
            )),
            Line::from(format!("Port: {}", shellscape_app.configs.get_port())),
            Line::from(format!("Version: {}", shellscape_app.configs.get_version())),
        ];

        let configs = Paragraph::new(galadriel_config_text)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Galadriel Configs"),
            );
        frame.render_widget(configs, chunks[3]);

        // Render Footer Information
        let footer = Paragraph::new(Span::styled(
            format!(
                "Author: {} | License: {}",
                shellscape_app.metadata.author, shellscape_app.metadata.license
            ),
            Style::default().fg(Color::Gray),
        ))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Footer"));
        frame.render_widget(footer, chunks[4]);
    }
}
