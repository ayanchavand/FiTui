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
use chrono::Datelike;

use app::App;

fn main() -> io::Result<()> {
    let conn = db::init_db().unwrap();

    // Insert recurring entries for the current month on startup
    let now = chrono::Local::now();
    let current_month = format!("{:04}-{:02}", now.year(), now.month());
    db::insert_recurring_for_month(&conn, &current_month).unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&conn);

    loop {
        let earned = stats::calculate_earned(&app.transactions);
        let spent = stats::calculate_spent(&app.transactions);
        let balance = earned - spent;

        let per_tag = stats::calculate_spent_per_tag(&app.transactions);

        let tx_count = app.transactions.len();

        let largest = stats::get_largest_transaction(&app.transactions);

        let smallest = stats::get_smallest_transaction(&app.transactions);

        let top_tags = stats::get_top_tags(&per_tag);

        let monthly_history = stats::calculate_monthly_history(&app.transactions);

        terminal.draw(|f| {
            ui::draw_ui(
                f,
                &app.transactions,
                earned,
                spent,
                balance,
                &per_tag,
                &monthly_history,
                tx_count,
                largest.clone(),
                smallest.clone(),
                &top_tags,
                &app,
            );
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
