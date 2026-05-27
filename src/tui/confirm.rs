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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Classification, Framework, Ownership, PortEntry, PortState, Project, Protocol,
    };
    use ratatui::backend::TestBackend;

    fn test_app(n: usize) -> super::super::App {
        let entries: Vec<PortEntry> = (0..n)
            .map(|i| PortEntry {
                port: 3000 + i as u16,
                protocol: Protocol::Tcp,
                pid: 100 + i as u32,
                process_name: format!("proc{i}"),
                command_line: format!("proc{i} --serve"),
                classification: Classification::DevServer,
                ownership: Ownership::Untracked,
                state: PortState::Listen,
                local_addr: format!("0.0.0.0:{}", 3000 + i),
                all_addrs: vec![format!("0.0.0.0:{}", 3000 + i)],
                project: None,
            })
            .collect();
        super::super::App {
            all_entries: entries.clone(),
            entries,
            selected: 0,
            view: super::super::View::Confirm,
            should_quit: false,
            watched_ports: vec![],
            sort_field: super::super::SortField::Port,
            filter: super::super::Filter::All,
            konami: super::super::KonamiDetector::new(),
            konami_mode: false,
            shuffle_remaining: 0,
        }
    }

    #[test]
    fn render_empty_no_panic() {
        let mut app = test_app(0);
        app.selected = 0;
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_entry_no_panic() {
        let app = test_app(1);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_project_no_panic() {
        let mut app = test_app(1);
        app.entries[0].project = Some(Project {
            name: "myapp".into(),
            root: "/tmp/myapp".into(),
            framework: Some(Framework::Vite),
        });
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn centered_rect_produces_inner_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let inner = centered_rect(50, 35, area);
        assert!(inner.width > 0);
        assert!(inner.height > 0);
        assert!(inner.x > 0);
        assert!(inner.y > 0);
    }
}
