use rusqlite::Connection;

use crate::{
    config::load_config,
    db,
    form::TransactionForm,
    models::{RecurringEntry, Tag, Transaction},
};

/// Main UI modes
#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Adding,
    Stats,
    Popup, // 👈 generic popup mode
}

/// Actions a popup can trigger
#[derive(Clone)]
pub enum PopupAction {
    DeleteTransaction(i32),
    Quit,
}

/// Popup types (reusable for future dialogs)
#[derive(Clone)]
pub enum PopupKind {
    Confirm {
        title: String,
        message: String,
        action: PopupAction,
    },
    Info {
        title: String,
        message: String,
    },
}

pub struct App {
    pub mode: Mode,
    pub form: TransactionForm,

    // When Some(id) we're editing an existing transaction
    pub editing: Option<i32>,

    // Tags loaded from YAML config
    pub tags: Vec<Tag>,

    pub transactions: Vec<Transaction>,
    pub recurring_entries: Vec<RecurringEntry>,
    pub selected: usize,

    pub currency: String,

    // 👇 Popup state
    pub popup: Option<PopupKind>,
}

impl App {
    pub fn new(conn: &Connection) -> Self {
        let config = load_config();

        let tags: Vec<Tag> = config
            .tags
            .into_iter()
            .map(|s| Tag::from_str(&s))
            .collect();

        let transactions = db::get_transactions(conn).unwrap_or_default();
        let recurring_entries = db::get_recurring_entries(conn).unwrap_or_default();

        Self {
            mode: Mode::Normal,
            form: TransactionForm::new(),
            editing: None,
            tags,
            transactions,
            recurring_entries,
            selected: 0,
            currency: config.currency,

            popup: None, // 👈 init popup
        }
    }

    /// Refresh transactions + recurring entries from DB
    pub fn refresh(&mut self, conn: &Connection) {
        self.transactions = db::get_transactions(conn).unwrap_or_default();
        self.recurring_entries = db::get_recurring_entries(conn).unwrap_or_default();

        // Clamp selection if list shrinks
        if self.selected >= self.transactions.len() && self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Save transaction (new or edit)
    pub fn save_transaction(&mut self, conn: &Connection) {
        let amount: f64 = self.form.amount.trim().parse().unwrap_or(0.0);

        let tag = self
            .tags
            .get(self.form.tag_index)
            .unwrap_or(&Tag("other".into()))
            .clone();

        if let Some(id) = self.editing {
            db::update_transaction(
                conn,
                id,
                &self.form.source,
                amount,
                self.form.kind,
                &tag,
                &self.form.date,
            )
            .unwrap();

            self.editing = None;
        } else {
            db::add_transaction(
                conn,
                &self.form.source,
                amount,
                self.form.kind,
                &tag,
                &self.form.date,
            )
            .unwrap();

            // If marked as recurring, also add to recurring_entries
            if self.form.recurring {
                db::add_recurring_entry(
                    conn,
                    &self.form.source,
                    amount,
                    self.form.kind,
                    &tag,
                )
                .unwrap();
            }
        }

        self.refresh(conn);
    }

    /// Begin editing currently selected transaction
    pub fn begin_edit_selected(&mut self) {
        if self.transactions.is_empty() {
            return;
        }

        let tx = &self.transactions[self.selected];

        self.form.source = tx.source.clone();
        self.form.amount = format!("{:.2}", tx.amount);
        self.form.kind = tx.kind;

        // Find tag index matching the transaction's tag
        self.form.tag_index = self
            .tags
            .iter()
            .position(|t| t.as_str() == tx.tag.as_str())
            .unwrap_or(0);

        self.form.date = tx.date.clone();
        self.form.active = crate::form::Field::Source;

        self.mode = Mode::Adding;
        self.editing = Some(tx.id);
    }

    /// Delete currently selected transaction (direct delete)
    pub fn delete_selected(&mut self, conn: &Connection) {
        if self.transactions.is_empty() {
            return;
        }

        let id = self.transactions[self.selected].id;
        db::delete_transaction(conn, id).unwrap();

        self.refresh(conn);
    }

    // ============================================================
    // POPUP SYSTEM (Reusable)
    // ============================================================

    /// Open a confirm popup
    pub fn open_confirm_popup(
        &mut self,
        title: &str,
        message: String,
        action: PopupAction,
    ) {
        self.popup = Some(PopupKind::Confirm {
            title: title.into(),
            message,
            action,
        });

        self.mode = Mode::Popup;
    }

    /// Open an info popup
    pub fn open_info_popup(&mut self, title: &str, message: String) {
        self.popup = Some(PopupKind::Info {
            title: title.into(),
            message,
        });

        self.mode = Mode::Popup;
    }

    /// Close popup and return to Normal mode
    pub fn close_popup(&mut self) {
        self.popup = None;
        self.mode = Mode::Normal;
    }

    /// Helper: get selected transaction safely
    pub fn selected_transaction(&self) -> Option<&Transaction> {
        self.transactions.get(self.selected)
    }
}
