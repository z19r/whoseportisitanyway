use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table as RatatuiTable};

use super::style;
use super::App;

pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).split(frame.area());

    render_table(app, frame, chunks[0]);
    render_status_bar(app, frame, chunks[1]);
}

fn render_table(app: &App, frame: &mut Frame, area: Rect) {
    let wild = app.konami_mode;

    let grouping_active = app.group_field != super::GroupField::None;

    let mut rows: Vec<Row> = Vec::new();
    let mut visual_selected: Option<usize> = None;
    let mut visual_row_idx: usize = 0;

    for (i, e) in app.entries.iter().enumerate() {
        // Insert a group header row when the group label changes.
        if grouping_active {
            let current_label = app.group_labels.get(i).map(|s| s.as_str()).unwrap_or("");
            let prev_label = if i > 0 {
                app.group_labels
                    .get(i - 1)
                    .map(|s| s.as_str())
                    .unwrap_or("")
            } else {
                ""
            };
            if i == 0 || current_label != prev_label {
                let header_style = if wild {
                    Style::default()
                        .fg(Color::White)
                        .bg(style::wild_header_bg())
                        .bold()
                        .italic()
                } else {
                    Style::default()
                        .fg(Color::Rgb(200, 180, 255))
                        .bold()
                        .italic()
                };
                let header_row = Row::new(vec![
                    Cell::from(format!("  \u{25B8} {current_label}")).style(header_style),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ]);
                rows.push(header_row);
                visual_row_idx += 1;
            }
        }

        if i == app.selected {
            visual_selected = Some(visual_row_idx);
        }

        let type_color = if wild {
            style::classification_color_wild(&e.classification, i)
        } else {
            style::classification_color(&e.classification)
        };
        let state_color = if wild {
            style::state_color_wild(&e.state, i)
        } else {
            style::state_color(&e.state)
        };
        let own_style = if wild {
            style::ownership_style_wild(&e.ownership, i)
        } else {
            style::ownership_style(&e.ownership)
        };
        let port_color = if wild {
            style::port_color_wild(e.port, i)
        } else {
            style::port_color(e.port)
        };
        let dim = if wild { style::wild_dim(i) } else { style::DIM };
        let name_color = if wild {
            style::wild_dim(i + 50)
        } else {
            Color::White
        };
        let proj_color = if wild {
            style::wild_dim(i + 100)
        } else {
            Color::Rgb(180, 160, 220)
        };

        // When pid=0 (blocked process), show user info as fallback
        let display_name = if e.process_name.is_empty() {
            if let Some(ref user) = e.user {
                format!("[{}]", user)
            } else {
                "—".to_string()
            }
        } else {
            e.process_name.clone()
        };

        let row = Row::new(vec![
            Cell::from(format!(" {:>7}", e.pid)).style(Style::default().fg(dim)),
            Cell::from(format!("{:>5}", e.port)).style(Style::default().fg(port_color).bold()),
            Cell::from(format!(" {:<5}", e.protocol)).style(Style::default().fg(dim)),
            Cell::from(format!(" {}", display_name)).style(Style::default().fg(name_color)),
            Cell::from(format!(" {}", e.classification)).style(Style::default().fg(type_color)),
            Cell::from(format!(
                " {}",
                e.project.as_ref().map(|p| p.name.as_str()).unwrap_or("—")
            ))
            .style(Style::default().fg(proj_color)),
            Cell::from(format!(" {}", e.ownership)).style(own_style),
            Cell::from(format!(" {}", e.state)).style(Style::default().fg(state_color)),
        ]);
        rows.push(row);
        visual_row_idx += 1;
    }

    let header_style = if wild {
        Style::default()
            .fg(Color::White)
            .bg(style::wild_header_bg())
            .bold()
    } else {
        Style::default()
            .fg(style::HEADER_FG)
            .bg(style::HEADER_BG)
            .bold()
    };

    let sort_col: usize = match app.sort_field {
        super::SortField::Pid => 0,
        super::SortField::Port => 1,
        super::SortField::Process => 3,
        super::SortField::Type => 4,
        super::SortField::State => 7,
    };

    let header_labels = [
        "   PID", "  PORT", " PROTO", " PROCESS", " TYPE", " PROJECT", " STATUS", " STATE",
    ];

    let header_cells: Vec<Cell> = header_labels
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            let cell = Cell::from(label);
            if i == sort_col {
                cell.style(Style::default().add_modifier(Modifier::REVERSED))
            } else {
                cell
            }
        })
        .collect();

    let header = Row::new(header_cells).style(header_style).height(1);

    let widths = [
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(16),
        Constraint::Length(6),
        Constraint::Length(13),
    ];

    let title_spans = if wild {
        style::wild_title()
    } else {
        style::plain_title()
    };
    let mut title_line: Vec<Span> = vec![Span::raw(" ")];
    title_line.extend(title_spans);
    title_line.push(Span::styled(
        format!(" [{} ports] ", app.entries.len()),
        Style::default().fg(if wild { style::wild_dim(0) } else { style::DIM }),
    ));

    let border_color = if wild {
        style::wild_border()
    } else {
        style::BORDER_COLOR
    };
    let bg = if wild {
        style::wild_bg()
    } else {
        Color::Rgb(10, 5, 25)
    };
    let highlight = if wild {
        Style::default()
            .bg(style::wild_selected_bg())
            .fg(Color::White)
            .bold()
    } else {
        Style::default()
            .bg(style::SELECTED_BG)
            .fg(style::SELECTED_FG)
            .bold()
    };

    let table = RatatuiTable::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(Line::from(title_line))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(bg)),
        )
        .row_highlight_style(highlight);

    // When grouping, use the visual row index (which accounts for header rows);
    // otherwise fall back to the logical entry index.
    let highlight_idx = if grouping_active {
        visual_selected
    } else {
        if app.entries.is_empty() {
            None
        } else {
            Some(app.selected)
        }
    };

    frame.render_stateful_widget(
        table,
        area,
        &mut ratatui::widgets::TableState::default().with_selected(highlight_idx),
    );
}

fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let key = |k: &str| {
        Span::styled(
            format!(" {k}"),
            Style::default().fg(style::STATUS_KEY).bold(),
        )
    };
    let label = |l: &str| Span::styled(l.to_string(), Style::default().fg(style::STATUS_FG));

    let filter_color = if app.filter != super::Filter::All {
        Color::Rgb(255, 180, 50)
    } else {
        style::STATUS_FG
    };

    let group_color = if app.group_field != super::GroupField::None {
        Color::Rgb(180, 255, 180)
    } else {
        style::STATUS_FG
    };

    let hide_sys_color = if app.hide_system {
        Color::Rgb(255, 180, 50)
    } else {
        style::STATUS_FG
    };
    let hide_sys_label = if app.hide_system {
        " hide sys "
    } else {
        " show sys "
    };

    let spans = vec![
        key("j/k"),
        label(" nav  "),
        key("enter"),
        label(" detail  "),
        key("x"),
        label(" kill  "),
        key("s"),
        label(&format!(" sort:{} ", app.sort_field.label())),
        key("f"),
        Span::styled(
            format!(" filter:{} ", app.filter.label()),
            Style::default().fg(filter_color),
        ),
        key("Tab"),
        Span::styled(
            format!(" group:{} ", app.group_field.label()),
            Style::default().fg(group_color),
        ),
        key("h"),
        Span::styled(hide_sys_label, Style::default().fg(hide_sys_color)),
        key("r"),
        label(" refresh  "),
        key("q"),
        label(" quit"),
    ];

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(style::STATUS_BG));

    frame.render_widget(bar, area);
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
            view: super::super::View::Table,
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
        let app = test_app(0);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_entries_no_panic() {
        let app = test_app(5);
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_wild_mode_no_panic() {
        let mut app = test_app(3);
        app.konami_mode = true;
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_pid_zero_no_panic() {
        let mut app = test_app(1);
        app.all_entries[0].pid = 0;
        app.entries[0].pid = 0;
        let backend = TestBackend::new(100, 30);
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
        app.all_entries[0].project = app.entries[0].project.clone();
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_all_sort_columns() {
        for sf in [
            super::super::SortField::Port,
            super::super::SortField::Process,
            super::super::SortField::Type,
            super::super::SortField::Pid,
            super::super::SortField::State,
        ] {
            let mut app = test_app(2);
            app.sort_field = sf;
            let backend = TestBackend::new(100, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| render(&app, frame)).unwrap();
        }
    }

    #[test]
    fn render_active_filter_style() {
        let mut app = test_app(3);
        app.filter = super::super::Filter::Listen;
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_all_classifications() {
        let classes = [
            Classification::DevServer,
            Classification::Database,
            Classification::Docker,
            Classification::Proxy,
            Classification::Browser,
            Classification::SshTunnel,
            Classification::System,
            Classification::Unknown,
            Classification::BuildTool,
            Classification::LanguageServer,
            Classification::MessageQueue,
        ];
        for cls in classes {
            let mut app = test_app(1);
            app.entries[0].classification = cls.clone();
            app.all_entries[0].classification = cls;
            let backend = TestBackend::new(100, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| render(&app, frame)).unwrap();
        }
    }

    #[test]
    fn render_all_ownership_types() {
        for own in [Ownership::Owned, Ownership::Blocked, Ownership::Untracked] {
            let mut app = test_app(1);
            app.entries[0].ownership = own.clone();
            app.all_entries[0].ownership = own;
            let backend = TestBackend::new(100, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| render(&app, frame)).unwrap();
        }
    }

    #[test]
    fn render_user_fallback_when_process_name_empty() {
        let mut app = test_app(1);
        app.entries[0].process_name = String::new();
        app.entries[0].user = Some("alice".to_string());
        app.all_entries[0].process_name = String::new();
        app.all_entries[0].user = Some("alice".to_string());
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_both_states() {
        for st in [PortState::Listen, PortState::Established] {
            let mut app = test_app(1);
            app.entries[0].state = st.clone();
            app.all_entries[0].state = st;
            let backend = TestBackend::new(100, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| render(&app, frame)).unwrap();
        }
    }

    #[test]
    fn render_with_group_field_no_panic() {
        let mut app = test_app(3);
        app.group_field = super::super::GroupField::Type;
        app.group_labels = app
            .entries
            .iter()
            .map(|e| e.classification.to_string())
            .collect();
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_group_field_state_no_panic() {
        let mut app = test_app(3);
        app.group_field = super::super::GroupField::State;
        app.group_labels = app.entries.iter().map(|e| e.state.to_string()).collect();
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }

    #[test]
    fn render_with_group_field_wild_no_panic() {
        let mut app = test_app(3);
        app.konami_mode = true;
        app.group_field = super::super::GroupField::Process;
        app.group_labels = app.entries.iter().map(|e| e.process_name.clone()).collect();
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
    }
}
