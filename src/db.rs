use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::Datelike;
use directories::ProjectDirs;

use crate::config::{Config, DbSource};
use crate::models::{RecurringEntry, RecurringInterval, Tag, Transaction, TransactionType};

// ── Public handle ────────────────────────────────────────────────────────────

pub struct DbHandle {
    rt: tokio::runtime::Runtime,
    conn: libsql::Connection,
    _db: libsql::Database,
}

impl DbHandle {
    /// Raw connection access (used by integration tests for migration testing).
    pub fn conn(&self) -> &libsql::Connection {
        &self.conn
    }

    /// Runtime access (used by integration tests for migration testing).
    pub fn rt(&self) -> &tokio::runtime::Runtime {
        &self.rt
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn get_local_db_path() -> PathBuf {
    if cfg!(debug_assertions) {
        let local_dir = std::path::Path::new("./data");
        fs::create_dir_all(local_dir).expect("Failed to create local debug data directory");
        local_dir.join("budget.db")
    } else {
        let proj_dirs = ProjectDirs::from("com", "ayan", "fitui")
            .expect("Could not determine data directory");
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).expect("Failed to create data directory");
        data_dir.join("budget.db")
    }
}

async fn create_schema(conn: &libsql::Connection) -> libsql::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            amount REAL NOT NULL,
            kind TEXT NOT NULL,
            tag TEXT NOT NULL,
            date TEXT NOT NULL
        )",
        (),
    )
    .await?;

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
        (),
    )
    .await?;

    migrate_recurring_entries_schema_async(conn).await?;

    Ok(())
}

async fn migrate_recurring_entries_schema_async(conn: &libsql::Connection) -> libsql::Result<()> {
    let has_old_column = conn
        .prepare("SELECT last_inserted_month FROM recurring_entries LIMIT 1")
        .await
        .map(|_| true)
        .unwrap_or(false);

    let has_last_inserted_date = conn
        .prepare("SELECT last_inserted_date FROM recurring_entries LIMIT 1")
        .await
        .map(|_| true)
        .unwrap_or(false);

    if has_old_column && !has_last_inserted_date {
        let _ = conn
            .execute(
                "ALTER TABLE recurring_entries ADD COLUMN last_inserted_date TEXT NOT NULL DEFAULT ''",
                (),
            )
            .await;
        let _ = conn
            .execute(
                "UPDATE recurring_entries SET last_inserted_date = COALESCE(last_inserted_month, '') WHERE 1=1",
                (),
            )
            .await;
    } else if !has_last_inserted_date {
        let _ = conn
            .execute(
                "ALTER TABLE recurring_entries ADD COLUMN last_inserted_date TEXT NOT NULL DEFAULT ''",
                (),
            )
            .await;
    }

    let has_interval = conn
        .prepare("SELECT interval FROM recurring_entries LIMIT 1")
        .await
        .map(|_| true)
        .unwrap_or(false);

    if !has_interval {
        let _ = conn
            .execute(
                "ALTER TABLE recurring_entries ADD COLUMN interval TEXT NOT NULL DEFAULT 'monthly'",
                (),
            )
            .await;
    }

    let has_original_date = conn
        .prepare("SELECT original_date FROM recurring_entries LIMIT 1")
        .await
        .map(|_| true)
        .unwrap_or(false);

    if !has_original_date {
        let _ = conn
            .execute(
                "ALTER TABLE recurring_entries ADD COLUMN original_date TEXT NOT NULL DEFAULT ''",
                (),
            )
            .await;
    }

    let has_active = conn
        .prepare("SELECT active FROM recurring_entries LIMIT 1")
        .await
        .map(|_| true)
        .unwrap_or(false);

    if !has_active {
        let _ = conn
            .execute(
                "ALTER TABLE recurring_entries ADD COLUMN active INTEGER NOT NULL DEFAULT 1",
                (),
            )
            .await;
    }

    Ok(())
}

fn finish_handle(rt: tokio::runtime::Runtime, db: libsql::Database) -> DbHandle {
    let conn = db.connect().expect("Failed to create database connection");
    let handle = DbHandle { rt, conn, _db: db };
    let conn = &handle.conn;
    handle
        .rt
        .block_on(create_schema(conn))
        .expect("Failed to create schema");
    handle
}

