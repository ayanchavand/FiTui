use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::app::{App, Mode};

pub fn handle_key(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    match app.mode {
        Mode::Normal => handle_normal(app, key, conn),
        Mode::Adding => handle_form(app, key, conn),
        Mode::Stats => handle_stats(app, key),
    }
}

/* ============================
   🟢 Normal Mode Handler
============================ */

fn handle_normal(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    let len = app.transactions.len();

    match key {
        // Quit app
        KeyCode::Char('q') => return true,

        // Enter Add Transaction mode
        KeyCode::Char('a') => {
            app.mode = Mode::Adding;
        }

        // Open Stats page
        KeyCode::Char('s') => {
            app.mode = Mode::Stats;
        }

        // Move selection up
        KeyCode::Up => {
            if app.selected > 0 {
                app.selected -= 1;
            }
        }

        // Move selection down
        KeyCode::Down => {
            if app.selected + 1 < len {
                app.selected += 1;
            }
        }

        // Delete selected transaction ✅ FIXED
        KeyCode::Char('d') => {
            app.delete_selected(conn);
        }

        _ => {}
    }

    false
}

/* ============================
   ✍️ Add Transaction Form Mode
============================ */

fn handle_form(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    match key {
        // Cancel form
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }

        // Next field
        KeyCode::Tab => {
            app.form.active = app.form.active.next();
        }

        // Toggle Kind / Cycle Tag
        KeyCode::Left | KeyCode::Right => match app.form.active {
            crate::form::Field::Kind => app.form.toggle_kind(),
            crate::form::Field::Tag => app.form.next_tag(),
            _ => {}
        },

        // Backspace
        KeyCode::Backspace => {
            app.form.pop_char();
        }

        // Typing into active field
        KeyCode::Char(c) => {
            app.form.push_char(c);
        }

        // Save transaction
        KeyCode::Enter => {
            app.save_transaction(conn);
            app.form.reset();
            app.mode = Mode::Normal;
        }

        _ => {}
    }

    false
}

/* ============================
   📊 Stats Mode Handler
============================ */

fn handle_stats(app: &mut App, key: KeyCode) -> bool {
    match key {
        // Exit Stats page back to normal
        KeyCode::Char('q') => {
            app.mode = Mode::Normal;
        }

        _ => {}
    }

    false
}
