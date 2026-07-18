use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::app::{App, Mode, PopupAction, PopupKind};
use crate::stats;

pub fn handle_key(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    // global tab/arrow handling applies when we're in any of the
    // "main" views. Adding/popup mode shouldn't switch tabs.
    match key {
        KeyCode::Char('q') => return true,

        KeyCode::Tab | KeyCode::Right
            if matches!(
                app.mode,
                Mode::Normal | Mode::Stats | Mode::RecurringManagement
            ) =>
        {
            app.next_tab();
            return false;
        }
        KeyCode::BackTab | KeyCode::Left
            if matches!(
                app.mode,
                Mode::Normal | Mode::Stats | Mode::RecurringManagement
            ) =>
        {
            app.prev_tab();
            return false;
        }
        _ => {}
    }

    match app.mode {
        Mode::Normal => handle_normal(app, key, conn),
        Mode::Adding => handle_form(app, key, conn),
        Mode::Stats => stats::handle_stats(app, key),

        // 👇 New popup mode
        Mode::Popup => handle_popup(app, key, conn),
        Mode::RecurringManagement => handle_recurring_management(app, key, conn),
        Mode::Filtering => handle_filter(app, key),
    }
}

//
// ---------------- POPUP MODE ----------------
//

fn handle_popup(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    match key {
        // Confirm action
        KeyCode::Char('y') => {
            if let Some(popup) = app.popup.clone() {
                if let PopupKind::Confirm { action, .. } = popup {
                    match action {
                        PopupAction::DeleteTransaction(id) => {
                            crate::db::delete_transaction(conn, id).unwrap();
                            app.refresh(conn);
                        }

                        PopupAction::Quit => {
                            return true;
                        }
                    }
                }
            }

            app.close_popup();
        }

        // Cancel popup
        KeyCode::Char('n') | KeyCode::Esc => {
            app.close_popup();
        }

        _ => {}
    }

    false
}

//
// ---------------- NORMAL MODE ----------------
//

fn handle_normal(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    let len = app.get_filtered_transactions().len();

    match key {
        KeyCode::Char('a') => {
            app.form.reset();
            app.editing = None;
            app.mode = Mode::Adding;
        }

        KeyCode::Char('f') => {
            app.mode = Mode::Filtering;
        }

        KeyCode::Char('c') => {
            if app.filter.active {
                app.filter.active = false;
                app.filter.month_year.clear();
                app.filter.tag_index = None;
                app.selected = 0;
            }
        }

        KeyCode::Up => {
            if app.selected > 0 {
                app.selected -= 1;
            }
        }

        KeyCode::Down => {
            let max_len = std::cmp::min(15, len);
            if app.selected + 1 < max_len {
                app.selected += 1;
            }
        }

        // ✅ Delete now opens confirmation popup
        KeyCode::Char('d') => {
            if let Some(tx) = app.selected_transaction() {
                app.open_confirm_popup(
                    "Confirm Delete",
                    format!(
                        "Delete this transaction?\n\n{}  ({}{})",
                        tx.source, app.currency, tx.amount
                    ),
                    PopupAction::DeleteTransaction(tx.id),
                );
            }
        }

        KeyCode::Char('e') => {
            // Begin editing the currently selected transaction
            app.begin_edit_selected();
        }

        _ => {}
    }

    false
}

//
// ---------------- FORM MODE ----------------
//

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

        KeyCode::BackTab => {
            app.form.active = app.form.active.back();
        }

        // Arrow keys toggle Kind, cycle Tags, or toggle Recurring depending on active field
        KeyCode::Right => match app.form.active {
            crate::form::Field::Kind => app.form.toggle_kind(),
            crate::form::Field::Tag => app.form.next_tag(app.tags.len()),
            crate::form::Field::Recurring => app.form.toggle_recurring(),
            crate::form::Field::RecurringInterval => app.form.next_interval(),
            _ => {}
        },

        KeyCode::Left => match app.form.active {
            crate::form::Field::Kind => app.form.toggle_kind(),
            crate::form::Field::Tag => app.form.prev_tag(app.tags.len()),
            crate::form::Field::Recurring => app.form.toggle_recurring(),
            crate::form::Field::RecurringInterval => app.form.prev_interval(),
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

//
// ---------------- RECURRING MANAGEMENT MODE ----------------
//

fn handle_recurring_management(app: &mut App, key: KeyCode, conn: &Connection) -> bool {
    let len = app.recurring_entries.len();

    match key {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }

        KeyCode::Up => {
            if app.selected_recurring > 0 {
                app.selected_recurring -= 1;
            }
        }

        KeyCode::Down => {
            if app.selected_recurring + 1 < len {
                app.selected_recurring += 1;
            }
        }

        KeyCode::Char(' ') => {
            // Toggle active/inactive for selected recurring entry
            if !app.recurring_entries.is_empty() {
                let entry = &app.recurring_entries[app.selected_recurring];
                let new_active = !entry.active;
                crate::db::toggle_recurring_entry(conn, entry.id, new_active).unwrap();
                app.refresh(conn);
            }
        }

        KeyCode::Char('d') => {
            // Delete selected recurring entry
            if !app.recurring_entries.is_empty() {
                let entry = &app.recurring_entries[app.selected_recurring];
                crate::db::delete_recurring_entry(conn, entry.id).unwrap();
                app.refresh(conn);

                // Clamp selection if needed
                if app.selected_recurring >= app.recurring_entries.len()
                    && app.selected_recurring > 0
                {
                    app.selected_recurring -= 1;
                }
            }
        }

        _ => {}
    }

    false
}

fn handle_filter(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Tab => {
            app.filter.active_field = app.filter.active_field.next();
        }
        KeyCode::BackTab => {
            app.filter.active_field = app.filter.active_field.back();
        }
        KeyCode::Right => {
            if matches!(app.filter.active_field, crate::app::FilterField::Tag) {
                app.filter.tag_index = match app.filter.tag_index {
                    None => Some(0),
                    Some(idx) => {
                        if idx + 1 < app.tags.len() {
                            Some(idx + 1)
                        } else {
                            None
                        }
                    }
                };
            }
        }
        KeyCode::Left => {
            if matches!(app.filter.active_field, crate::app::FilterField::Tag) {
                app.filter.tag_index = match app.filter.tag_index {
                    None => Some(app.tags.len() - 1),
                    Some(idx) => {
                        if idx > 0 {
                            Some(idx - 1)
                        } else {
                            None
                        }
                    }
                };
            }
        }
        KeyCode::Backspace => {
            if matches!(app.filter.active_field, crate::app::FilterField::MonthYear) {
                app.filter.month_year.pop();
            }
        }
        KeyCode::Char(c) => {
            if matches!(app.filter.active_field, crate::app::FilterField::MonthYear) {
                if c.is_ascii_digit() || c == '-' {
                    if app.filter.month_year.len() < 7 {
                        app.filter.month_year.push(c);
                    }
                }
            }
        }
        KeyCode::Enter => {
            app.filter.active = !app.filter.month_year.is_empty() || app.filter.tag_index.is_some();
            app.selected = 0;
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    false
}
