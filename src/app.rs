#![allow(dead_code)]
use rusqlite::Connection;

use crate::{
    config::load_config,
    db,
    form::TransactionForm,
    models::{RecurringEntry, Tag, Transaction},
    theme::Theme,
};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Adding,
    Stats,
    Popup,
    RecurringManagement,
    Filtering,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FilterField {
    StartDate,
    EndDate,
    Tag,
}

impl FilterField {
    pub fn next(&self) -> Self {
        match self {
            Self::StartDate => Self::EndDate,
            Self::EndDate => Self::Tag,
            Self::Tag => Self::StartDate,
        }
    }

    pub fn back(&self) -> Self {
        match self {
            Self::StartDate => Self::Tag,
            Self::EndDate => Self::StartDate,
            Self::Tag => Self::EndDate,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransactionFilter {
    pub active: bool,
    pub start_date: String,
    pub end_date: String,
    pub tag_index: Option<usize>, // None represents "All"
    pub active_field: FilterField,
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
    pub theme: Theme,
    pub filter: TransactionFilter,
}

// helpers for tab management; the UI shows three tabs and the
// current mode determines which one is active. We don't store an
// index, it's derived so adding new modes isn't error prone.
impl App {
    /// Return the index of the currently active tab:
    /// 0 = transactions, 1 = stats, 2 = recurring management.
    pub fn current_tab(&self) -> usize {
        match self.mode {
            Mode::Normal | Mode::Adding | Mode::Popup | Mode::Filtering => 0,
            Mode::Stats => 1,
            Mode::RecurringManagement => 2,
        }
    }

    /// Switch to the given tab index, updating `mode` accordingly.
    /// Indices outside 0..=2 are wrapped.
    pub fn set_tab(&mut self, idx: usize) {
        let idx_mod = idx % 3;
        self.mode = match idx_mod {
            0 => Mode::Normal,
            1 => Mode::Stats,
            2 => {
                // make sure selection doesn't go out of bounds when
                // entering recurring view
                if self.selected_recurring >= self.recurring_entries.len() &&
                    self.selected_recurring > 0
                {
                    self.selected_recurring = self.recurring_entries.len() - 1;
                }
                Mode::RecurringManagement
            }
            _ => unreachable!(),
        };
    }

    /// Advance to the next tab (wraps).
    pub fn next_tab(&mut self) {
        let next = self.current_tab().saturating_add(1);
        self.set_tab(next);
    }

    /// Go to the previous tab (wraps backwards).
    pub fn prev_tab(&mut self) {
        let tab = self.current_tab();
        let prev = if tab == 0 { 2 } else { tab - 1 };
        self.set_tab(prev);
    }
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

        let theme_name = &config.theme;
        let theme = if let Some(custom_config) = config.custom_themes.get(theme_name) {
            match Theme::from_config(custom_config) {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("Error parsing custom theme '{}': {}. Falling back to default.", theme_name, err);
                    Theme::default()
                }
            }
        } else if let Some(preconfigured) = Theme::get_preconfigured(theme_name) {
            preconfigured
        } else {
            eprintln!("Theme '{}' not found. Falling back to default.", theme_name);
            Theme::default()
        };

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
            theme,
            filter: TransactionFilter {
                active: false,
                start_date: String::new(),
                end_date: String::new(),
                tag_index: None,
                active_field: FilterField::StartDate,
            },
        }
    }

