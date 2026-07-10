#!/usr/bin/env python3
"""Generate realistic development data for FiTui debug builds.

This script recreates the same local SQLite database path used by the app in
non-release builds: ./data/budget.db

It populates the main 'transactions' table and the recurring entries table with
sample data that exercises the app UI and stats pages.
"""

from __future__ import annotations

import os
import sqlite3
from datetime import date, timedelta
from random import Random

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir))
DATA_DIR = os.path.join(ROOT, "data")
DB_PATH = os.path.join(DATA_DIR, "budget.db")

rng = Random(7)

BASE_TRANSACTIONS = [
    ("Salary", 3200.0, "credit", "income", "2024-01-01"),
    ("Client Retainer", 1200.0, "credit", "income", "2024-01-05"),
    ("Cloud Hosting", 145.0, "debit", "ops", "2024-01-06"),
    ("Team Lunch", 68.5, "debit", "food", "2024-01-07"),
    ("API Subscriptions", 89.0, "debit", "ops", "2024-01-08"),
    ("Design Sprint", 900.0, "debit", "product", "2024-01-10"),
    ("Quarterly License", 350.0, "debit", "ops", "2024-01-12"),
    ("Consulting Invoice", 750.0, "credit", "income", "2024-01-15"),
    ("Coffee & Sync", 24.0, "debit", "food", "2024-01-16"),
    ("Infrastructure Spend", 210.0, "debit", "ops", "2024-01-18"),
]

BASE_RECURRING = [
    ("Cloud Hosting", 145.0, "debit", "ops", "monthly", "2024-01-06", "", 1),
    ("Team Lunch", 68.5, "debit", "food", "weekly", "2024-01-07", "", 1),
    ("API Subscriptions", 89.0, "debit", "ops", "monthly", "2024-01-08", "", 1),
]


def ensure_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            amount REAL NOT NULL,
            kind TEXT NOT NULL,
            tag TEXT NOT NULL,
            date TEXT NOT NULL
        )
        """
    )
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS recurring_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            amount REAL NOT NULL,
            kind TEXT NOT NULL,
            tag TEXT NOT NULL,
            interval TEXT NOT NULL DEFAULT 'monthly',
            original_date TEXT NOT NULL,
            last_inserted_date TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1
        )
        """
    )
    conn.commit()


def populate_transactions(conn: sqlite3.Connection) -> None:
    conn.execute("DELETE FROM transactions")

    for source, amount, kind, tag, date_str in BASE_TRANSACTIONS:
        conn.execute(
            "INSERT INTO transactions (source, amount, kind, tag, date) VALUES (?, ?, ?, ?, ?)",
            (source, amount, kind, tag, date_str),
        )

    start = date(2024, 2, 1)
    for idx in range(24):
        day = start + timedelta(days=idx)
        source = rng.choice([
            "Roadmap Sync",
            "Sprint Review",
            "Vendor Ops",
            "Automation Run",
            "Analytics Dash",
            "Support Triage",
            "Security Scan",
            "Backlog Grooming",
        ])
        amount = round(rng.uniform(20.0, 260.0), 2)
        kind = "credit" if rng.random() < 0.35 else "debit"
        tag = rng.choice(["income", "ops", "product", "food", "travel"])
        conn.execute(
            "INSERT INTO transactions (source, amount, kind, tag, date) VALUES (?, ?, ?, ?, ?)",
            (source, amount, kind, tag, day.strftime("%Y-%m-%d")),
        )

    conn.commit()


def populate_recurring(conn: sqlite3.Connection) -> None:
    conn.execute("DELETE FROM recurring_entries")

    for source, amount, kind, tag, interval, original_date, last_inserted_date, active in BASE_RECURRING:
        conn.execute(
            """
            INSERT INTO recurring_entries (
                source, amount, kind, tag, interval, original_date, last_inserted_date, active
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (source, amount, kind, tag, interval, original_date, last_inserted_date, active),
        )

    conn.commit()


def main() -> None:
    os.makedirs(DATA_DIR, exist_ok=True)
    conn = sqlite3.connect(DB_PATH)
    ensure_schema(conn)
    populate_transactions(conn)
    populate_recurring(conn)
    conn.close()
    print(f"Generated development data at {DB_PATH}")


if __name__ == "__main__":
    main()
