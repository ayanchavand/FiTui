use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::fs;
use chrono::Datelike;

use directories::ProjectDirs;

use crate::models::{RecurringEntry, RecurringInterval, Tag, Transaction, TransactionType};

/// Initialize the database from a provided path. Useful for tests (`:memory:`) or custom locations.
pub fn init_db_with_path(path: &std::path::Path) -> Result<Connection> {
    println!("Database location: {:?}", path);

    let conn = Connection::open(path)?;

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
            interval TEXT NOT NULL DEFAULT 'monthly',
            original_date TEXT NOT NULL,
            last_inserted_date TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )?;

    // Migrate existing recurring_entries table if it has old schema
    migrate_recurring_entries_schema(&conn)?;

    Ok(conn)
}

/// Initialize DB in-memory for tests.
pub fn init_in_memory() -> Result<Connection> {
    init_db_with_path(std::path::Path::new(":memory:"))
}

pub fn init_db() -> Result<Connection> {
    let db_path = if cfg!(debug_assertions) {
        // Debug build: store DB locally inside the project folder
        let local_dir = std::path::Path::new("./data");
        fs::create_dir_all(local_dir).expect("Failed to create local debug data directory");

        local_dir.join("budget.db")
    } else {
        // Release build: store DB in OS-standard application data directory
        let proj_dirs =
            ProjectDirs::from("com", "ayan", "fitui")
                .expect("Could not determine data directory");

        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).expect("Failed to create data directory");

        data_dir.join("budget.db")
    };

    init_db_with_path(&db_path)
}

