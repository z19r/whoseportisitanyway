use crossterm::event::KeyCode;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

use super::{App, View};

pub fn handle_table_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down if !app.entries.is_empty() => {
            app.selected = (app.selected + 1).min(app.entries.len() - 1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.selected = app.selected.saturating_sub(1);
        }
        KeyCode::Char('G') if !app.entries.is_empty() => {
            app.selected = app.entries.len() - 1;
        }
        KeyCode::Char('g') => {
            app.selected = 0;
        }
        KeyCode::Enter | KeyCode::Char('d') if app.selected_entry().is_some() => {
            app.view = View::Detail;
        }
        KeyCode::Char('x') if app.selected_entry().is_some() => {
            app.view = View::Confirm;
        }
        KeyCode::Char('s') => app.cycle_sort(),
        KeyCode::Char('f') => app.cycle_filter(),
        KeyCode::Char('r') => {
            let _ = app.refresh();
        }
        _ => {}
    }
}

pub fn handle_detail_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.view = View::Table,
        KeyCode::Char('x') => app.view = View::Confirm,
        _ => {}
    }
}

pub fn handle_confirm_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('y') => {
            if let Some(entry) = app.selected_entry() {
                if entry.pid != 0 {
                    let _ = signal::kill(Pid::from_raw(entry.pid as i32), Signal::SIGTERM);
                }
            }
            app.view = View::Table;
            let _ = app.refresh();
        }
        KeyCode::Char('n') | KeyCode::Esc => app.view = View::Table,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Classification, Ownership, PortEntry, PortState, Protocol};

    fn test_app(n: usize) -> App {
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
        App {
            all_entries: entries.clone(),
            entries,
            selected: 0,
            view: View::Table,
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
    fn table_q_quits() {
        let mut app = test_app(3);
        handle_table_key(&mut app, KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn table_esc_quits() {
        let mut app = test_app(3);
        handle_table_key(&mut app, KeyCode::Esc);
        assert!(app.should_quit);
    }

    #[test]
    fn table_j_moves_down() {
        let mut app = test_app(5);
        handle_table_key(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn table_down_moves_down() {
        let mut app = test_app(5);
        handle_table_key(&mut app, KeyCode::Down);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn table_k_moves_up() {
        let mut app = test_app(5);
        app.selected = 3;
        handle_table_key(&mut app, KeyCode::Char('k'));
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn table_up_moves_up() {
        let mut app = test_app(5);
        app.selected = 2;
        handle_table_key(&mut app, KeyCode::Up);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn table_k_at_zero_stays() {
        let mut app = test_app(5);
        handle_table_key(&mut app, KeyCode::Char('k'));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn table_g_goes_to_top() {
        let mut app = test_app(5);
        app.selected = 4;
        handle_table_key(&mut app, KeyCode::Char('g'));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn table_big_g_goes_to_bottom() {
        let mut app = test_app(5);
        handle_table_key(&mut app, KeyCode::Char('G'));
        assert_eq!(app.selected, 4);
    }

    #[test]
    fn table_enter_opens_detail() {
        let mut app = test_app(3);
        handle_table_key(&mut app, KeyCode::Enter);
        assert_eq!(app.view, View::Detail);
    }

    #[test]
    fn table_d_opens_detail() {
        let mut app = test_app(3);
        handle_table_key(&mut app, KeyCode::Char('d'));
        assert_eq!(app.view, View::Detail);
    }

    #[test]
    fn table_x_opens_confirm() {
        let mut app = test_app(3);
        handle_table_key(&mut app, KeyCode::Char('x'));
        assert_eq!(app.view, View::Confirm);
    }

    #[test]
    fn table_s_cycles_sort() {
        let mut app = test_app(3);
        assert_eq!(app.sort_field, super::super::SortField::Port);
        handle_table_key(&mut app, KeyCode::Char('s'));
        assert_eq!(app.sort_field, super::super::SortField::Process);
    }

    #[test]
    fn table_f_cycles_filter() {
        let mut app = test_app(3);
        assert_eq!(app.filter, super::super::Filter::All);
        handle_table_key(&mut app, KeyCode::Char('f'));
        assert_eq!(app.filter, super::super::Filter::Listen);
    }

    #[test]
    fn table_j_clamps_at_end() {
        let mut app = test_app(3);
        app.selected = 2;
        handle_table_key(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn table_empty_j_noop() {
        let mut app = test_app(0);
        handle_table_key(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn table_empty_big_g_noop() {
        let mut app = test_app(0);
        handle_table_key(&mut app, KeyCode::Char('G'));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn table_unknown_key_noop() {
        let mut app = test_app(3);
        handle_table_key(&mut app, KeyCode::Char('z'));
        assert_eq!(app.view, View::Table);
        assert!(!app.should_quit);
    }

    #[test]
    fn table_enter_empty_noop() {
        let mut app = test_app(0);
        handle_table_key(&mut app, KeyCode::Enter);
        assert_eq!(app.view, View::Table);
    }

    #[test]
    fn table_x_empty_noop() {
        let mut app = test_app(0);
        handle_table_key(&mut app, KeyCode::Char('x'));
        assert_eq!(app.view, View::Table);
    }

    #[test]
    fn detail_q_back_to_table() {
        let mut app = test_app(3);
        app.view = View::Detail;
        handle_detail_key(&mut app, KeyCode::Char('q'));
        assert_eq!(app.view, View::Table);
    }

    #[test]
    fn detail_esc_back_to_table() {
        let mut app = test_app(3);
        app.view = View::Detail;
        handle_detail_key(&mut app, KeyCode::Esc);
        assert_eq!(app.view, View::Table);
    }

    #[test]
    fn detail_x_opens_confirm() {
        let mut app = test_app(3);
        app.view = View::Detail;
        handle_detail_key(&mut app, KeyCode::Char('x'));
        assert_eq!(app.view, View::Confirm);
    }

    #[test]
    fn detail_unknown_noop() {
        let mut app = test_app(3);
        app.view = View::Detail;
        handle_detail_key(&mut app, KeyCode::Char('z'));
        assert_eq!(app.view, View::Detail);
    }

    #[test]
    fn confirm_n_back_to_table() {
        let mut app = test_app(3);
        app.view = View::Confirm;
        handle_confirm_key(&mut app, KeyCode::Char('n'));
        assert_eq!(app.view, View::Table);
    }

    #[test]
    fn confirm_esc_back_to_table() {
        let mut app = test_app(3);
        app.view = View::Confirm;
        handle_confirm_key(&mut app, KeyCode::Esc);
        assert_eq!(app.view, View::Table);
    }

    #[test]
    fn confirm_unknown_noop() {
        let mut app = test_app(3);
        app.view = View::Confirm;
        handle_confirm_key(&mut app, KeyCode::Char('z'));
        assert_eq!(app.view, View::Confirm);
    }
}
