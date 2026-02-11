use std::io::{self, Write};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

mod db;
mod models;

use models::{Tag, TransactionType};

fn main() -> io::Result<()> {
    // ----------------------------
    // Database startup
    // ----------------------------
    let conn = db::init_db().expect("Failed to initialize database");

    // ----------------------------
    // Terminal setup
    // ----------------------------
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ----------------------------
    // Main UI loop
    // ----------------------------
    loop {
        let transactions = db::get_transactions(&conn).unwrap();
        let earned = db::total_earned(&conn).unwrap();
        let spent = db::total_spent(&conn).unwrap();
        let balance = earned - spent;

        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(size);

            // Header
            let header = Paragraph::new(format!(
                " Earned: ₹{:.2}   Spent: ₹{:.2}   Balance: ₹{:.2} ",
                earned, spent, balance
            ))
            .block(Block::default().title("📊 Stats").borders(Borders::ALL))
            .alignment(Alignment::Center);

            f.render_widget(header, chunks[0]);

            // Transactions
            let items: Vec<ListItem> = transactions
                .iter()
                .map(|tx| {
                    ListItem::new(format!(
                        "{} | {} | ₹{:.2} | {} | {}",
                        tx.date,
                        tx.source,
                        tx.amount,
                        tx.kind.as_str(),
                        tx.tag.as_str()
                    ))
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title("💰 Transactions (a = add, q = quit)")
                    .borders(Borders::ALL),
            );

            f.render_widget(list, chunks[1]);
        })?;

        // Input
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') => {
                        add_transaction_prompt(&conn)?;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

// --------------------------------------------------
// Prompt-based add transaction
// --------------------------------------------------
fn add_transaction_prompt(conn: &rusqlite::Connection) -> io::Result<()> {
    // Leave TUI mode
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let mut input = String::new();

    print!("Source: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let source = input.trim().to_string();

    input.clear();
    print!("Amount: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let amount: f64 = input.trim().parse().unwrap_or(0.0);

    input.clear();
    print!("Type (credit/debit): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let kind = match input.trim() {
        "credit" => TransactionType::Credit,
        _ => TransactionType::Debit,
    };

    input.clear();
    print!("Tag (food/travel/shopping/bills/salary/other): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let tag = Tag::from_str(input.trim());

    input.clear();
    print!("Date (YYYY-MM-DD): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let date = input.trim().to_string();

    db::add_transaction(
        conn,
        &source,
        amount,
        kind,
        tag,
        &date,
    )
    .expect("Failed to insert transaction");

    println!("✔ Transaction added! Press Enter to continue...");
    input.clear();
    io::stdin().read_line(&mut input)?;

    // Re-enter TUI mode
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    Ok(())
}
