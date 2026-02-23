use rusqlite::Connection;

use crate::{
    config::load_config,
    db,
    form::TransactionForm,
    models::{RecurringEntry, Tag, Transaction},
};

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Adding,
    Stats,
    Popup,
    RecurringManagement,
}

#[derive(Clone)]
pub enum PopupAction {
    DeleteTransaction(i32),
    Quit,
}

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
    pub editing: Option<i32>,
    pub tags: Vec<Tag>,
    pub transactions: Vec<Transaction>,
    pub recurring_entries: Vec<RecurringEntry>,
    pub selected: usize,
    pub selected_recurring: usize,
    pub currency: String,
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
            selected_recurring: 0,
            currency: config.currency,
            popup: None,
        }
    }

    pub fn refresh(&mut self, conn: &Connection) {
        self.transactions = db::get_transactions(conn).unwrap_or_default();
        self.recurring_entries = db::get_recurring_entries(conn).unwrap_or_default();

        if self.selected >= self.transactions.len() && self.selected > 0 {
            self.selected -= 1;
        }
    }

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

            if self.form.recurring {
                db::add_recurring_entry(
                    conn,
                    &self.form.source,
                    amount,
                    self.form.kind,
                    &tag,
                    &self.form.recurring_interval,
                    &self.form.date,
                )
                .unwrap();
            }
        }

        self.refresh(conn);
    }

    pub fn begin_edit_selected(&mut self) {
        if self.transactions.is_empty() {
            return;
        }

        let tx = &self.transactions[self.selected];

        self.form.source = tx.source.clone();
        self.form.amount = format!("{:.2}", tx.amount);
        self.form.kind = tx.kind;

        self.form.tag_index = self
            .tags
            .iter()
            .position(|t| t.as_str() == tx.tag.as_str())
            .unwrap_or(0);

        self.form.date = tx.date.clone();
        self.form.active = crate::form::Field::Source;

        let recurring_entry = self.recurring_entries.iter().find(|r| {
            r.source == tx.source
                && r.amount == tx.amount
                && r.kind == tx.kind
                && r.tag == tx.tag
        });

        if let Some(entry) = recurring_entry {
            self.form.recurring = true;
            self.form.recurring_interval = entry.interval.clone();
        } else {
            self.form.recurring = false;
            self.form.recurring_interval = crate::models::RecurringInterval::Monthly;
        }

        self.mode = Mode::Adding;
        self.editing = Some(tx.id);
    }

    pub fn delete_selected(&mut self, conn: &Connection) {
        if self.transactions.is_empty() {
            return;
        }

        let id = self.transactions[self.selected].id;
        db::delete_transaction(conn, id).unwrap();

        self.refresh(conn);
    }

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

    pub fn open_info_popup(&mut self, title: &str, message: String) {
        self.popup = Some(PopupKind::Info {
            title: title.into(),
            message,
        });

        self.mode = Mode::Popup;
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
        self.mode = Mode::Normal;
    }

    pub fn selected_transaction(&self) -> Option<&Transaction> {
        self.transactions.get(self.selected)
    }

    pub fn get_recurring_for_transaction(&self, tx: &Transaction) -> Option<&RecurringEntry> {
        self.recurring_entries.iter().find(|r| {
            r.source == tx.source
                && r.amount == tx.amount
                && r.kind == tx.kind
                && r.tag == tx.tag
                && r.active
        })
    }
}