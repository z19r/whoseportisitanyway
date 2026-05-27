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
