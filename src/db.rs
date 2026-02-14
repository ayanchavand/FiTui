use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::fs;

use directories::ProjectDirs;

use crate::models::{RecurringEntry, Tag, Transaction, TransactionType};

pub fn init_db() -> Result<Connection> {
    // Store DB in the OS-standard application data directory
    let proj_dirs =
        ProjectDirs::from("com", "ayan", "fitui").expect("Could not determine data directory");

    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir).expect("Failed to create data directory");

    let db_path = data_dir.join("budget.db");
    
    #[cfg(debug_assertions)]
    println!("Database location: {:?}", db_path);

    let conn = Connection::open(db_path)?;

    // Create schema on first run if it doesn't exist yet
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            amount REAL NOT NULL,
            kind TEXT NOT NULL,
            tag TEXT NOT NULL,
            date TEXT NOT NULL
        )",
        [],
    )?;

    // Create recurring entries table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recurring_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            amount REAL NOT NULL,
            kind TEXT NOT NULL,
            tag TEXT NOT NULL,
            last_inserted_month TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )?;

    Ok(conn)
}

pub fn get_transactions(conn: &Connection) -> Result<Vec<Transaction>> {
    let mut stmt = conn.prepare(
        "SELECT id, source, amount, kind, tag, date
         FROM transactions
         ORDER BY date DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Transaction {
            id: row.get(0)?,
            source: row.get(1)?,
            amount: row.get(2)?,

            // Stored as string in DB, converted back into enum
            kind: TransactionType::from_str(&row.get::<_, String>(3)?),

            // Tags are wrapped in your custom Tag type
            tag: Tag::from_str(&row.get::<_, String>(4)?),

            date: row.get(5)?,
        })
    })?;

    let mut transactions = Vec::new();
    for tx in rows {
        transactions.push(tx?);
    }

    Ok(transactions)
}

pub fn add_transaction(
    conn: &Connection,
    source: &str,
    amount: f64,
    kind: TransactionType,
    tag: &Tag,
    date: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO transactions (source, amount, kind, tag, date)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (source, amount, kind.as_str(), tag.as_str(), date),
    )?;

    Ok(())
}

pub fn delete_transaction(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
    Ok(())
}

pub fn update_transaction(
    conn: &Connection,
    id: i32,
    source: &str,
    amount: f64,
    kind: TransactionType,
    tag: &Tag,
    date: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE transactions SET source = ?1, amount = ?2, kind = ?3, tag = ?4, date = ?5 WHERE id = ?6",
        (source, amount, kind.as_str(), tag.as_str(), date, id),
    )?;

    Ok(())
}

pub fn total_earned(conn: &Connection) -> Result<f64> {
    conn.query_row(
        "SELECT COALESCE(SUM(amount), 0)
         FROM transactions
         WHERE kind = 'credit'",
        [],
        |row| row.get(0),
    )
}

pub fn total_spent(conn: &Connection) -> Result<f64> {
    conn.query_row(
        "SELECT COALESCE(SUM(amount), 0)
         FROM transactions
         WHERE kind = 'debit'",
        [],
        |row| row.get(0),
    )
}

pub fn spent_per_tag(conn: &Connection) -> Result<HashMap<Tag, f64>> {
    // Aggregate total spending grouped by tag
    let mut stmt = conn.prepare(
        "SELECT tag, COALESCE(SUM(amount), 0)
         FROM transactions
         WHERE kind = 'debit'
         GROUP BY tag",
    )?;

    let rows = stmt.query_map([], |row| {
        let tag_str: String = row.get(0)?;
        let total: f64 = row.get(1)?;

        Ok((Tag::from_str(&tag_str), total))
    })?;

    let mut map = HashMap::new();
    for r in rows {
        let (tag, total) = r?;
        map.insert(tag, total);
    }

    Ok(map)
}
// Recurring entry functions
pub fn get_recurring_entries(conn: &Connection) -> Result<Vec<RecurringEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, source, amount, kind, tag, last_inserted_month, active
         FROM recurring_entries
         ORDER BY id DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(RecurringEntry {
            id: row.get(0)?,
            source: row.get(1)?,
            amount: row.get(2)?,
            kind: TransactionType::from_str(&row.get::<_, String>(3)?),
            tag: Tag::from_str(&row.get::<_, String>(4)?),
            last_inserted_month: row.get(5)?,
            active: row.get::<_, i32>(6)? != 0,
        })
    })?;

    let mut entries = Vec::new();
    for entry in rows {
        entries.push(entry?);
    }

    Ok(entries)
}

pub fn add_recurring_entry(
    conn: &Connection,
    source: &str,
    amount: f64,
    kind: TransactionType,
    tag: &Tag,
) -> Result<()> {
    conn.execute(
        "INSERT INTO recurring_entries (source, amount, kind, tag, last_inserted_month, active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            source,
            amount,
            kind.as_str(),
            tag.as_str(),
            "", // Empty string indicates it hasn't been inserted yet
            1,
        ),
    )?;

    Ok(())
}

pub fn delete_recurring_entry(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM recurring_entries WHERE id = ?1", [id])?;
    Ok(())
}

pub fn toggle_recurring_entry(conn: &Connection, id: i32, active: bool) -> Result<()> {
    conn.execute(
        "UPDATE recurring_entries SET active = ?1 WHERE id = ?2",
        (if active { 1 } else { 0 }, id),
    )?;
    Ok(())
}

// Auto-insert recurring entries for the current month
pub fn insert_recurring_for_month(conn: &Connection, current_month: &str) -> Result<()> {
    // Get all active recurring entries that haven't been inserted this month
    let mut stmt = conn.prepare(
        "SELECT id, source, amount, kind, tag FROM recurring_entries
         WHERE active = 1 AND last_inserted_month != ?1",
    )?;

    let entries: Vec<_> = stmt
        .query_map([current_month], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Insert each recurring entry as a transaction for this month
    for (rec_id, source, amount, kind, tag) in entries {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let kind_enum = TransactionType::from_str(&kind);
        let tag_obj = Tag::from_str(&tag);

        add_transaction(conn, &source, amount, kind_enum, &tag_obj, &today)?;

        // Update the last_inserted_month
        conn.execute(
            "UPDATE recurring_entries SET last_inserted_month = ?1 WHERE id = ?2",
            (current_month, rec_id),
        )?;
    }

    Ok(())
}