use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::app::{App, Mode};
use crate::stats;

pub fn handle_key(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    match app.mode {
        Mode::Normal => handle_normal(app, key, conn),
        Mode::Adding => handle_form(app, key, conn),
        Mode::Stats => stats::handle_stats(app, key),
    }
}

fn handle_normal(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    let len = app.transactions.len();

    match key {
        KeyCode::Char('q') => return true,

        KeyCode::Char('a') => {
            app.form.reset();
            app.editing = None;
            app.mode = Mode::Adding;
        }

        KeyCode::Char('s') => {
            app.mode = Mode::Stats;
        }

        KeyCode::Up => {
            if app.selected > 0 {
                app.selected -= 1;
            }
        }

        KeyCode::Down => {
            if app.selected + 1 < len {
                app.selected += 1;
            }
        }

        KeyCode::Char('d') => {
            app.delete_selected(conn);
        }

        KeyCode::Char('e') => {
            // Begin editing the currently selected transaction
            app.begin_edit_selected();
        }

        _ => {}
    }

    false
}

fn handle_form(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    match key {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.editing = None;
            app.form.reset();
        }

        KeyCode::Tab => {
            app.form.active = app.form.active.next();
        }

        // Arrow keys toggle Kind, cycle Tags, or toggle Recurring depending on active field
        KeyCode::Right => match app.form.active {
            crate::form::Field::Kind => app.form.toggle_kind(),
            crate::form::Field::Tag => app.form.next_tag(app.tags.len()),
            crate::form::Field::Recurring => app.form.toggle_recurring(),
            _ => {}
        },

        KeyCode::Left => match app.form.active {
            crate::form::Field::Kind => app.form.toggle_kind(),
            crate::form::Field::Tag => app.form.prev_tag(app.tags.len()),
            crate::form::Field::Recurring => app.form.toggle_recurring(),
            _ => {}
        },

        KeyCode::Backspace => {
            app.form.pop_char();
        }

        KeyCode::Char(c) => {
            app.form.push_char(c);
        }

        KeyCode::Enter => {
            app.save_transaction(conn);
            app.form.reset();
            app.mode = Mode::Normal;
        }

        _ => {}
    }

    false
}
