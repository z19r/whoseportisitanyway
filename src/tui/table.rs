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

    let rows: Vec<Row> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
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

            Row::new(vec![
                Cell::from(format!(" {:>7}", e.pid)).style(Style::default().fg(dim)),
                Cell::from(format!("{:>5}", e.port)).style(Style::default().fg(port_color).bold()),
                Cell::from(format!(" {:<5}", e.protocol)).style(Style::default().fg(dim)),
                Cell::from(format!(" {}", e.process_name)).style(Style::default().fg(name_color)),
                Cell::from(format!(" {}", e.classification)).style(Style::default().fg(type_color)),
                Cell::from(format!(
                    " {}",
                    e.project.as_ref().map(|p| p.name.as_str()).unwrap_or("—")
                ))
                .style(Style::default().fg(proj_color)),
                Cell::from(format!(" {}", e.ownership)).style(own_style),
                Cell::from(format!(" {}", e.state)).style(Style::default().fg(state_color)),
            ])
        })
        .collect();

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

    if app.all_entries.iter().any(|e| e.pid == 0) {
        title_line.push(Span::styled(
            "run with sudo for full info ",
            Style::default().fg(Color::Rgb(255, 100, 100)).italic(),
        ));
    }

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

    frame.render_stateful_widget(
        table,
        area,
        &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected)),
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
        key("r"),
        label(" refresh  "),
        key("q"),
        label(" quit"),
    ];

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(style::STATUS_BG));

    frame.render_widget(bar, area);
}
