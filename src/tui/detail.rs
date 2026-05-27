use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::style;
use super::App;

pub fn render(app: &App, frame: &mut Frame) {
    let wild = app.konami_mode;
    let lines = match app.selected_entry() {
        Some(entry) => {
            let type_color = if wild {
                style::classification_color_wild(&entry.classification, 0)
            } else {
                style::classification_color(&entry.classification)
            };
            let own_style = if wild {
                style::ownership_style_wild(&entry.ownership, 0)
            } else {
                style::ownership_style(&entry.ownership)
            };
            let state_color = if wild {
                style::state_color_wild(&entry.state, 0)
            } else {
                style::state_color(&entry.state)
            };
            let dim = if wild { style::wild_dim(0) } else { style::DIM };
            let port_c = if wild {
                style::port_color_wild(entry.port, 0)
            } else {
                style::port_color(entry.port)
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Port: ", Style::default().fg(dim)),
                    Span::styled(entry.port.to_string(), Style::default().fg(port_c).bold()),
                    Span::styled(format!("  ({})", entry.protocol), Style::default().fg(dim)),
                ]),
                Line::from(vec![
                    Span::styled("PID: ", Style::default().fg(dim)),
                    Span::styled(
                        entry.pid.to_string(),
                        Style::default().fg(if wild {
                            style::wild_dim(1)
                        } else {
                            Color::White
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Process: ", Style::default().fg(dim)),
                    Span::styled(
                        entry.process_name.clone(),
                        Style::default()
                            .fg(if wild {
                                style::wild_dim(2)
                            } else {
                                Color::White
                            })
                            .bold(),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Command: ", Style::default().fg(dim)),
                    Span::styled(
                        entry.command_line.clone(),
                        Style::default().fg(if wild {
                            style::wild_dim(3)
                        } else {
                            Color::Rgb(180, 160, 220)
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Address: ", Style::default().fg(dim)),
                    Span::styled(
                        entry.local_addr.clone(),
                        Style::default().fg(if wild {
                            style::wild_dim(4)
                        } else {
                            Color::Rgb(100, 200, 255)
                        }),
                    ),
                ]),
            ];

            if entry.all_addrs.len() > 1 {
                lines.push(Line::from(vec![
                    Span::styled("All addrs: ", Style::default().fg(dim)),
                    Span::styled(
                        entry.all_addrs.join(", "),
                        Style::default().fg(if wild {
                            style::wild_dim(5)
                        } else {
                            Color::Rgb(100, 200, 255)
                        }),
                    ),
                ]));
            }

            lines.extend(vec![
                Line::from(vec![
                    Span::styled("State: ", Style::default().fg(dim)),
                    Span::styled(entry.state.to_string(), Style::default().fg(state_color)),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(dim)),
                    Span::styled(
                        entry.classification.to_string(),
                        Style::default().fg(type_color).bold(),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(dim)),
                    Span::styled(entry.ownership.to_string(), own_style),
                ]),
            ]);

            if let Some(ref project) = entry.project {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Project: ", Style::default().fg(dim)),
                    Span::styled(
                        project.name.clone(),
                        Style::default()
                            .fg(if wild {
                                style::wild_dim(6)
                            } else {
                                Color::Rgb(255, 180, 50)
                            })
                            .bold(),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Root: ", Style::default().fg(dim)),
                    Span::styled(
                        project.root.display().to_string(),
                        Style::default().fg(if wild {
                            style::wild_dim(7)
                        } else {
                            Color::Rgb(180, 160, 220)
                        }),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Framework: ", Style::default().fg(dim)),
                    Span::styled(
                        project
                            .framework
                            .as_ref()
                            .map(|f| f.to_string())
                            .unwrap_or_else(|| "—".to_string()),
                        Style::default().fg(if wild {
                            style::wild_dim(8)
                        } else {
                            Color::Rgb(50, 255, 120)
                        }),
                    ),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(" Esc", Style::default().fg(style::STATUS_KEY).bold()),
                Span::styled(" back  ", Style::default().fg(style::STATUS_FG)),
                Span::styled("x", Style::default().fg(style::STATUS_KEY).bold()),
                Span::styled(" kill", Style::default().fg(style::STATUS_FG)),
            ]));

            lines
        }
        None => vec![Line::from(Span::styled(
            "No port selected",
            Style::default().fg(style::DIM),
        ))],
    };

    let border_c = if wild {
        style::wild_border()
    } else {
        style::BORDER_HIGHLIGHT
    };
    let bg = if wild {
        style::wild_bg()
    } else {
        Color::Rgb(10, 5, 25)
    };
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(
                " Detail ",
                Style::default()
                    .fg(if wild {
                        style::wild_dim(99)
                    } else {
                        style::HEADER_FG
                    })
                    .bold(),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_c))
            .style(Style::default().bg(bg)),
    );

    let area = centered_rect(65, 70, frame.area());
    frame.render_widget(Clear, area);
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
            view: super::super::View::Detail,
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
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_wild_mode_no_panic() {
        let mut app = test_app(1);
        app.konami_mode = true;
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_project_and_framework() {
        let mut app = test_app(1);
        app.entries[0].project = Some(Project {
            name: "myapp".into(),
            root: "/tmp/myapp".into(),
            framework: Some(Framework::Next),
        });
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_project_no_framework() {
        let mut app = test_app(1);
        app.entries[0].project = Some(Project {
            name: "bare".into(),
            root: "/tmp/bare".into(),
            framework: None,
        });
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_multiple_addrs() {
        let mut app = test_app(1);
        app.entries[0].all_addrs =
            vec!["0.0.0.0:3000".into(), "127.0.0.1:3000".into(), "::1:3000".into()];
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_wild_with_project() {
        let mut app = test_app(1);
        app.konami_mode = true;
        app.entries[0].project = Some(Project {
            name: "wild".into(),
            root: "/tmp/wild".into(),
            framework: Some(Framework::Vite),
        });
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn centered_rect_produces_inner_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let inner = centered_rect(50, 50, area);
        assert!(inner.width > 0);
        assert!(inner.height > 0);
        assert!(inner.x > 0);
        assert!(inner.y > 0);
    }
}
