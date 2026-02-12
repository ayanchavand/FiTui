use rusqlite::Connection;

use crate::{
    config::load_config,
    db,
   form::TransactionForm,
    models::{Tag, Transaction},
};

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Adding,
    Stats,
}

pub struct App {
    pub mode: Mode,
    pub form: TransactionForm,
    // When Some(id) we're editing an existing transaction
    pub editing: Option<i32>,

    // Tags loaded from YAML config
    pub tags: Vec<Tag>,

    pub transactions: Vec<Transaction>,
    pub selected: usize,
    pub currency: String,
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

        Self {
            mode: Mode::Normal,
            form: TransactionForm::new(),
            editing: None,
            tags,
            transactions,
            selected: 0,
            currency: config.currency,
        }
    }

    pub fn refresh(&mut self, conn: &Connection) {
        self.transactions = db::get_transactions(conn).unwrap_or_default();

        // Clamp selection if list shrinks
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

    pub fn delete_selected(&mut self, conn: &Connection) {
        if self.transactions.is_empty() {
            return;
        }

        let id = self.transactions[self.selected].id;
        db::delete_transaction(conn, id).unwrap();

        self.refresh(conn);
    }
}
