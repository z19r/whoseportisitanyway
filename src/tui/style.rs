use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::prelude::*;

use crate::model::{Classification, Ownership, PortState};

pub const HEADER_BG: Color = Color::Rgb(30, 15, 60);
pub const HEADER_FG: Color = Color::Rgb(200, 160, 255);

pub const BORDER_COLOR: Color = Color::Rgb(100, 60, 180);
pub const BORDER_HIGHLIGHT: Color = Color::Rgb(180, 120, 255);

pub const STATUS_BG: Color = Color::Rgb(20, 10, 45);
pub const STATUS_FG: Color = Color::Rgb(140, 120, 180);
pub const STATUS_KEY: Color = Color::Rgb(255, 100, 200);

pub const SELECTED_BG: Color = Color::Rgb(60, 30, 120);
pub const SELECTED_FG: Color = Color::White;

pub const DIM: Color = Color::Rgb(100, 90, 120);

pub const TITLE_GRADIENT: [Color; 7] = [
    Color::Rgb(255, 50, 100),
    Color::Rgb(255, 120, 50),
    Color::Rgb(255, 220, 50),
    Color::Rgb(50, 255, 120),
    Color::Rgb(50, 180, 255),
    Color::Rgb(120, 80, 255),
    Color::Rgb(255, 50, 200),
];

fn tick() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn hue_to_rgb(hue: f64) -> Color {
    let h = ((hue % 360.0) + 360.0) % 360.0;
    let s = 1.0_f64;
    let v = 1.0_f64;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u16 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn wild_color(offset: usize) -> Color {
    let base_hue = (tick() / 50) as f64;
    hue_to_rgb(base_hue + (offset as f64 * 37.0))
}

pub fn classification_color(class: &Classification) -> Color {
    match class {
        Classification::DevServer => Color::Rgb(80, 255, 120),
        Classification::Database => Color::Rgb(255, 140, 50),
        Classification::Docker => Color::Rgb(50, 180, 255),
        Classification::BuildTool => Color::Rgb(200, 200, 100),
        Classification::LanguageServer => Color::Rgb(160, 120, 255),
        Classification::Proxy => Color::Rgb(50, 220, 220),
        Classification::Browser => Color::Rgb(200, 180, 220),
        Classification::MessageQueue => Color::Rgb(255, 100, 200),
        Classification::SshTunnel => Color::Rgb(255, 180, 50),
        Classification::System => Color::Rgb(160, 140, 180),
        Classification::Unknown => Color::Rgb(120, 100, 140),
    }
}

pub fn classification_color_wild(class: &Classification, row: usize) -> Color {
    let base = match class {
        Classification::DevServer => 0,
        Classification::Database => 1,
        Classification::Docker => 2,
        Classification::BuildTool => 3,
        Classification::LanguageServer => 4,
        Classification::Proxy => 5,
        Classification::Browser => 6,
        Classification::MessageQueue => 7,
        Classification::SshTunnel => 8,
        Classification::System => 9,
        Classification::Unknown => 10,
    };
    wild_color(base * 30 + row * 7)
}

pub fn ownership_style(ownership: &Ownership) -> Style {
    match ownership {
        Ownership::Owned => Style::default().fg(Color::Rgb(80, 255, 120)).bold(),
        Ownership::Blocked => Style::default().fg(Color::Rgb(255, 70, 70)).bold(),
        Ownership::Untracked => Style::default().fg(DIM),
    }
}

pub fn ownership_style_wild(ownership: &Ownership, row: usize) -> Style {
    match ownership {
        Ownership::Owned => Style::default().fg(wild_color(row * 13)).bold(),
        Ownership::Blocked => Style::default().fg(wild_color(row * 13 + 180)).bold(),
        Ownership::Untracked => Style::default().fg(wild_color(row * 13 + 90)),
    }
}

pub fn state_color(state: &PortState) -> Color {
    match state {
        PortState::Listen => Color::Rgb(255, 220, 50),
        PortState::Established => Color::Rgb(100, 80, 140),
    }
}

pub fn state_color_wild(state: &PortState, row: usize) -> Color {
    match state {
        PortState::Listen => wild_color(row * 11),
        PortState::Established => wild_color(row * 11 + 180),
    }
}

pub fn rainbow_title() -> Vec<Span<'static>> {
    let title = "whosportisitanyway";
    title
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let color = TITLE_GRADIENT[i % TITLE_GRADIENT.len()];
            Span::styled(c.to_string(), Style::default().fg(color).bold())
        })
        .collect()
}

pub fn wild_title() -> Vec<Span<'static>> {
    let title = "whosportisitanyway";
    title
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let color = wild_color(i * 20);
            Span::styled(c.to_string(), Style::default().fg(color).bold())
        })
        .collect()
}

pub fn port_color(port: u16) -> Color {
    match port {
        0..=1023 => Color::Rgb(255, 100, 100),
        1024..=8999 => Color::Rgb(180, 140, 255),
        9000..=49151 => Color::Rgb(100, 200, 255),
        _ => Color::Rgb(160, 160, 180),
    }
}

pub fn port_color_wild(port: u16, row: usize) -> Color {
    let _ = port;
    wild_color(row * 9)
}

pub fn wild_bg() -> Color {
    let t = tick() / 200;
    let hue = (t % 360) as f64;
    let h = ((hue % 360.0) + 360.0) % 360.0;
    let s = 0.4_f64;
    let v = 0.08_f64;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u16 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub fn wild_border() -> Color {
    wild_color(0)
}

pub fn wild_header_bg() -> Color {
    let t = tick() / 100;
    let hue = (t % 360) as f64;
    let h = ((hue % 360.0) + 360.0) % 360.0;
    let s = 0.6_f64;
    let v = 0.25_f64;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u16 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub fn wild_dim(row: usize) -> Color {
    wild_color(row * 5 + 60)
}

pub fn wild_selected_bg() -> Color {
    let t = tick() / 80;
    let hue = (t % 360) as f64;
    let h = ((hue % 360.0) + 360.0) % 360.0;
    let s = 0.7_f64;
    let v = 0.35_f64;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u16 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
