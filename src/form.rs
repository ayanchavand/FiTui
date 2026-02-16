use crate::models::TransactionType;

#[derive(PartialEq, Copy, Clone)]
pub enum Field {
    Source,
    Amount,
    Kind,
    Tag,
    Date,
    Recurring,
}

impl Field {
    pub fn next(self) -> Self {
        use Field::*;
        match self {
            Source => Amount,
            Amount => Kind,
            Kind => Tag,
            Tag => Date,
            Date => Recurring,
            Recurring => Source,
        }
    }
}

pub struct TransactionForm {
    pub source: String,
    pub amount: String,
    pub kind: TransactionType,

    // Index into the dynamically loaded config tags
    pub tag_index: usize,

    pub date: String,
    pub recurring: bool,
    pub active: Field,
}

impl TransactionForm {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            amount: String::new(),
            kind: TransactionType::Debit,
            tag_index: 0,
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            recurring: false,
            active: Field::Source,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn push_char(&mut self, c: char) {
        match self.active {
            Field::Source => self.source.push(c),
            Field::Amount => self.amount.push(c),
            Field::Date => self.date.push(c),
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

    pub fn toggle_kind(&mut self) {
        self.kind = match self.kind {
            TransactionType::Credit => TransactionType::Debit,
            TransactionType::Debit => TransactionType::Credit,
        };
    }

    pub fn toggle_recurring(&mut self) {
        self.recurring = !self.recurring;
    }

    pub fn next_tag(&mut self, total_tags: usize) {
        if total_tags == 0 {
            return;
        }

        self.tag_index = (self.tag_index + 1) % total_tags;
    }

    pub fn prev_tag(&mut self, total_tags: usize) {
        if total_tags == 0 {
            return;
        }

        if self.tag_index == 0 {
            self.tag_index = total_tags - 1;
        } else {
            self.tag_index -= 1;
        }
    }
}
