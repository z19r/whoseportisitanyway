use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

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

    frame.render_widget(paragraph, frame.area());
}
