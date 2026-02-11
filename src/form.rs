use crate::models::{Tag, TransactionType};

#[derive(PartialEq, Copy, Clone)]
pub enum Field {
    Source,
    Amount,
    Kind,
    Tag,
    Date,
}

impl Field {
    pub fn next(self) -> Self {
        use Field::*;
        match self {
            Source => Amount,
            Amount => Kind,
            Kind => Tag,
            Tag => Date,
            Date => Source,
        }
    }
}

pub struct TransactionForm {
    pub source: String,
    pub amount: String,
    pub kind: TransactionType,
    pub tag: Tag,
    pub date: String,

    pub active: Field,
}

impl TransactionForm {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            amount: String::new(),
            kind: TransactionType::Debit,
            tag: Tag::Other,
            date: "2026-02-11".into(),
            active: Field::Source,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    // ✅ Only text fields accept typing
    pub fn push_char(&mut self, c: char) {
        match self.active {
            Field::Source => self.source.push(c),
            Field::Amount => self.amount.push(c),
            Field::Date => self.date.push(c),

            // Kind + Tag no longer accept raw typing
            _ => {}
        }
    }

    pub fn pop_char(&mut self) {
        match self.active {
            Field::Source => {
                self.source.pop();
            }
            Field::Amount => {
                self.amount.pop();
            }
            Field::Date => {
                self.date.pop();
            }
            _ => {}
        }
    }

    // ✅ Toggle Credit/Debit
    pub fn toggle_kind(&mut self) {
        self.kind = match self.kind {
            TransactionType::Credit => TransactionType::Debit,
            TransactionType::Debit => TransactionType::Credit,
        };
    }

    // ✅ Cycle Tags
    pub fn next_tag(&mut self) {
        self.tag = match self.tag {
            Tag::Food => Tag::Travel,
            Tag::Travel => Tag::Shopping,
            Tag::Shopping => Tag::Bills,
            Tag::Bills => Tag::Salary,
            Tag::Salary => Tag::Other,
            Tag::Other => Tag::Food,
        };
    }
}
