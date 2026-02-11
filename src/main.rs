use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::prelude::*;

mod db;
mod models;
mod ui;

use models::{Tag, TransactionType};

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Adding,
}

pub struct App {
    pub mode: Mode,

    pub source: String,
    pub amount: String,
    pub kind: String,
    pub tag: String,
    pub date: String,

    pub field_index: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,

            source: String::new(),
            amount: String::new(),
            kind: "debit".into(),
            tag: "other".into(),
            date: "2026-02-11".into(),

            field_index: 0,
        }
    }
}

fn main() -> io::Result<()> {
    let conn = db::init_db().expect("Failed to initialize database");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let result = run_app(&mut terminal, &conn, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    conn: &rusqlite::Connection,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Load DB data
        let transactions = db::get_transactions(conn).unwrap();
        let earned = db::total_earned(conn).unwrap();
        let spent = db::total_spent(conn).unwrap();
        let balance = earned - spent;

        // Draw UI
        terminal.draw(|f| {
            ui::draw_ui(f, &transactions, earned, spent, balance, app);
        })?;

        // Input handling
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),

                        KeyCode::Char('a') => {
                            app.mode = Mode::Adding;
                        }

                        _ => {}
                    },

                    Mode::Adding => match key.code {
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                        }

                        KeyCode::Tab => {
                            app.field_index = (app.field_index + 1) % 5;
                        }

                        KeyCode::Backspace => {
                            active_field_pop(app);
                        }

                        KeyCode::Char(c) => {
                            active_field_push(app, c);
                        }

                        KeyCode::Enter => {
                            save_transaction(conn, app);

                            // Reset form
                            *app = App::new();
                        }

                        _ => {}
                    },
                }
            }
        }
    }
}

// ----------------------------
// Field Editing Helpers
// ----------------------------

fn active_field_push(app: &mut App, c: char) {
    match app.field_index {
        0 => app.source.push(c),
        1 => app.amount.push(c),
        2 => app.kind.push(c),
        3 => app.tag.push(c),
        4 => app.date.push(c),
        _ => {}
    }
}

fn active_field_pop(app: &mut App) {
    match app.field_index {
        0 => {
            app.source.pop();
        }
        1 => {
            app.amount.pop();
        }
        2 => {
            app.kind.pop();
        }
        3 => {
            app.tag.pop();
        }
        4 => {
            app.date.pop();
        }
        _ => {}
    }
}

// ----------------------------
// Save Transaction
// ----------------------------

fn save_transaction(conn: &rusqlite::Connection, app: &mut App) {
    let amount: f64 = app.amount.trim().parse().unwrap_or(0.0);

    let kind = if app.kind.trim() == "credit" {
        TransactionType::Credit
    } else {
        TransactionType::Debit
    };

    let tag = Tag::from_str(app.tag.trim());

    db::add_transaction(
        conn,
        &app.source,
        amount,
        kind,
        tag,
        &app.date,
    )
    .unwrap();

    app.mode = Mode::Normal;
}
