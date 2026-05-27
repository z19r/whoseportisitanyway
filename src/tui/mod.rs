mod confirm;
mod detail;
mod keybindings;
pub mod style;
mod table;

use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;

use crate::classifier;
use crate::config::Config;
use crate::model::{Classification, PortEntry, PortState};
use crate::scanner;

const KONAMI_SEQUENCE: [KeyCode; 11] = [
    KeyCode::Up,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Char('b'),
    KeyCode::Char('a'),
    KeyCode::Enter,
];

#[derive(Debug)]
pub struct KonamiDetector {
    position: usize,
}

impl KonamiDetector {
    fn new() -> Self {
        Self { position: 0 }
    }

    fn feed(&mut self, key: KeyCode) -> bool {
        if key == KONAMI_SEQUENCE[self.position] {
            self.position += 1;
            if self.position == KONAMI_SEQUENCE.len() {
                self.position = 0;
                return true;
            }
        } else if key == KONAMI_SEQUENCE[0] {
            self.position = 1;
        } else {
            self.position = 0;
        }
        false
    }
}

#[derive(Debug, PartialEq, Eq)]
enum View {
    Table,
    Detail,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Port,
    Process,
    Type,
    Pid,
    State,
}

impl SortField {
    pub fn next(self) -> Self {
        match self {
            Self::Port => Self::Process,
            Self::Process => Self::Type,
            Self::Type => Self::Pid,
            Self::Pid => Self::State,
            Self::State => Self::Port,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Port => "port",
            Self::Process => "process",
            Self::Type => "type",
            Self::Pid => "pid",
            Self::State => "state",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Listen,
    Established,
    DevServer,
    Database,
    Docker,
    Proxy,
    Browser,
    SshTunnel,
    System,
}

impl Filter {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Listen,
            Self::Listen => Self::Established,
            Self::Established => Self::DevServer,
            Self::DevServer => Self::Database,
            Self::Database => Self::Docker,
            Self::Docker => Self::Proxy,
            Self::Proxy => Self::Browser,
            Self::Browser => Self::SshTunnel,
            Self::SshTunnel => Self::System,
            Self::System => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Listen => "listen",
            Self::Established => "established",
            Self::DevServer => "dev server",
            Self::Database => "database",
            Self::Docker => "docker",
            Self::Proxy => "proxy",
            Self::Browser => "browser",
            Self::SshTunnel => "ssh tunnel",
            Self::System => "system",
        }
    }

    pub fn matches(self, entry: &PortEntry) -> bool {
        match self {
            Self::All => true,
            Self::Listen => entry.state == PortState::Listen,
            Self::Established => entry.state == PortState::Established,
            Self::DevServer => entry.classification == Classification::DevServer,
            Self::Database => entry.classification == Classification::Database,
            Self::Docker => entry.classification == Classification::Docker,
            Self::Proxy => entry.classification == Classification::Proxy,
            Self::Browser => entry.classification == Classification::Browser,
            Self::SshTunnel => entry.classification == Classification::SshTunnel,
            Self::System => entry.classification == Classification::System,
        }
    }
}

pub struct App {
    all_entries: Vec<PortEntry>,
    entries: Vec<PortEntry>,
    selected: usize,
    view: View,
    should_quit: bool,
    watched_ports: Vec<u16>,
    sort_field: SortField,
    filter: Filter,
    pub konami: KonamiDetector,
    pub konami_mode: bool,
    shuffle_remaining: u8,
}

impl App {
    fn new(watched_ports: Vec<u16>) -> Self {
        Self {
            all_entries: Vec::new(),
            entries: Vec::new(),
            selected: 0,
            view: View::Table,
            should_quit: false,
            watched_ports,
            sort_field: SortField::Port,
            filter: Filter::All,
            konami: KonamiDetector::new(),
            konami_mode: false,
            shuffle_remaining: 0,
        }
    }

    fn shuffle_entries(&mut self) {
        let len = self.entries.len();
        if len <= 1 {
            return;
        }
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        let mut rng = seed;
        for i in (1..len).rev() {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = (rng as usize) % (i + 1);
            self.entries.swap(i, j);
        }
    }

    fn refresh(&mut self) -> Result<()> {
        let scanner = scanner::create_scanner();
        let raw_ports = scanner.scan()?;
        self.all_entries = classifier::classify_all(raw_ports, &self.watched_ports);
        self.apply_filter_sort();
        Ok(())
    }

    fn apply_filter_sort(&mut self) {
        let mut filtered: Vec<PortEntry> = self
            .all_entries
            .iter()
            .filter(|e| self.filter.matches(e))
            .cloned()
            .collect();

        match self.sort_field {
            SortField::Port => filtered.sort_by_key(|e| e.port),
            SortField::Process => filtered.sort_by_key(|e| e.process_name.clone()),
            SortField::Type => filtered.sort_by_key(|e| e.classification.to_string()),
            SortField::Pid => filtered.sort_by_key(|e| e.pid),
            SortField::State => filtered.sort_by_key(|e| e.state.to_string()),
        }

        self.entries = filtered;
        if self.selected >= self.entries.len() && !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
        if self.entries.is_empty() {
            self.selected = 0;
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_field = self.sort_field.next();
        self.apply_filter_sort();
    }

    fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.selected = 0;
        self.apply_filter_sort();
    }

    fn selected_entry(&self) -> Option<&PortEntry> {
        self.entries.get(self.selected)
    }
}

pub fn run(config: &Config) -> Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config.watched_ports.clone());
    app.refresh()?;

    let tick_rate = Duration::from_secs(config.refresh_interval_secs);
    let wild_rate = Duration::from_millis(50);

    loop {
        terminal.draw(|frame| render(&app, frame))?;

        let poll_rate = if app.konami_mode || app.shuffle_remaining > 0 {
            wild_rate
        } else {
            tick_rate
        };
        if event::poll(poll_rate)? {
            if let Event::Key(key) = event::read()? {
                if app.konami.feed(key.code) {
                    app.konami_mode = !app.konami_mode;
                    if app.konami_mode {
                        app.shuffle_remaining = 8;
                        app.shuffle_entries();
                    }
                } else {
                    handle_key(&mut app, key.code);
                }
            }
        } else if app.shuffle_remaining > 0 {
            app.shuffle_remaining -= 1;
            if app.shuffle_remaining == 0 {
                app.apply_filter_sort();
            } else {
                app.shuffle_entries();
            }
        } else if !app.konami_mode {
            app.refresh()?;
        }

        if app.should_quit {
            break;
        }
    }

    terminal::disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn render(app: &App, frame: &mut Frame) {
    table::render(app, frame);
    match app.view {
        View::Table => {}
        View::Detail => detail::render(app, frame),
        View::Confirm => confirm::render(app, frame),
    }
}

fn handle_key(app: &mut App, key: KeyCode) {
    match app.view {
        View::Table => keybindings::handle_table_key(app, key),
        View::Detail => keybindings::handle_detail_key(app, key),
        View::Confirm => keybindings::handle_confirm_key(app, key),
    }
}