// ── Public init functions ────────────────────────────────────────────────────

pub fn init_db(config: &Config) -> DbHandle {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    let db = match &config.db_source {
        DbSource::Local => {
            let path = get_local_db_path();
            println!("Database location: {:?}", path);
            rt.block_on(libsql::Builder::new_local(path).build())
                .expect("Failed to open local database")
        }
        DbSource::Turso => {
            let url = config
                .turso_url
                .as_deref()
                .expect("turso_url is required when db_source is 'turso'");
            let token = config
                .turso_token
                .as_deref()
                .expect("turso_token is required when db_source is 'turso'");
            println!("Connecting to Turso: {}", url);
            rt.block_on(
                libsql::Builder::new_remote(url.to_string(), token.to_string()).build(),
            )
            .expect("Failed to connect to Turso database")
        }
    };

    finish_handle(rt, db)
}

/// In-memory database with full schema. Used by unit/integration tests.
pub fn init_in_memory() -> libsql::Result<DbHandle> {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let db = rt.block_on(libsql::Builder::new_local(":memory:").build())?;
    let conn = db.connect()?;
    let handle = DbHandle { rt, conn, _db: db };
    let conn = &handle.conn;
    handle.rt.block_on(create_schema(conn))?;
    Ok(handle)
}

/// In-memory database with NO schema. Used by migration tests to set up old schema manually.
pub fn init_in_memory_empty() -> libsql::Result<DbHandle> {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let db = rt.block_on(libsql::Builder::new_local(":memory:").build())?;
    let conn = db.connect()?;
    Ok(DbHandle { rt, conn, _db: db })
}

// ── Schema migration (public for integration tests) ───────────────────────────

pub fn migrate_recurring_entries_schema(db: &DbHandle) -> libsql::Result<()> {
    let conn = &db.conn;
    db.rt.block_on(migrate_recurring_entries_schema_async(conn))
}

// ── Transaction functions ─────────────────────────────────────────────────────

pub fn get_transactions(db: &DbHandle) -> libsql::Result<Vec<Transaction>> {
    let conn = &db.conn;
    db.rt.block_on(async {
        let mut rows = conn
            .query(
                "SELECT id, source, amount, kind, tag, date
                 FROM transactions
                 ORDER BY date DESC",
                (),
            )
            .await?;

        let mut transactions = Vec::new();
        while let Some(row) = rows.next().await? {
            transactions.push(Transaction {
                id: row.get(0)?,
                source: row.get(1)?,
                amount: row.get(2)?,
                kind: TransactionType::from_str(&row.get::<String>(3)?),
                tag: Tag::from_str(&row.get::<String>(4)?),
                date: row.get(5)?,
            });
        }

        Ok(transactions)
    })
}

pub fn add_transaction(
    db: &DbHandle,
    source: &str,
    amount: f64,
    kind: TransactionType,
    tag: &Tag,
    date: &str,
) -> libsql::Result<()> {
    let conn = &db.conn;
    let source = source.to_string();
    let kind_str = kind.as_str().to_string();
    let tag_str = tag.as_str().to_string();
    let date = date.to_string();

    db.rt.block_on(async {
        conn.execute(
            "INSERT INTO transactions (source, amount, kind, tag, date)
             VALUES (?, ?, ?, ?, ?)",
            libsql::params![source, amount, kind_str, tag_str, date],
        )
        .await?;
        Ok(())
    })
}

pub fn delete_transaction(db: &DbHandle, id: i32) -> libsql::Result<()> {
    let conn = &db.conn;
    db.rt.block_on(async {
        conn.execute("DELETE FROM transactions WHERE id = ?", libsql::params![id])
            .await?;
        Ok(())
    })
}

pub fn update_transaction(
    db: &DbHandle,
    id: i32,
    source: &str,
    amount: f64,
    kind: TransactionType,
    tag: &Tag,
    date: &str,
) -> libsql::Result<()> {
    let conn = &db.conn;
    let source = source.to_string();
    let kind_str = kind.as_str().to_string();
    let tag_str = tag.as_str().to_string();
    let date = date.to_string();

    db.rt.block_on(async {
        conn.execute(
            "UPDATE transactions SET source = ?, amount = ?, kind = ?, tag = ?, date = ? WHERE id = ?",
            libsql::params![source, amount, kind_str, tag_str, date, id],
        )
        .await?;
        Ok(())
    })
}

