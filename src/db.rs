use rusqlite::{Connection, Result};
use std::collections::HashMap;
use directories::ProjectDirs;
use std::fs;

use crate::models::{Tag, Transaction, TransactionType};

pub fn init_db() -> Result<Connection> {
    // ✅ Get standard OS data directory
    let proj_dirs = ProjectDirs::from("com", "ayan", "fitui")
        .expect("Could not determine data directory");

    let data_dir = proj_dirs.data_dir();

    // ✅ Ensure directory exists
    fs::create_dir_all(data_dir).expect("Failed to create data directory");

    // ✅ Final database path
    let db_path = data_dir.join("budget.db");

    println!("Database location: {:?}", db_path);

    let conn = Connection::open(db_path)?;

    // Create table if missing
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

    Ok(conn)
}


// ============================
// Load Transactions
// ============================

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
            kind: TransactionType::from_str(&row.get::<_, String>(3)?),

            // ✅ Tag wrapper
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

// ============================
// Add Transaction
// ============================

pub fn add_transaction(
    conn: &Connection,
    source: &str,
    amount: f64,
    kind: TransactionType,
    tag: &Tag, // ✅ borrow tag, don’t move
    date: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO transactions (source, amount, kind, tag, date)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (source, amount, kind.as_str(), tag.as_str(), date),
    )?;

    Ok(())
}

// ============================
// Delete Transaction
// ============================

pub fn delete_transaction(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
    Ok(())
}

// ============================
// Totals
// ============================

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

// ============================
// Stats: Spending Per Tag
// ============================

pub fn spent_per_tag(conn: &Connection) -> Result<HashMap<Tag, f64>> {
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
