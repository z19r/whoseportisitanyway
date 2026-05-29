use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::style;
use super::App;
use crate::model::PortEntry;

pub(crate) fn build_search_url(entry: &PortEntry) -> String {
    let query = format!(
        "what is port {} {} {}",
        entry.port, entry.process_name, entry.classification
    );
    // Encode the query: replace spaces with +, percent-encode non-URL-safe chars
    let encoded: String = query
        .chars()
        .map(|c| match c {
            ' ' => '+'.to_string(),
            c if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect();
    format!("https://www.google.com/search?q={encoded}")
}

pub(crate) fn open_in_browser(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
}

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

            // Show user info if available (works even without root/pid)
            if let Some(ref user) = entry.user {
                lines.push(Line::from(vec![
                    Span::styled("User: ", Style::default().fg(dim)),
                    Span::styled(
                        user.clone(),
                        Style::default().fg(if wild {
                            style::wild_dim(5)
                        } else {
                            Color::Rgb(100, 220, 180)
                        }),
                    ),
                ]));
            } else if let Some(uid) = entry.uid {
                lines.push(Line::from(vec![
                    Span::styled("UID: ", Style::default().fg(dim)),
                    Span::styled(
                        uid.to_string(),
                        Style::default().fg(if wild {
                            style::wild_dim(5)
                        } else {
                            Color::Rgb(100, 220, 180)
                        }),
                    ),
                ]));
            }

            // Show remote address for established connections
            if let Some(ref remote) = entry.remote_addr {
                lines.push(Line::from(vec![
                    Span::styled("Remote: ", Style::default().fg(dim)),
                    Span::styled(
                        remote.clone(),
                        Style::default().fg(if wild {
                            style::wild_dim(6)
                        } else {
                            Color::Rgb(255, 160, 80)
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

            let search_url = build_search_url(entry);
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Search: ", Style::default().fg(dim)),
                Span::styled(
                    search_url,
                    Style::default().fg(Color::Rgb(100, 150, 255)),
                ),
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(" Esc", Style::default().fg(style::STATUS_KEY).bold()),
                Span::styled(" back  ", Style::default().fg(style::STATUS_FG)),
                Span::styled("x", Style::default().fg(style::STATUS_KEY).bold()),
                Span::styled(" kill  ", Style::default().fg(style::STATUS_FG)),
                Span::styled("o", Style::default().fg(style::STATUS_KEY).bold()),
                Span::styled(" search", Style::default().fg(style::STATUS_FG)),
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
                uid: None,
                user: None,
                remote_addr: None,
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
            group_field: super::super::GroupField::None,
            group_labels: vec![],
            konami: super::super::KonamiDetector::new(),
            konami_mode: false,
            shuffle_remaining: 0,
            hide_system: false,
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
        app.entries[0].all_addrs = vec![
            "0.0.0.0:3000".into(),
            "127.0.0.1:3000".into(),
            "::1:3000".into(),
        ];
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_user_info_no_panic() {
        let mut app = test_app(1);
        app.entries[0].uid = Some(1000);
        app.entries[0].user = Some("alice".into());
        let backend = TestBackend::new(80, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_uid_only_no_panic() {
        let mut app = test_app(1);
        app.entries[0].uid = Some(0);
        app.entries[0].user = None;
        let backend = TestBackend::new(80, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_remote_addr_no_panic() {
        let mut app = test_app(1);
        app.entries[0].remote_addr = Some("192.168.1.100:54321".into());
        let backend = TestBackend::new(80, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_all_new_fields_no_panic() {
        let mut app = test_app(1);
        app.entries[0].uid = Some(1000);
        app.entries[0].user = Some("developer".into());
        app.entries[0].remote_addr = Some("10.0.0.1:443".into());
        let backend = TestBackend::new(80, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_all_new_fields_wild_no_panic() {
        let mut app = test_app(1);
        app.konami_mode = true;
        app.entries[0].uid = Some(1000);
        app.entries[0].user = Some("developer".into());
        app.entries[0].remote_addr = Some("10.0.0.1:443".into());
        let backend = TestBackend::new(80, 40);
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

    fn make_entry(port: u16, process_name: &str, classification: Classification) -> PortEntry {
        PortEntry {
            port,
            protocol: Protocol::Tcp,
            pid: 1234,
            process_name: process_name.to_string(),
            command_line: format!("{process_name} --serve"),
            classification,
            ownership: Ownership::Untracked,
            state: PortState::Listen,
            local_addr: format!("0.0.0.0:{port}"),
            all_addrs: vec![format!("0.0.0.0:{port}")],
            project: None,
            uid: None,
            user: None,
            remote_addr: None,
        }
    }

    #[test]
    fn build_search_url_produces_correct_url() {
        let entry = make_entry(3000, "node", Classification::DevServer);
        let url = build_search_url(&entry);
        assert!(url.starts_with("https://www.google.com/search?q="));
        assert!(url.contains("3000"));
        assert!(url.contains("node"));
        // spaces replaced with +
        assert!(!url.contains(' '));
    }

    #[test]
    fn build_search_url_contains_all_parts() {
        let entry = make_entry(5432, "postgres", Classification::Database);
        let url = build_search_url(&entry);
        assert!(url.contains("5432"));
        assert!(url.contains("postgres"));
        // classification display is "Database"
        assert!(url.contains("Database"));
    }

    #[test]
    fn build_search_url_spaces_encoded_as_plus() {
        // Classification::DevServer displays as "Dev Server" which has a space
        let entry = make_entry(3001, "vite", Classification::DevServer);
        let url = build_search_url(&entry);
        assert!(!url.contains(' '));
        assert!(url.contains('+'));
    }

    #[test]
    fn build_search_url_handles_special_chars_in_process_name() {
        // Process names with special chars should be percent-encoded in the query part
        let entry = make_entry(8080, "my/proc&name", Classification::Unknown);
        let url = build_search_url(&entry);
        // Extract just the query param value (after ?q=)
        let query_part = url.split("?q=").nth(1).expect("should have ?q=");
        assert!(!query_part.contains('/'), "slash should be encoded in query");
        assert!(!query_part.contains('&'), "ampersand should be encoded in query");
        assert!(query_part.contains("%2F"), "slash should be %2F");
        assert!(query_part.contains("%26"), "ampersand should be %26");
    }

    #[test]
    fn build_search_url_format_is_valid_https() {
        let entry = make_entry(80, "nginx", Classification::Proxy);
        let url = build_search_url(&entry);
        assert!(url.starts_with("https://www.google.com/search?q="));
        // No bare spaces — URL is safe to pass to a browser
        assert!(!url.contains(' '));
    }

    #[test]
    fn open_in_browser_is_callable() {
        // We can't verify a browser opens, but the function must not panic or crash
        // when called with a valid URL string.
        let url = "https://www.google.com/search?q=test";
        // Just verify it compiles and runs without panic.
        // On Linux it will attempt xdg-open; on macOS `open`. The call is fire-and-forget.
        open_in_browser(url);
    }

    #[test]
    fn render_with_search_url_no_panic() {
        let app = test_app(1);
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }
}
