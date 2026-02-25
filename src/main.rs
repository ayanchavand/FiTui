mod app;
mod db;
mod form;
mod handlers;
mod models;
mod stats;
mod theme;
mod ui;
mod config;

use std::io;

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use ratatui::prelude::*;

use app::App;

fn main() -> io::Result<()> {
    let cfg = config::load_config();
    let conn = db::init_db(&cfg);

    // Insert recurring entries based on their intervals
    db::insert_recurring_transactions(&conn).unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&conn);

    loop {
        let snapshot = stats::StatsSnapshot::new(&app.transactions);

        terminal.draw(|f| {
            ui::draw_ui(f, &app, &snapshot);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let quit = handlers::handle_key(&mut app, key.code, &conn);

                    if quit {
                        break;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
