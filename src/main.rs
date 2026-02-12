mod app;
mod db;
mod form;
mod handlers;
mod models;
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
    let conn = db::init_db().unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&conn);

    loop {
        let earned = db::total_earned(&conn).unwrap();
        let spent = db::total_spent(&conn).unwrap();
        let balance = earned - spent;

        let per_tag = db::spent_per_tag(&conn).unwrap();

        let tx_count = app.transactions.len();

        let largest = app
            .transactions
            .iter()
            .max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal))
            .cloned();

        let smallest = app
            .transactions
            .iter()
            .min_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal))
            .cloned();

        let mut top_tags: Vec<(crate::models::Tag, f64)> =
            per_tag.iter().map(|(t, v)| (t.clone(), *v)).collect();

        top_tags.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        use std::collections::BTreeMap;
        let mut monthly_map: BTreeMap<String, (f64, f64)> = BTreeMap::new();

        for tx in &app.transactions {
            let month = if tx.date.len() >= 7 {
                tx.date[..7].to_string()
            } else {
                tx.date.clone()
            };

            let entry = monthly_map.entry(month).or_insert((0.0, 0.0));

            match tx.kind {
                crate::models::TransactionType::Credit => entry.0 += tx.amount,
                crate::models::TransactionType::Debit => entry.1 += tx.amount,
            }
        }

        let monthly_history: Vec<(String, f64, f64)> = monthly_map
            .into_iter()
            .rev()
            .take(6)
            .map(|(m, (e, s))| (m, e, s))
            .collect();

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