pub fn total_earned(db: &DbHandle) -> libsql::Result<f64> {
    let conn = &db.conn;
    db.rt.block_on(async {
        let mut rows = conn
            .query(
                "SELECT COALESCE(SUM(amount), 0)
                 FROM transactions
                 WHERE kind = 'credit'",
                (),
            )
            .await?;
        if let Some(row) = rows.next().await? {
            row.get(0)
        } else {
            Ok(0.0)
        }
    })
}

pub fn total_spent(db: &DbHandle) -> libsql::Result<f64> {
    let conn = &db.conn;
    db.rt.block_on(async {
        let mut rows = conn
            .query(
                "SELECT COALESCE(SUM(amount), 0)
                 FROM transactions
                 WHERE kind = 'debit'",
                (),
            )
            .await?;
        if let Some(row) = rows.next().await? {
            row.get(0)
        } else {
            Ok(0.0)
        }
    })
}

pub fn spent_per_tag(db: &DbHandle) -> libsql::Result<HashMap<Tag, f64>> {
    let conn = &db.conn;
    db.rt.block_on(async {
        let mut rows = conn
            .query(
                "SELECT tag, COALESCE(SUM(amount), 0)
                 FROM transactions
                 WHERE kind = 'debit'
                 GROUP BY tag",
                (),
            )
            .await?;

        let mut map = HashMap::new();
        while let Some(row) = rows.next().await? {
            let tag_str: String = row.get(0)?;
            let total: f64 = row.get(1)?;
            map.insert(Tag::from_str(&tag_str), total);
        }

        Ok(map)
    })
}

// ── Recurring entry functions ─────────────────────────────────────────────────

pub fn get_recurring_entries(db: &DbHandle) -> libsql::Result<Vec<RecurringEntry>> {
    let conn = &db.conn;
    db.rt.block_on(async {
        let mut rows = conn
            .query(
                "SELECT id, source, amount, kind, tag, interval, original_date, last_inserted_date, active
                 FROM recurring_entries
                 ORDER BY id DESC",
                (),
            )
            .await?;

        let mut entries = Vec::new();
        while let Some(row) = rows.next().await? {
            entries.push(RecurringEntry {
                id: row.get(0)?,
                source: row.get(1)?,
                amount: row.get(2)?,
                kind: TransactionType::from_str(&row.get::<String>(3)?),
                tag: Tag::from_str(&row.get::<String>(4)?),
                interval: RecurringInterval::from_str(&row.get::<String>(5)?),
                original_date: row.get(6)?,
                last_inserted_date: row.get(7)?,
                active: row.get::<i32>(8)? != 0,
            });
        }

        Ok(entries)
    })
}

pub fn add_recurring_entry(
    db: &DbHandle,
    source: &str,
    amount: f64,
    kind: TransactionType,
    tag: &Tag,
    interval: &RecurringInterval,
    original_date: &str,
) -> libsql::Result<()> {
    let conn = &db.conn;
    let source = source.to_string();
    let kind_str = kind.as_str().to_string();
    let tag_str = tag.as_str().to_string();
    let interval_str = interval.as_str().to_string();
    let original_date = original_date.to_string();

    db.rt.block_on(async {
        conn.execute(
            "INSERT INTO recurring_entries (source, amount, kind, tag, interval, original_date, last_inserted_date, active)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![source, amount, kind_str, tag_str, interval_str, original_date, "", 1i32],
        )
        .await?;
        Ok(())
    })
}

pub fn delete_recurring_entry(db: &DbHandle, id: i32) -> libsql::Result<()> {
    let conn = &db.conn;
    db.rt.block_on(async {
        conn.execute(
            "DELETE FROM recurring_entries WHERE id = ?",
            libsql::params![id],
        )
        .await?;
        Ok(())
    })
}

pub fn toggle_recurring_entry(db: &DbHandle, id: i32, active: bool) -> libsql::Result<()> {
    let conn = &db.conn;
    let active_val: i32 = if active { 1 } else { 0 };
    db.rt.block_on(async {
        conn.execute(
            "UPDATE recurring_entries SET active = ? WHERE id = ?",
            libsql::params![active_val, id],
        )
        .await?;
        Ok(())
    })
}

