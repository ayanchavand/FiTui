use rusqlite::{Connection, Result};

use crate::models::{Transaction, TransactionType, Tag};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("budget.db")?;

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
    tag: Tag,
    date: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO transactions (source, amount, kind, tag, date)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            source,
            amount,
            kind.as_str(),
            tag.as_str(),
            date,
        ),
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

pub fn spent_per_tag(conn: &Connection) -> Result<Vec<(String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT tag, COALESCE(SUM(amount), 0)
         FROM transactions
         WHERE kind = 'debit'
         GROUP BY tag",
    )?;

    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }

    Ok(result)
}
