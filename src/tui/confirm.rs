use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::style;
use super::App;

pub fn render(app: &App, frame: &mut Frame) {
    let lines = match app.selected_entry() {
        Some(entry) => {
            let mut lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Kill ", Style::default().fg(Color::White)),
                    Span::styled(
                        entry.process_name.clone(),
                        Style::default().fg(Color::Rgb(255, 100, 100)).bold(),
                    ),
                    Span::styled(
                        format!(" (PID {})?", entry.pid),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Port ", Style::default().fg(style::DIM)),
                    Span::styled(
                        entry.port.to_string(),
                        Style::default().fg(style::port_color(entry.port)).bold(),
                    ),
                    Span::styled(
                        format!(" on {}", entry.local_addr),
                        Style::default().fg(style::DIM),
                    ),
                ]),
            ];

            if let Some(ref project) = entry.project {
                lines.push(Line::from(vec![
                    Span::styled(
                        "  This will stop ",
                        Style::default().fg(Color::Rgb(255, 180, 50)),
                    ),
                    Span::styled(
                        project.name.clone(),
                        Style::default().fg(Color::Rgb(255, 180, 50)).bold(),
                    ),
                    Span::styled(".", Style::default().fg(Color::Rgb(255, 180, 50))),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("     ", Style::default()),
                Span::styled(
                    " y ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(200, 40, 40))
                        .bold(),
                ),
                Span::styled(" Yes, kill it  ", Style::default().fg(style::DIM)),
                Span::styled(
                    " n ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(60, 60, 80))
                        .bold(),
                ),
                Span::styled(" Cancel", Style::default().fg(style::DIM)),
            ]));

            lines
        }
        None => vec![Line::from(Span::styled(
            "No port selected",
            Style::default().fg(style::DIM),
        ))],
    };

    let area = centered_rect(50, 35, frame.area());

    frame.render_widget(Clear, area);

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(
                " Confirm Kill ",
                Style::default().fg(Color::Rgb(255, 70, 70)).bold(),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(200, 40, 40)))
            .style(Style::default().bg(Color::Rgb(30, 5, 10))),
    );

    frame.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
