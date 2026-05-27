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

pub fn plain_title() -> Vec<Span<'static>> {
    vec![Span::styled(
        "whoseportisitanyway",
        Style::default().fg(HEADER_FG).bold(),
    )]
}

pub fn rainbow_title() -> Vec<Span<'static>> {
    let title = "whoseportisitanyway";
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
    let title = "whoseportisitanyway";
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_to_rgb_red() {
        let Color::Rgb(r, g, b) = hue_to_rgb(0.0) else {
            panic!()
        };
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn hue_to_rgb_green() {
        let Color::Rgb(r, g, b) = hue_to_rgb(120.0) else {
            panic!()
        };
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn hue_to_rgb_blue() {
        let Color::Rgb(r, g, b) = hue_to_rgb(240.0) else {
            panic!()
        };
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 255);
    }

    #[test]
    fn hue_to_rgb_wraps_negative() {
        let a = hue_to_rgb(-60.0);
        let b = hue_to_rgb(300.0);
        assert_eq!(a, b);
    }

    #[test]
    fn hue_to_rgb_wraps_over_360() {
        let a = hue_to_rgb(420.0);
        let b = hue_to_rgb(60.0);
        assert_eq!(a, b);
    }

    #[test]
    fn classification_color_all_variants() {
        let variants = [
            Classification::DevServer,
            Classification::Database,
            Classification::Docker,
            Classification::BuildTool,
            Classification::LanguageServer,
            Classification::Proxy,
            Classification::Browser,
            Classification::MessageQueue,
            Classification::SshTunnel,
            Classification::System,
            Classification::Unknown,
        ];
        for v in &variants {
            let c = classification_color(v);
            assert!(matches!(c, Color::Rgb(_, _, _)));
        }
    }

    #[test]
    fn classification_color_devserver_is_green() {
        let Color::Rgb(r, g, b) = classification_color(&Classification::DevServer) else {
            panic!()
        };
        assert!(g > r && g > b);
    }

    #[test]
    fn classification_color_wild_returns_color() {
        let c = classification_color_wild(&Classification::Docker, 3);
        assert!(matches!(c, Color::Rgb(_, _, _)));
    }

    #[test]
    fn ownership_style_owned_is_bold_green() {
        let s = ownership_style(&Ownership::Owned);
        assert!(s.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn ownership_style_blocked_is_bold() {
        let s = ownership_style(&Ownership::Blocked);
        assert!(s.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn ownership_style_untracked_not_bold() {
        let s = ownership_style(&Ownership::Untracked);
        assert!(!s.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn ownership_style_wild_no_panic() {
        let _ = ownership_style_wild(&Ownership::Owned, 0);
        let _ = ownership_style_wild(&Ownership::Blocked, 5);
        let _ = ownership_style_wild(&Ownership::Untracked, 10);
    }

    #[test]
    fn state_color_listen_is_yellow() {
        let Color::Rgb(r, g, _) = state_color(&PortState::Listen) else {
            panic!()
        };
        assert!(r > 200 && g > 200);
    }

    #[test]
    fn state_color_established_is_dim() {
        let Color::Rgb(r, _, _) = state_color(&PortState::Established) else {
            panic!()
        };
        assert!(r < 150);
    }

    #[test]
    fn state_color_wild_no_panic() {
        let _ = state_color_wild(&PortState::Listen, 0);
        let _ = state_color_wild(&PortState::Established, 5);
    }

    #[test]
    fn port_color_privileged() {
        let Color::Rgb(r, _, _) = port_color(80) else {
            panic!()
        };
        assert_eq!(r, 255);
    }

    #[test]
    fn port_color_common() {
        let Color::Rgb(_, g, _) = port_color(3000) else {
            panic!()
        };
        assert_eq!(g, 140);
    }

    #[test]
    fn port_color_high() {
        let Color::Rgb(_, g, _) = port_color(9000) else {
            panic!()
        };
        assert_eq!(g, 200);
    }

    #[test]
    fn port_color_ephemeral() {
        let Color::Rgb(r, g, _) = port_color(50000) else {
            panic!()
        };
        assert_eq!(r, 160);
        assert_eq!(g, 160);
    }

    #[test]
    fn port_color_boundary_1023() {
        assert_eq!(port_color(1023), Color::Rgb(255, 100, 100));
    }

    #[test]
    fn port_color_boundary_1024() {
        assert_eq!(port_color(1024), Color::Rgb(180, 140, 255));
    }

    #[test]
    fn port_color_boundary_49151() {
        assert_eq!(port_color(49151), Color::Rgb(100, 200, 255));
    }

    #[test]
    fn port_color_boundary_49152() {
        assert_eq!(port_color(49152), Color::Rgb(160, 160, 180));
    }

    #[test]
    fn port_color_wild_no_panic() {
        let _ = port_color_wild(3000, 0);
        let _ = port_color_wild(80, 10);
    }

    #[test]
    fn plain_title_single_span() {
        let spans = plain_title();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "whoseportisitanyway");
    }

    #[test]
    fn rainbow_title_has_19_spans() {
        let spans = rainbow_title();
        assert_eq!(spans.len(), 19);
    }

    #[test]
    fn rainbow_title_uses_gradient_colors() {
        let spans = rainbow_title();
        assert_eq!(spans[0].style.fg, Some(TITLE_GRADIENT[0]));
    }

    #[test]
    fn wild_title_has_19_spans() {
        let spans = wild_title();
        assert_eq!(spans.len(), 19);
    }

    #[test]
    fn wild_bg_returns_rgb() {
        assert!(matches!(wild_bg(), Color::Rgb(_, _, _)));
    }

    #[test]
    fn wild_border_returns_rgb() {
        assert!(matches!(wild_border(), Color::Rgb(_, _, _)));
    }

    #[test]
    fn wild_header_bg_returns_rgb() {
        assert!(matches!(wild_header_bg(), Color::Rgb(_, _, _)));
    }

    #[test]
    fn wild_dim_returns_rgb() {
        assert!(matches!(wild_dim(0), Color::Rgb(_, _, _)));
    }

    #[test]
    fn wild_selected_bg_returns_rgb() {
        assert!(matches!(wild_selected_bg(), Color::Rgb(_, _, _)));
    }

    #[test]
    fn title_gradient_has_7_colors() {
        assert_eq!(TITLE_GRADIENT.len(), 7);
    }

    #[test]
    fn constants_are_correct() {
        assert_eq!(HEADER_BG, Color::Rgb(30, 15, 60));
        assert_eq!(HEADER_FG, Color::Rgb(200, 160, 255));
        assert_eq!(BORDER_COLOR, Color::Rgb(100, 60, 180));
        assert_eq!(SELECTED_BG, Color::Rgb(60, 30, 120));
        assert_eq!(SELECTED_FG, Color::White);
        assert_eq!(DIM, Color::Rgb(100, 90, 120));
    }
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
