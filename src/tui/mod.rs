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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupField {
    None,
    Type,
    Project,
    Process,
    State,
}

impl GroupField {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Type,
            Self::Type => Self::Project,
            Self::Project => Self::Process,
            Self::Process => Self::State,
            Self::State => Self::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Type => "type",
            Self::Project => "project",
            Self::Process => "process",
            Self::State => "state",
        }
    }

    pub fn group_key(self, entry: &PortEntry) -> String {
        match self {
            Self::None => String::new(),
            Self::Type => entry.classification.to_string(),
            Self::Project => entry
                .project
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "\u{2014}".to_string()),
            Self::Process => entry.process_name.clone(),
            Self::State => entry.state.to_string(),
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
    pub group_field: GroupField,
    /// Group label for each entry in `entries`. When adjacent labels differ,
    /// the table renderer inserts a visual group-header row.
    pub group_labels: Vec<String>,
    pub konami: KonamiDetector,
    pub konami_mode: bool,
    shuffle_remaining: u8,
    pub(crate) hide_system: bool,
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
            group_field: GroupField::None,
            group_labels: Vec::new(),
            konami: KonamiDetector::new(),
            konami_mode: false,
            shuffle_remaining: 0,
            hide_system: false,
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
        let prev_identity = self
            .entries
            .get(self.selected)
            .map(|e| (e.port, e.protocol, e.pid));

        let scanner = scanner::create_scanner();
        let raw_ports = scanner.scan()?;
        self.all_entries = classifier::classify_all(raw_ports, &self.watched_ports);
        self.apply_filter_sort();

        if let Some((port, protocol, pid)) = prev_identity {
            match self
                .entries
                .iter()
                .position(|e| e.port == port && e.protocol == protocol && e.pid == pid)
            {
                Some(new_idx) => self.selected = new_idx,
                None if self.view != View::Table => self.view = View::Table,
                None => {}
            }
        }

        Ok(())
    }

    fn apply_filter_sort(&mut self) {
        let hide_sys = self.hide_system;
        let mut filtered: Vec<PortEntry> = self
            .all_entries
            .iter()
            .filter(|e| !hide_sys || e.classification != Classification::System)
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

        // When grouping is active, stable-sort by group key so that the
        // within-group order established above is preserved.
        if self.group_field != GroupField::None {
            let gf = self.group_field;
            filtered.sort_by_key(|a| gf.group_key(a));
        }

        self.group_labels = if self.group_field == GroupField::None {
            Vec::new()
        } else {
            filtered
                .iter()
                .map(|e| self.group_field.group_key(e))
                .collect()
        };

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

    fn cycle_group(&mut self) {
        self.group_field = self.group_field.next();
        self.selected = 0;
        self.apply_filter_sort();
    }

    pub fn toggle_hide_system(&mut self) {
        self.hide_system = !self.hide_system;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn konami_new_starts_at_zero() {
        let k = KonamiDetector::new();
        assert_eq!(k.position, 0);
    }

    #[test]
    fn konami_full_sequence_returns_true() {
        let mut k = KonamiDetector::new();
        for &code in &KONAMI_SEQUENCE[..KONAMI_SEQUENCE.len() - 1] {
            assert!(!k.feed(code));
        }
        assert!(k.feed(KONAMI_SEQUENCE[KONAMI_SEQUENCE.len() - 1]));
    }

    #[test]
    fn konami_resets_after_complete() {
        let mut k = KonamiDetector::new();
        for &code in &KONAMI_SEQUENCE {
            k.feed(code);
        }
        assert_eq!(k.position, 0);
    }

    #[test]
    fn konami_wrong_key_resets() {
        let mut k = KonamiDetector::new();
        k.feed(KeyCode::Up);
        k.feed(KeyCode::Up);
        assert_eq!(k.position, 2);
        k.feed(KeyCode::Char('x'));
        assert_eq!(k.position, 0);
    }

    #[test]
    fn konami_wrong_key_restarts_if_first() {
        let mut k = KonamiDetector::new();
        k.feed(KeyCode::Up);
        k.feed(KeyCode::Up);
        k.feed(KeyCode::Up);
        assert_eq!(k.position, 1);
    }

    #[test]
    fn sort_field_cycles_through_all() {
        let mut sf = SortField::Port;
        let mut seen = vec![sf];
        for _ in 0..4 {
            sf = sf.next();
            seen.push(sf);
        }
        assert_eq!(sf.next(), SortField::Port);
        assert_eq!(
            seen,
            vec![
                SortField::Port,
                SortField::Process,
                SortField::Type,
                SortField::Pid,
                SortField::State,
            ]
        );
    }

    #[test]
    fn sort_field_labels() {
        assert_eq!(SortField::Port.label(), "port");
        assert_eq!(SortField::Process.label(), "process");
        assert_eq!(SortField::Type.label(), "type");
        assert_eq!(SortField::Pid.label(), "pid");
        assert_eq!(SortField::State.label(), "state");
    }

    #[test]
    fn filter_cycles_through_all() {
        let mut f = Filter::All;
        let mut seen = vec![f];
        for _ in 0..9 {
            f = f.next();
            seen.push(f);
        }
        assert_eq!(f.next(), Filter::All);
        assert_eq!(
            seen,
            vec![
                Filter::All,
                Filter::Listen,
                Filter::Established,
                Filter::DevServer,
                Filter::Database,
                Filter::Docker,
                Filter::Proxy,
                Filter::Browser,
                Filter::SshTunnel,
                Filter::System,
            ]
        );
    }

    #[test]
    fn filter_labels() {
        assert_eq!(Filter::All.label(), "all");
        assert_eq!(Filter::Listen.label(), "listen");
        assert_eq!(Filter::Established.label(), "established");
        assert_eq!(Filter::DevServer.label(), "dev server");
        assert_eq!(Filter::Database.label(), "database");
        assert_eq!(Filter::Docker.label(), "docker");
        assert_eq!(Filter::Proxy.label(), "proxy");
        assert_eq!(Filter::Browser.label(), "browser");
        assert_eq!(Filter::SshTunnel.label(), "ssh tunnel");
        assert_eq!(Filter::System.label(), "system");
    }

    fn make_entry(class: Classification, state: PortState) -> PortEntry {
        use crate::model::{Ownership, Protocol};
        PortEntry {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 100,
            process_name: "test".into(),
            command_line: "test".into(),
            state,
            classification: class,
            project: None,
            local_addr: "127.0.0.1:3000".into(),
            all_addrs: vec!["127.0.0.1:3000".into()],
            ownership: Ownership::Untracked,
            uid: None,
            user: None,
            remote_addr: None,
        }
    }

    #[test]
    fn filter_all_matches_everything() {
        let e = make_entry(Classification::Docker, PortState::Established);
        assert!(Filter::All.matches(&e));
    }

    #[test]
    fn filter_listen_matches_state() {
        let listen = make_entry(Classification::DevServer, PortState::Listen);
        let est = make_entry(Classification::DevServer, PortState::Established);
        assert!(Filter::Listen.matches(&listen));
        assert!(!Filter::Listen.matches(&est));
    }

    #[test]
    fn filter_established_matches_state() {
        let est = make_entry(Classification::DevServer, PortState::Established);
        let listen = make_entry(Classification::DevServer, PortState::Listen);
        assert!(Filter::Established.matches(&est));
        assert!(!Filter::Established.matches(&listen));
    }

    #[test]
    fn filter_devserver_matches_classification() {
        let dev = make_entry(Classification::DevServer, PortState::Listen);
        let docker = make_entry(Classification::Docker, PortState::Listen);
        assert!(Filter::DevServer.matches(&dev));
        assert!(!Filter::DevServer.matches(&docker));
    }

    #[test]
    fn filter_database_matches() {
        assert!(Filter::Database.matches(&make_entry(Classification::Database, PortState::Listen)));
    }

    #[test]
    fn filter_docker_matches() {
        assert!(Filter::Docker.matches(&make_entry(Classification::Docker, PortState::Listen)));
    }

    #[test]
    fn filter_proxy_matches() {
        assert!(Filter::Proxy.matches(&make_entry(Classification::Proxy, PortState::Listen)));
    }

    #[test]
    fn filter_browser_matches() {
        assert!(Filter::Browser.matches(&make_entry(Classification::Browser, PortState::Listen)));
    }

    #[test]
    fn filter_ssh_matches() {
        assert!(
            Filter::SshTunnel.matches(&make_entry(Classification::SshTunnel, PortState::Listen))
        );
    }

    #[test]
    fn filter_system_matches() {
        assert!(Filter::System.matches(&make_entry(Classification::System, PortState::Listen)));
    }

    #[test]
    fn app_new_defaults() {
        let app = App::new(vec![3000, 8080]);
        assert!(app.all_entries.is_empty());
        assert_eq!(app.selected, 0);
        assert_eq!(app.view, View::Table);
        assert!(!app.should_quit);
        assert_eq!(app.sort_field, SortField::Port);
        assert_eq!(app.filter, Filter::All);
        assert!(!app.konami_mode);
        assert!(!app.hide_system);
    }

    #[test]
    fn hide_system_default_false() {
        let app = App::new(vec![]);
        assert!(!app.hide_system);
    }

    #[test]
    fn toggle_hide_system_filters_system_entries() {
        use crate::model::{Ownership, Protocol};
        let mut app = App::new(vec![]);
        app.all_entries = vec![
            PortEntry {
                port: 22,
                protocol: Protocol::Tcp,
                pid: 1,
                process_name: "sshd".into(),
                command_line: "sshd".into(),
                classification: Classification::System,
                ownership: Ownership::Untracked,
                state: PortState::Listen,
                local_addr: "0.0.0.0:22".into(),
                all_addrs: vec!["0.0.0.0:22".into()],
                project: None,
                uid: None,
                user: None,
                remote_addr: None,
            },
            PortEntry {
                port: 3000,
                protocol: Protocol::Tcp,
                pid: 100,
                process_name: "node".into(),
                command_line: "node server.js".into(),
                classification: Classification::DevServer,
                ownership: Ownership::Untracked,
                state: PortState::Listen,
                local_addr: "0.0.0.0:3000".into(),
                all_addrs: vec!["0.0.0.0:3000".into()],
                project: None,
                uid: None,
                user: None,
                remote_addr: None,
            },
        ];
        app.apply_filter_sort();
        assert_eq!(app.entries.len(), 2, "both entries visible before toggle");

        app.toggle_hide_system();

        assert!(app.hide_system);
        assert_eq!(app.entries.len(), 1, "only non-system entry should remain");
        assert_eq!(app.entries[0].port, 3000);
    }

    #[test]
    fn toggle_hide_system_twice_restores() {
        use crate::model::{Ownership, Protocol};
        let mut app = App::new(vec![]);
        app.all_entries = vec![
            PortEntry {
                port: 22,
                protocol: Protocol::Tcp,
                pid: 1,
                process_name: "sshd".into(),
                command_line: "sshd".into(),
                classification: Classification::System,
                ownership: Ownership::Untracked,
                state: PortState::Listen,
                local_addr: "0.0.0.0:22".into(),
                all_addrs: vec!["0.0.0.0:22".into()],
                project: None,
                uid: None,
                user: None,
                remote_addr: None,
            },
            PortEntry {
                port: 3000,
                protocol: Protocol::Tcp,
                pid: 100,
                process_name: "node".into(),
                command_line: "node server.js".into(),
                classification: Classification::DevServer,
                ownership: Ownership::Untracked,
                state: PortState::Listen,
                local_addr: "0.0.0.0:3000".into(),
                all_addrs: vec!["0.0.0.0:3000".into()],
                project: None,
                uid: None,
                user: None,
                remote_addr: None,
            },
        ];
        app.apply_filter_sort();

        app.toggle_hide_system();
        assert_eq!(app.entries.len(), 1);

        app.toggle_hide_system();
        assert!(!app.hide_system);
        assert_eq!(
            app.entries.len(),
            2,
            "both entries restored after second toggle"
        );
    }

    #[test]
    fn app_selected_entry_none_when_empty() {
        let app = App::new(vec![]);
        assert!(app.selected_entry().is_none());
    }

    #[test]
    fn app_apply_filter_sort_caps_selected() {
        let mut app = App::new(vec![]);
        app.all_entries = vec![
            make_entry(Classification::DevServer, PortState::Listen),
            make_entry(Classification::Docker, PortState::Listen),
        ];
        app.selected = 5;
        app.apply_filter_sort();
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn app_cycle_sort() {
        let mut app = App::new(vec![]);
        assert_eq!(app.sort_field, SortField::Port);
        app.cycle_sort();
        assert_eq!(app.sort_field, SortField::Process);
    }

    #[test]
    fn app_cycle_filter() {
        let mut app = App::new(vec![]);
        app.all_entries = vec![make_entry(Classification::DevServer, PortState::Listen)];
        app.selected = 0;
        app.cycle_filter();
        assert_eq!(app.filter, Filter::Listen);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn app_shuffle_entries_single_noop() {
        let mut app = App::new(vec![]);
        app.entries = vec![make_entry(Classification::DevServer, PortState::Listen)];
        app.shuffle_entries();
        assert_eq!(app.entries.len(), 1);
    }

    // Helpers for check_modal_validity tests
    fn make_entry_custom(
        port: u16,
        pid: u32,
        class: Classification,
        state: PortState,
    ) -> PortEntry {
        use crate::model::{Ownership, Protocol};
        PortEntry {
            port,
            protocol: Protocol::Tcp,
            pid,
            process_name: "test".into(),
            command_line: "test".into(),
            state,
            classification: class,
            project: None,
            local_addr: format!("127.0.0.1:{port}"),
            all_addrs: vec![format!("127.0.0.1:{port}")],
            ownership: Ownership::Untracked,
            uid: None,
            user: None,
            remote_addr: None,
        }
    }

    fn simulate_refresh(app: &mut App, new_entries: Vec<PortEntry>) {
        let prev_identity = app
            .entries
            .get(app.selected)
            .map(|e| (e.port, e.protocol, e.pid));

        app.all_entries = new_entries;
        app.apply_filter_sort();

        if let Some((port, protocol, pid)) = prev_identity {
            match app
                .entries
                .iter()
                .position(|e| e.port == port && e.protocol == protocol && e.pid == pid)
            {
                Some(new_idx) => app.selected = new_idx,
                None if app.view != View::Table => app.view = View::Table,
                None => {}
            }
        }
    }

    #[test]
    fn refresh_closes_detail_when_entry_gone() {
        let mut app = App::new(vec![]);
        app.entries = vec![make_entry_custom(
            3000,
            100,
            Classification::DevServer,
            PortState::Listen,
        )];
        app.all_entries = app.entries.clone();
        app.selected = 0;
        app.view = View::Detail;

        simulate_refresh(&mut app, vec![]);

        assert_eq!(
            app.view,
            View::Table,
            "modal should close when port disappears"
        );
    }

    #[test]
    fn refresh_closes_confirm_when_entry_gone() {
        let mut app = App::new(vec![]);
        app.entries = vec![make_entry_custom(
            8080,
            200,
            Classification::Database,
            PortState::Listen,
        )];
        app.all_entries = app.entries.clone();
        app.selected = 0;
        app.view = View::Confirm;

        simulate_refresh(&mut app, vec![]);

        assert_eq!(
            app.view,
            View::Table,
            "confirm modal should close when port disappears"
        );
    }

    #[test]
    fn refresh_keeps_detail_when_entry_present() {
        let mut app = App::new(vec![]);
        let entry = make_entry_custom(3000, 100, Classification::DevServer, PortState::Listen);
        app.all_entries = vec![entry.clone()];
        app.apply_filter_sort();
        app.selected = 0;
        app.view = View::Detail;

        simulate_refresh(&mut app, vec![entry]);

        assert_eq!(app.view, View::Detail, "modal should stay open");
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn refresh_updates_index_when_entry_moves() {
        let mut app = App::new(vec![]);
        app.entries = vec![
            make_entry_custom(3000, 100, Classification::DevServer, PortState::Listen),
            make_entry_custom(8080, 200, Classification::Database, PortState::Listen),
        ];
        app.all_entries = app.entries.clone();
        app.selected = 1;
        app.view = View::Detail;

        simulate_refresh(
            &mut app,
            vec![make_entry_custom(
                8080,
                200,
                Classification::Database,
                PortState::Listen,
            )],
        );

        assert_eq!(app.view, View::Detail, "modal should stay open");
        assert_eq!(app.selected, 0, "selected should follow entry to new index");
    }

    #[test]
    fn refresh_table_view_keeps_selection_stable() {
        let mut app = App::new(vec![]);
        app.entries = vec![
            make_entry_custom(3000, 100, Classification::DevServer, PortState::Listen),
            make_entry_custom(8080, 200, Classification::Database, PortState::Listen),
        ];
        app.all_entries = app.entries.clone();
        app.selected = 1;
        app.view = View::Table;

        simulate_refresh(
            &mut app,
            vec![
                make_entry_custom(8080, 200, Classification::Database, PortState::Listen),
                make_entry_custom(3000, 100, Classification::DevServer, PortState::Listen),
            ],
        );

        assert_eq!(app.view, View::Table);
        let sel = &app.entries[app.selected];
        assert_eq!(
            sel.port, 8080,
            "highlight should track port 8080, not stay at index 1"
        );
    }

    #[test]
    fn refresh_table_view_entry_gone_stays_in_table() {
        let mut app = App::new(vec![]);
        app.entries = vec![make_entry_custom(
            3000,
            100,
            Classification::DevServer,
            PortState::Listen,
        )];
        app.all_entries = app.entries.clone();
        app.selected = 0;
        app.view = View::Table;

        simulate_refresh(&mut app, vec![]);

        assert_eq!(app.view, View::Table, "table view should not change");
    }

    // ---- GroupField tests ----

    #[test]
    fn group_field_cycles_through_all() {
        let mut gf = GroupField::None;
        let mut seen = vec![gf];
        for _ in 0..4 {
            gf = gf.next();
            seen.push(gf);
        }
        // After 5 steps should be back to None
        assert_eq!(gf.next(), GroupField::None);
        assert_eq!(
            seen,
            vec![
                GroupField::None,
                GroupField::Type,
                GroupField::Project,
                GroupField::Process,
                GroupField::State,
            ]
        );
    }

    #[test]
    fn group_field_labels() {
        assert_eq!(GroupField::None.label(), "none");
        assert_eq!(GroupField::Type.label(), "type");
        assert_eq!(GroupField::Project.label(), "project");
        assert_eq!(GroupField::Process.label(), "process");
        assert_eq!(GroupField::State.label(), "state");
    }

    #[test]
    fn group_field_group_key_none_is_empty() {
        let entry = make_entry(Classification::DevServer, PortState::Listen);
        assert_eq!(GroupField::None.group_key(&entry), "");
    }

    #[test]
    fn group_field_group_key_type() {
        let entry = make_entry(Classification::Database, PortState::Listen);
        assert_eq!(GroupField::Type.group_key(&entry), "Database");
    }

    #[test]
    fn group_field_group_key_project_some() {
        use crate::model::Project;
        let mut entry = make_entry(Classification::DevServer, PortState::Listen);
        entry.project = Some(Project {
            name: "myapp".into(),
            root: "/tmp/myapp".into(),
            framework: None,
        });
        assert_eq!(GroupField::Project.group_key(&entry), "myapp");
    }

    #[test]
    fn group_field_group_key_project_none() {
        let entry = make_entry(Classification::DevServer, PortState::Listen);
        // No project — should return em-dash placeholder
        assert_eq!(GroupField::Project.group_key(&entry), "\u{2014}");
    }

    #[test]
    fn group_field_group_key_process() {
        use crate::model::{Ownership, Protocol};
        let entry = PortEntry {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 100,
            process_name: "node".into(),
            command_line: "node index.js".into(),
            state: PortState::Listen,
            classification: Classification::DevServer,
            project: None,
            local_addr: "127.0.0.1:3000".into(),
            all_addrs: vec!["127.0.0.1:3000".into()],
            ownership: Ownership::Untracked,
            uid: None,
            user: None,
            remote_addr: None,
        };
        assert_eq!(GroupField::Process.group_key(&entry), "node");
    }

    #[test]
    fn group_field_group_key_state() {
        let listen = make_entry(Classification::DevServer, PortState::Listen);
        let est = make_entry(Classification::DevServer, PortState::Established);
        assert_eq!(GroupField::State.group_key(&listen), "LISTEN");
        assert_eq!(GroupField::State.group_key(&est), "ESTABLISHED");
    }

    #[test]
    fn apply_filter_sort_groups_entries_by_type() {
        use crate::model::{Ownership, Protocol};
        let mut app = App::new(vec![]);
        // Mix of DevServer and Database entries — without grouping they'd sort by port
        app.all_entries = vec![
            PortEntry {
                port: 5432,
                protocol: Protocol::Tcp,
                pid: 1,
                process_name: "postgres".into(),
                command_line: "postgres".into(),
                state: PortState::Listen,
                classification: Classification::Database,
                project: None,
                local_addr: "127.0.0.1:5432".into(),
                all_addrs: vec![],
                ownership: Ownership::Untracked,
                uid: None,
                user: None,
                remote_addr: None,
            },
            PortEntry {
                port: 3000,
                protocol: Protocol::Tcp,
                pid: 2,
                process_name: "node".into(),
                command_line: "node".into(),
                state: PortState::Listen,
                classification: Classification::DevServer,
                project: None,
                local_addr: "127.0.0.1:3000".into(),
                all_addrs: vec![],
                ownership: Ownership::Untracked,
                uid: None,
                user: None,
                remote_addr: None,
            },
            PortEntry {
                port: 5433,
                protocol: Protocol::Tcp,
                pid: 3,
                process_name: "postgres2".into(),
                command_line: "postgres2".into(),
                state: PortState::Listen,
                classification: Classification::Database,
                project: None,
                local_addr: "127.0.0.1:5433".into(),
                all_addrs: vec![],
                ownership: Ownership::Untracked,
                uid: None,
                user: None,
                remote_addr: None,
            },
        ];
        app.group_field = GroupField::Type;
        app.apply_filter_sort();

        // All Database entries should come before DevServer (alphabetical: "Database" < "Dev Server")
        let labels: Vec<&str> = app.group_labels.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            labels[0], labels[1],
            "first two entries should share a group"
        );
        assert_ne!(
            labels[1], labels[2],
            "third entry should be in a different group"
        );
        // Verify the actual groups
        assert_eq!(app.entries[0].classification, Classification::Database);
        assert_eq!(app.entries[1].classification, Classification::Database);
        assert_eq!(app.entries[2].classification, Classification::DevServer);
    }

    #[test]
    fn apply_filter_sort_groups_preserve_sort_within_group() {
        use crate::model::{Ownership, Protocol};
        let mut app = App::new(vec![]);
        // Two database entries — port sort within group should keep 5432 before 5433
        app.all_entries = vec![
            PortEntry {
                port: 5433,
                protocol: Protocol::Tcp,
                pid: 2,
                process_name: "pg2".into(),
                command_line: "pg2".into(),
                state: PortState::Listen,
                classification: Classification::Database,
                project: None,
                local_addr: "127.0.0.1:5433".into(),
                all_addrs: vec![],
                ownership: Ownership::Untracked,
                uid: None,
                user: None,
                remote_addr: None,
            },
            PortEntry {
                port: 5432,
                protocol: Protocol::Tcp,
                pid: 1,
                process_name: "pg1".into(),
                command_line: "pg1".into(),
                state: PortState::Listen,
                classification: Classification::Database,
                project: None,
                local_addr: "127.0.0.1:5432".into(),
                all_addrs: vec![],
                ownership: Ownership::Untracked,
                uid: None,
                user: None,
                remote_addr: None,
            },
        ];
        app.sort_field = SortField::Port;
        app.group_field = GroupField::Type;
        app.apply_filter_sort();

        // Both are Database; within-group order should be by port ascending
        assert_eq!(app.entries[0].port, 5432);
        assert_eq!(app.entries[1].port, 5433);
    }

    #[test]
    fn group_labels_len_matches_entries_len() {
        use crate::model::{Ownership, Protocol};
        let mut app = App::new(vec![]);
        app.all_entries = vec![
            PortEntry {
                port: 3000,
                protocol: Protocol::Tcp,
                pid: 1,
                process_name: "node".into(),
                command_line: "node".into(),
                state: PortState::Listen,
                classification: Classification::DevServer,
                project: None,
                local_addr: "127.0.0.1:3000".into(),
                all_addrs: vec![],
                ownership: Ownership::Untracked,
                uid: None,
                user: None,
                remote_addr: None,
            },
            PortEntry {
                port: 5432,
                protocol: Protocol::Tcp,
                pid: 2,
                process_name: "postgres".into(),
                command_line: "postgres".into(),
                state: PortState::Listen,
                classification: Classification::Database,
                project: None,
                local_addr: "127.0.0.1:5432".into(),
                all_addrs: vec![],
                ownership: Ownership::Untracked,
                uid: None,
                user: None,
                remote_addr: None,
            },
        ];
        app.group_field = GroupField::Process;
        app.apply_filter_sort();

        assert_eq!(app.group_labels.len(), app.entries.len());
    }

    #[test]
    fn group_labels_empty_when_no_entries() {
        let mut app = App::new(vec![]);
        app.group_field = GroupField::Type;
        app.apply_filter_sort();
        assert!(app.group_labels.is_empty());
    }

    #[test]
    fn app_new_defaults_group_field() {
        let app = App::new(vec![]);
        assert_eq!(app.group_field, GroupField::None);
        assert!(app.group_labels.is_empty());
    }

    #[test]
    fn app_cycle_group() {
        let mut app = App::new(vec![]);
        assert_eq!(app.group_field, GroupField::None);
        app.cycle_group();
        assert_eq!(app.group_field, GroupField::Type);
        app.cycle_group();
        assert_eq!(app.group_field, GroupField::Project);
    }

    #[test]
    fn app_cycle_group_resets_selected() {
        use crate::model::{Ownership, Protocol};
        let mut app = App::new(vec![]);
        app.all_entries = vec![PortEntry {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 1,
            process_name: "node".into(),
            command_line: "node".into(),
            state: PortState::Listen,
            classification: Classification::DevServer,
            project: None,
            local_addr: "127.0.0.1:3000".into(),
            all_addrs: vec![],
            ownership: Ownership::Untracked,
            uid: None,
            user: None,
            remote_addr: None,
        }];
        app.apply_filter_sort();
        app.selected = 0;
        app.cycle_group();
        assert_eq!(app.selected, 0, "selected should reset to 0 on group cycle");
    }
}