    pub fn refresh(&mut self, conn: &Connection) {
        self.transactions = db::get_transactions(conn).unwrap_or_default();
        self.recurring_entries = db::get_recurring_entries(conn).unwrap_or_default();

        let max_len = std::cmp::min(15, self.transactions.len());
        if self.selected >= max_len && max_len > 0 {
            self.selected = max_len - 1;
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

    pub fn get_filtered_transactions(&self) -> Vec<Transaction> {
        if !self.filter.active {
            return self.transactions.clone();
        }
        self.transactions
            .iter()
            .filter(|tx| {
                if let Some(tag_idx) = self.filter.tag_index {
                    if tx.tag.as_str() != self.tags[tag_idx].as_str() {
                        return false;
                    }
                }
                if !self.filter.start_date.is_empty() {
                    if tx.date < self.filter.start_date {
                        return false;
                    }
                }
                if !self.filter.end_date.is_empty() {
                    if tx.date > self.filter.end_date {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn begin_edit_selected(&mut self) {
        let tx = match self.selected_transaction() {
            Some(t) => t,
            None => return,
        };

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
        if let Some(tx) = self.selected_transaction() {
            db::delete_transaction(conn, tx.id).unwrap();
            self.refresh(conn);
        }
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

    pub fn selected_transaction(&self) -> Option<Transaction> {
        let filtered = self.get_filtered_transactions();
        filtered.get(self.selected).cloned()
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

// ---------------------------------------------------------------------------
// tests for tab navigation helpers
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn base_app() -> App {
        let conn = db::init_in_memory().unwrap();
        App::new(&conn)
    }

    #[test]
    fn initial_tab() {
        let app = base_app();
        assert_eq!(app.current_tab(), 0);
    }

    #[test]
    fn cycle_forward_and_back() {
        let mut app = base_app();
        app.next_tab();
        assert_eq!(app.current_tab(), 1);
        app.next_tab();
        assert_eq!(app.current_tab(), 2);
        app.next_tab();
        assert_eq!(app.current_tab(), 0);

        app.prev_tab();
        assert_eq!(app.current_tab(), 2);
        app.prev_tab();
        assert_eq!(app.current_tab(), 1);
        app.prev_tab();
        assert_eq!(app.current_tab(), 0);
    }

    #[test]
    fn set_tab_wraps_modulo() {
        let mut app = base_app();
        app.set_tab(3);
        assert_eq!(app.current_tab(), 0);
        app.set_tab(4);
        assert_eq!(app.current_tab(), 1);
        app.set_tab(5);
        assert_eq!(app.current_tab(), 2);
    }

    #[test]
    fn test_transaction_filtering() {
        let mut app = base_app();
        use crate::models::{Transaction, TransactionType, Tag};
        
        let tx1 = Transaction {
            id: 1,
            source: "Food shop".into(),
            amount: 50.0,
            kind: TransactionType::Debit,
            tag: Tag("food".into()),
            date: "2024-02-10".into(),
        };
        let tx2 = Transaction {
            id: 2,
            source: "Salary".into(),
            amount: 2000.0,
            kind: TransactionType::Credit,
            tag: Tag("salary".into()),
            date: "2024-02-15".into(),
        };
        let tx3 = Transaction {
            id: 3,
            source: "Hosting".into(),
            amount: 15.0,
            kind: TransactionType::Debit,
            tag: Tag("ops".into()),
            date: "2024-03-01".into(),
        };
        
        app.transactions = vec![tx1, tx2, tx3];
        app.tags = vec![Tag("food".into()), Tag("salary".into()), Tag("ops".into())];
        
        // Initially no filter
        assert_eq!(app.get_filtered_transactions().len(), 3);
        
        // Filter by start_date
        app.filter.active = true;
        app.filter.start_date = "2024-02-11".into();
        let filtered = app.get_filtered_transactions();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].source, "Salary");
        assert_eq!(filtered[1].source, "Hosting");
        
        // Filter by start_date and end_date range
        app.filter.end_date = "2024-02-28".into();
        let filtered = app.get_filtered_transactions();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source, "Salary");
        
        // Filter by range and tag
        app.filter.tag_index = Some(0); // "food"
        let filtered = app.get_filtered_transactions();
        assert_eq!(filtered.len(), 0); // "food" is 2024-02-10, outside range
        
        // Clear filter
        app.filter.active = false;
        assert_eq!(app.get_filtered_transactions().len(), 3);
    }
}