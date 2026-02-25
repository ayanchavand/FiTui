use chrono::Datelike;

use FiTui::{db, models::{Tag, RecurringInterval, TransactionType}};

#[test]
fn full_transaction_lifecycle() {
    let conn = db::init_in_memory().expect("init in-memory");

    // Add
    db::add_transaction(&conn, "pay", 123.45, TransactionType::Credit, &Tag::from_str("salary"), "2026-02-23").unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), 1);

    let id = txs[0].id;

    // Update
    db::update_transaction(&conn, id, "pay-2", 200.0, TransactionType::Credit, &Tag::from_str("salary"), "2026-02-24").unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs[0].source, "pay-2");
    assert_eq!(txs[0].amount, 200.0);

    // Delete
    db::delete_transaction(&conn, id).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert!(txs.is_empty());
}

#[test]
fn recurring_insertion_simulation() {
    let conn = db::init_in_memory().expect("init in-memory");

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let current_month = format!("{:04}-{:02}", chrono::Local::now().year(), chrono::Local::now().month());

    // Add a monthly recurring entry starting today
    db::add_recurring_entry(&conn, "rent", 500.0, TransactionType::Debit, &Tag::from_str("housing"), &RecurringInterval::Monthly, &today).unwrap();

    // Run insert logic
    db::insert_recurring_transactions(&conn).unwrap();

    // A transaction should be created for today
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].date, today);
    assert_eq!(txs[0].source, "rent");

    // Recurring entry's last_inserted_date should be updated to current month
    let entries = db::get_recurring_entries(&conn).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].last_inserted_date, current_month);
}

#[test]
fn recurring_intervals_behavior() {
    let conn = db::init_in_memory().expect("init in-memory");
    let rt = conn.rt();
    let raw = conn.conn();

    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();

    // Helper: reset last_inserted_date for a recurring entry (simulates time passing).
    // Uses inline SQL with literal values to avoid importing libsql directly.
    let reset_last_inserted = |id: i32, reset_val: &str| {
        let sql = format!(
            "UPDATE recurring_entries SET last_inserted_date = '{}' WHERE id = {}",
            reset_val, id
        );
        rt.block_on(async {
            raw.execute(&sql, ()).await.unwrap();
        });
    };

    // === TEST DAILY ===
    db::add_recurring_entry(&conn, "daily-item", 10.0, TransactionType::Debit, &Tag::from_str("test"), &RecurringInterval::Daily, &today).unwrap();
    let daily_entries = db::get_recurring_entries(&conn).unwrap();
    let daily_id = daily_entries.iter().find(|e| e.source == "daily-item").unwrap().id;

    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), 1);

    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), 1, "daily should not insert twice on same day");

    reset_last_inserted(daily_id, "");
    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), 2, "daily should insert on new day");

    // === TEST WEEKLY ===
    let daily_txs_count = db::get_transactions(&conn).unwrap().len();

    db::add_recurring_entry(&conn, "weekly-item", 20.0, TransactionType::Debit, &Tag::from_str("test"), &RecurringInterval::Weekly, &today).unwrap();
    let weekly_entries = db::get_recurring_entries(&conn).unwrap();
    let weekly_id = weekly_entries.iter().find(|e| e.source == "weekly-item").unwrap().id;

    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), daily_txs_count + 1, "weekly should insert on matching day");

    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), daily_txs_count + 1, "weekly should not insert twice in same week");

    reset_last_inserted(weekly_id, "");
    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), daily_txs_count + 2, "weekly should insert next week");

    // === TEST MONTHLY ===
    let weekly_txs_count = db::get_transactions(&conn).unwrap().len();

    db::add_recurring_entry(&conn, "monthly-item", 30.0, TransactionType::Debit, &Tag::from_str("test"), &RecurringInterval::Monthly, &today).unwrap();
    let monthly_entries = db::get_recurring_entries(&conn).unwrap();
    let monthly_id = monthly_entries.iter().find(|e| e.source == "monthly-item").unwrap().id;

    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), weekly_txs_count + 1, "monthly should insert on matching day");

    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), weekly_txs_count + 1, "monthly should not insert twice in same month");

    reset_last_inserted(monthly_id, "");
    db::insert_recurring_transactions(&conn).unwrap();
    let txs = db::get_transactions(&conn).unwrap();
    assert_eq!(txs.len(), weekly_txs_count + 2, "monthly should insert next month");
}

#[test]
fn migration_safety_test() {
    // Create a raw in-memory DB without any schema (simulates an old install).
    let db = db::init_in_memory_empty().expect("open empty in-memory db");
    let rt = db.rt();
    let conn = db.conn();

    // Create the old recurring_entries table (missing the columns added in later versions).
    rt.block_on(async {
        conn.execute(
            "CREATE TABLE recurring_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL,
                amount REAL NOT NULL,
                kind TEXT NOT NULL,
                tag TEXT NOT NULL,
                last_inserted_month TEXT NOT NULL DEFAULT ''
            )",
            (),
        )
        .await
        .unwrap();

        // Inline literal values so we don't need libsql::params! in this file.
        conn.execute(
            "INSERT INTO recurring_entries (source, amount, kind, tag, last_inserted_month) \
             VALUES ('x', 10.0, 'debit', 'a', '2026-02')",
            (),
        )
        .await
        .unwrap();
    });

    // Run migration.
    db::migrate_recurring_entries_schema(&db).unwrap();

    // last_inserted_date should exist and contain the migrated value.
    let migrated: String = rt.block_on(async {
        let mut rows = conn
            .query("SELECT last_inserted_date FROM recurring_entries WHERE id = 1", ())
            .await
            .unwrap();
        rows.next().await.unwrap().unwrap().get(0).unwrap()
    });
    assert_eq!(migrated, "2026-02");

    // New columns interval, original_date, active should exist (SELECT must not error).
    rt.block_on(async {
        let mut rows = conn
            .query("SELECT interval FROM recurring_entries WHERE id = 1", ())
            .await
            .unwrap();
        let _: String = rows.next().await.unwrap().unwrap().get(0).unwrap();

        let mut rows = conn
            .query("SELECT original_date FROM recurring_entries WHERE id = 1", ())
            .await
            .unwrap();
        let _: String = rows.next().await.unwrap().unwrap().get(0).unwrap();

        let mut rows = conn
            .query("SELECT active FROM recurring_entries WHERE id = 1", ())
            .await
            .unwrap();
        let active: i32 = rows.next().await.unwrap().unwrap().get(0).unwrap();
        assert!(active == 0 || active == 1);
    });
}