pub fn insert_recurring_transactions(db: &DbHandle) -> libsql::Result<()> {
    let now = chrono::Local::now();
    let today_str = now.format("%Y-%m-%d").to_string();
    let current_week = format!("{:04}-W{:02}", now.year(), now.iso_week().week());
    let current_month = format!("{:04}-{:02}", now.year(), now.month());

    let conn = &db.conn;
    db.rt.block_on(async {
        let mut rows = conn
            .query(
                "SELECT id, source, amount, kind, tag, interval, original_date, last_inserted_date
                 FROM recurring_entries
                 WHERE active = 1",
                (),
            )
            .await?;

        // Collect all entries first so we can release the rows borrow before executing writes.
        let mut entries: Vec<(i32, String, f64, String, String, String, String, String)> =
            Vec::new();
        while let Some(row) = rows.next().await? {
            entries.push((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ));
        }
        drop(rows);

        for (rec_id, source, amount, kind, tag, interval_str, original_date, last_inserted_date) in
            entries
        {
            let interval = RecurringInterval::from_str(&interval_str);
            let kind_enum = TransactionType::from_str(&kind);
            let tag_obj = Tag::from_str(&tag);

            let should_insert = match interval {
                RecurringInterval::Daily => last_inserted_date != today_str,
                RecurringInterval::Weekly => {
                    if let Ok(orig) =
                        chrono::NaiveDate::parse_from_str(&original_date, "%Y-%m-%d")
                    {
                        orig.weekday() == now.weekday()
                            && last_inserted_date != current_week
                    } else {
                        false
                    }
                }
                RecurringInterval::Monthly => {
                    if let Ok(orig) =
                        chrono::NaiveDate::parse_from_str(&original_date, "%Y-%m-%d")
                    {
                        orig.day() == now.day() && last_inserted_date != current_month
                    } else {
                        false
                    }
                }
            };

            if should_insert {
                let kind_str = kind_enum.as_str().to_string();
                let tag_str = tag_obj.as_str().to_string();
                let today = today_str.clone();

                conn.execute(
                    "INSERT INTO transactions (source, amount, kind, tag, date)
                     VALUES (?, ?, ?, ?, ?)",
                    libsql::params![source, amount, kind_str, tag_str, today],
                )
                .await?;

                let new_last_inserted = match interval {
                    RecurringInterval::Daily => today_str.clone(),
                    RecurringInterval::Weekly => current_week.clone(),
                    RecurringInterval::Monthly => current_month.clone(),
                };

                conn.execute(
                    "UPDATE recurring_entries SET last_inserted_date = ? WHERE id = ?",
                    libsql::params![new_last_inserted, rec_id],
                )
                .await?;
            }
        }

        Ok(())
    })
}

// Keep old name for backwards compatibility with any external callers.
pub fn insert_recurring_for_month(db: &DbHandle, _current_month: &str) -> libsql::Result<()> {
    insert_recurring_transactions(db)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RecurringInterval, Tag, TransactionType};

    fn setup() -> DbHandle {
        init_in_memory().expect("failed to init in-memory db")
    }

    #[test]
    fn totals_are_calculated() {
        let db = setup();

        add_transaction(&db, "pay", 100.0, TransactionType::Credit, &Tag::from_str("salary"), "2026-02-23").unwrap();
        add_transaction(&db, "buy", 40.0, TransactionType::Debit, &Tag::from_str("food"), "2026-02-23").unwrap();

        assert_eq!(total_earned(&db).unwrap(), 100.0);
        assert_eq!(total_spent(&db).unwrap(), 40.0);

        let per_tag = spent_per_tag(&db).unwrap();
        assert_eq!(per_tag.get(&Tag::from_str("food")).copied().unwrap_or(0.0), 40.0);
    }

    #[test]
    fn recurring_roundtrip() {
        let db = setup();

        add_recurring_entry(&db, "rent", 500.0, TransactionType::Debit, &Tag::from_str("housing"), &RecurringInterval::Monthly, "2026-02-23").unwrap();

        let entries = get_recurring_entries(&db).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].source, "rent");
        assert_eq!(entries[0].amount, 500.0);
        assert_eq!(entries[0].interval, RecurringInterval::Monthly);
    }
}