/// Migrate old recurring_entries table to new schema with interval and original_date columns
pub fn migrate_recurring_entries_schema(conn: &Connection) -> Result<()> {
    // First, check if the old last_inserted_month column exists
    let has_old_column = conn
        .prepare("SELECT last_inserted_month FROM recurring_entries LIMIT 1")
        .map(|_| true)
        .unwrap_or(false);

    // Check if the new last_inserted_date column exists
    let has_last_inserted_date = conn
        .prepare("SELECT last_inserted_date FROM recurring_entries LIMIT 1")
        .map(|_| true)
        .unwrap_or(false);

    // If old column exists but new one doesn't, add the new column and migrate data
    if has_old_column && !has_last_inserted_date {
        let _ = conn.execute(
            "ALTER TABLE recurring_entries ADD COLUMN last_inserted_date TEXT NOT NULL DEFAULT ''",
            [],
        );

        // Migrate last_inserted_month data to last_inserted_date
        let _ = conn.execute(
            "UPDATE recurring_entries SET last_inserted_date = COALESCE(last_inserted_month, '') WHERE 1=1",
            [],
        );
    } else if !has_last_inserted_date {
        // Only add the column if neither old nor new exists
        let _ = conn.execute(
            "ALTER TABLE recurring_entries ADD COLUMN last_inserted_date TEXT NOT NULL DEFAULT ''",
            [],
        );
    }

    // Check and add interval column if missing
    let has_interval = conn
        .prepare("SELECT interval FROM recurring_entries LIMIT 1")
        .map(|_| true)
        .unwrap_or(false);

    if !has_interval {
        let _ = conn.execute(
            "ALTER TABLE recurring_entries ADD COLUMN interval TEXT NOT NULL DEFAULT 'monthly'",
            [],
        );
    }

    // Check and add original_date column if missing
    let has_original_date = conn
        .prepare("SELECT original_date FROM recurring_entries LIMIT 1")
        .map(|_| true)
        .unwrap_or(false);

    if !has_original_date {
        let _ = conn.execute(
            "ALTER TABLE recurring_entries ADD COLUMN original_date TEXT NOT NULL DEFAULT ''",
            [],
        );
    }

    // Check and add active column if missing
    let has_active = conn
        .prepare("SELECT active FROM recurring_entries LIMIT 1")
        .map(|_| true)
        .unwrap_or(false);

    if !has_active {
        let _ = conn.execute(
            "ALTER TABLE recurring_entries ADD COLUMN active INTEGER NOT NULL DEFAULT 1",
            [],
        );
    }

    Ok(())
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
        "SELECT id, source, amount, kind, tag, interval, original_date, last_inserted_date, active
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
            interval: RecurringInterval::from_str(&row.get::<_, String>(5)?),
            original_date: row.get(6)?,
            last_inserted_date: row.get(7)?,
            active: row.get::<_, i32>(8)? != 0,
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
    interval: &RecurringInterval,
    original_date: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO recurring_entries (source, amount, kind, tag, interval, original_date, last_inserted_date, active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            source,
            amount,
            kind.as_str(),
            tag.as_str(),
            interval.as_str(),
            original_date,
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

// Auto-insert recurring entries based on their interval
pub fn insert_recurring_transactions(conn: &Connection) -> Result<()> {
    let now = chrono::Local::now();
    let today_str = now.format("%Y-%m-%d").to_string();
    let current_week = format!("{:04}-W{:02}", now.year(), now.iso_week().week());
    let current_month = format!("{:04}-{:02}", now.year(), now.month());

    // Get all active recurring entries
    let mut stmt = conn.prepare(
        "SELECT id, source, amount, kind, tag, interval, original_date, last_inserted_date
         FROM recurring_entries
         WHERE active = 1",
    )?;

    let entries: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Process each recurring entry
    for (rec_id, source, amount, kind, tag, interval_str, original_date, last_inserted_date) in entries {
        let interval = RecurringInterval::from_str(&interval_str);
        let kind_enum = TransactionType::from_str(&kind);
        let tag_obj = Tag::from_str(&tag);

        let should_insert = match interval {
            RecurringInterval::Daily => {
                // Insert if we haven't inserted today
                last_inserted_date != today_str
            }
            RecurringInterval::Weekly => {
                // Extract day of week from original date
                if let Ok(original_ndt) = chrono::NaiveDate::parse_from_str(&original_date, "%Y-%m-%d") {
                    let original_dow = original_ndt.weekday();
                    let today_dow = now.weekday();
                    
                    // Check if this is the same day of the week and hasn't been inserted this week
                    original_dow == today_dow && last_inserted_date != current_week
                } else {
                    false
                }
            }
            RecurringInterval::Monthly => {
                // Extract day of month from original date
                if let Ok(original_ndt) = chrono::NaiveDate::parse_from_str(&original_date, "%Y-%m-%d") {
                    let original_day = original_ndt.day();
                    let today_day = now.day();
                    
                    // Check if this is the same day of month and hasn't been inserted this month
                    original_day == today_day && last_inserted_date != current_month
                } else {
                    false
                }
            }
        };

        if should_insert {
            // Insert as a transaction with today's date
            add_transaction(conn, &source, amount, kind_enum, &tag_obj, &today_str)?;

            // Update the last_inserted_date based on interval
            let new_last_inserted = match interval {
                RecurringInterval::Daily => today_str.clone(),
                RecurringInterval::Weekly => current_week.clone(),
                RecurringInterval::Monthly => current_month.clone(),
            };

            conn.execute(
                "UPDATE recurring_entries SET last_inserted_date = ?1 WHERE id = ?2",
                (new_last_inserted, rec_id),
            )?;
        }
    }

    Ok(())
}


// Keep the old function name for backwards compatibility
pub fn insert_recurring_for_month(conn: &Connection, _current_month: &str) -> Result<()> {
    insert_recurring_transactions(conn)
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::{RecurringInterval, Tag, TransactionType};

    fn setup_conn() -> Connection {
        init_in_memory().expect("failed to init in-memory db")
    }

    #[test]
    fn totals_are_calculated() {
        let conn = setup_conn();

        add_transaction(&conn, "pay", 100.0, TransactionType::Credit, &Tag::from_str("salary"), "2026-02-23").unwrap();
        add_transaction(&conn, "buy", 40.0, TransactionType::Debit, &Tag::from_str("food"), "2026-02-23").unwrap();

        let earned = total_earned(&conn).unwrap();
        let spent = total_spent(&conn).unwrap();

        assert_eq!(earned, 100.0);
        assert_eq!(spent, 40.0);

        let per_tag = spent_per_tag(&conn).unwrap();
        assert_eq!(per_tag.get(&Tag::from_str("food")).copied().unwrap_or(0.0), 40.0);
    }

    #[test]
    fn recurring_roundtrip() {
        let conn = setup_conn();

        add_recurring_entry(&conn, "rent", 500.0, TransactionType::Debit, &Tag::from_str("housing"), &RecurringInterval::Monthly, "2026-02-23").unwrap();

        let entries = get_recurring_entries(&conn).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].source, "rent");
        assert_eq!(entries[0].amount, 500.0);
        assert_eq!(entries[0].interval, RecurringInterval::Monthly);
    }
}