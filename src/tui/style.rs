use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::prelude::*;

use crate::model::{Classification, Ownership, PortState};

// All accent colors below sit in a luminance band that clears >=3:1 contrast
// against BOTH a pure-white and a pure-black terminal background, so the UI
// stays readable whatever theme the terminal uses. Neutral text and base
// surfaces use Color::Reset (see the render code) to inherit the terminal's
// own foreground/background. See `theme_accents_readable_on_light_and_dark`.
pub const HEADER_FG: Color = Color::Rgb(150, 100, 225);

pub const BORDER_COLOR: Color = Color::Rgb(120, 80, 190);
pub const BORDER_HIGHLIGHT: Color = Color::Rgb(150, 100, 225);

pub const STATUS_FG: Color = Color::Rgb(150, 130, 190);
pub const STATUS_KEY: Color = Color::Rgb(210, 80, 175);

// Selected row keeps an explicit highlight bar. The foreground is an explicit
// bright truecolor (NOT Color::White, which maps to ANSI palette color 15 and
// renders as a muted off-white under many themes) so the row reads as clearly
// light on the bar regardless of the terminal palette (~7.9:1 contrast).
pub const SELECTED_BG: Color = Color::Rgb(95, 62, 160);
pub const SELECTED_FG: Color = Color::Rgb(245, 245, 250);

pub const DIM: Color = Color::Rgb(125, 118, 140);

// Secondary text (project names, wrapped command lines): a muted purple that
// still reads on light and dark backgrounds.
pub const SECONDARY_FG: Color = Color::Rgb(150, 130, 190);

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
        Classification::DevServer => Color::Rgb(40, 160, 70),
        Classification::Database => Color::Rgb(200, 110, 40),
        Classification::Docker => Color::Rgb(60, 130, 220),
        Classification::BuildTool => Color::Rgb(165, 135, 20),
        Classification::LanguageServer => Color::Rgb(150, 100, 225),
        Classification::Proxy => Color::Rgb(20, 160, 150),
        Classification::Browser => Color::Rgb(150, 130, 180),
        Classification::MessageQueue => Color::Rgb(210, 80, 175),
        Classification::SshTunnel => Color::Rgb(200, 130, 30),
        Classification::System => Color::Rgb(140, 128, 155),
        Classification::Unknown => Color::Rgb(120, 105, 140),
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
        Ownership::Owned => Style::default().fg(Color::Rgb(40, 160, 70)).bold(),
        Ownership::Blocked => Style::default().fg(Color::Rgb(215, 70, 70)).bold(),
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
        PortState::Listen => Color::Rgb(175, 135, 20),
        PortState::Established => Color::Rgb(125, 115, 150),
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
        0..=1023 => Color::Rgb(215, 70, 70),
        1024..=8999 => Color::Rgb(150, 100, 225),
        9000..=49151 => Color::Rgb(60, 130, 220),
        _ => Color::Rgb(140, 140, 155),
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
    fn state_color_listen_is_warm() {
        // Warm amber, ordered r > g > b (yellow would wash out on a light term).
        let Color::Rgb(r, g, b) = state_color(&PortState::Listen) else {
            panic!()
        };
        assert!(r > g && g > b);
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
    fn port_color_privileged_is_red() {
        let Color::Rgb(r, g, b) = port_color(80) else {
            panic!()
        };
        assert!(r > g && r > b);
    }

    #[test]
    fn port_color_common_is_purple() {
        let Color::Rgb(r, g, b) = port_color(3000) else {
            panic!()
        };
        assert!(b > r && b > g);
    }

    #[test]
    fn port_color_high_is_blue() {
        let Color::Rgb(r, g, b) = port_color(9000) else {
            panic!()
        };
        assert!(b > r && b > g);
    }

    #[test]
    fn port_color_ephemeral_is_gray() {
        let Color::Rgb(r, g, _) = port_color(50000) else {
            panic!()
        };
        assert_eq!(r, g);
    }

    #[test]
    fn port_color_boundary_1023() {
        assert_eq!(port_color(1023), Color::Rgb(215, 70, 70));
    }

    #[test]
    fn port_color_boundary_1024() {
        assert_eq!(port_color(1024), Color::Rgb(150, 100, 225));
    }

    #[test]
    fn port_color_boundary_49151() {
        assert_eq!(port_color(49151), Color::Rgb(60, 130, 220));
    }

    #[test]
    fn port_color_boundary_49152() {
        assert_eq!(port_color(49152), Color::Rgb(140, 140, 155));
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
        assert_eq!(HEADER_FG, Color::Rgb(150, 100, 225));
        assert_eq!(BORDER_COLOR, Color::Rgb(120, 80, 190));
        assert_eq!(SELECTED_BG, Color::Rgb(95, 62, 160));
        assert_eq!(SELECTED_FG, Color::Rgb(245, 245, 250));
        assert_eq!(DIM, Color::Rgb(125, 118, 140));
    }

    // --- Theme readability guard -------------------------------------------
    // The app paints no base background of its own (Color::Reset), so every
    // accent color must read against whatever the terminal uses. WCAG relative
    // luminance in [0.10, 0.30] guarantees >=3:1 contrast against BOTH pure
    // black and pure white. This test fails if a future color drifts out of
    // that band and would become invisible on a light or dark terminal.

    fn channel(c: u8) -> f64 {
        let c = c as f64 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    fn luminance(color: Color) -> f64 {
        let Color::Rgb(r, g, b) = color else {
            panic!("expected an Rgb color, got {color:?}")
        };
        0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
    }

    fn assert_readable_on_both_themes(name: &str, color: Color) {
        let l = luminance(color);
        assert!(
            (0.10..=0.30).contains(&l),
            "{name} luminance {l:.3} is outside the theme-safe band [0.10, 0.30]; \
             it would be low-contrast on a light or dark terminal",
        );
    }

    #[test]
    fn theme_accents_readable_on_light_and_dark() {
        assert_readable_on_both_themes("HEADER_FG", HEADER_FG);
        assert_readable_on_both_themes("BORDER_COLOR", BORDER_COLOR);
        assert_readable_on_both_themes("BORDER_HIGHLIGHT", BORDER_HIGHLIGHT);
        assert_readable_on_both_themes("STATUS_FG", STATUS_FG);
        assert_readable_on_both_themes("STATUS_KEY", STATUS_KEY);
        assert_readable_on_both_themes("DIM", DIM);
        assert_readable_on_both_themes("SECONDARY_FG", SECONDARY_FG);

        for class in [
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
        ] {
            assert_readable_on_both_themes("classification", classification_color(&class));
        }

        assert_readable_on_both_themes("listen", state_color(&PortState::Listen));
        assert_readable_on_both_themes("established", state_color(&PortState::Established));

        for port in [80u16, 3000, 9000, 50000] {
            assert_readable_on_both_themes("port", port_color(port));
        }

        if let Some(fg) = ownership_style(&Ownership::Owned).fg {
            assert_readable_on_both_themes("owned", fg);
        }
        if let Some(fg) = ownership_style(&Ownership::Blocked).fg {
            assert_readable_on_both_themes("blocked", fg);
        }
    }

    #[test]
    fn selected_bar_carries_white_text() {
        // The selection bar is intentionally darker than the accent band; what
        // matters is that its white text stays high-contrast on top of it.
        let ratio = {
            let bar = luminance(SELECTED_BG);
            (1.0 + 0.05) / (bar + 0.05)
        };
        assert!(ratio >= 4.5, "white on SELECTED_BG is only {ratio:.1}:1");
    }
}
