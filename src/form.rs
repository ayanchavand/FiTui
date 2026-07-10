use crate::models::{RecurringInterval, TransactionType};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Field {
    Source,
    Amount,
    Kind,
    Tag,
    Date,
    Recurring,
    RecurringInterval,
}

// Canonical visual/focus order for the form fields. Use this as the single
// source of truth for rendering order and keyboard traversal.
pub const FIELD_ORDER: &[Field] = &[
    Field::Source,
    Field::Amount,
    Field::Date,
    Field::Kind,
    Field::Tag,
    Field::Recurring,
    Field::RecurringInterval,
];

impl Field {
    pub fn next(self) -> Self {
        // Find the current field in FIELD_ORDER and return the next one,
        // wrapping around to the first.
        let len = FIELD_ORDER.len();
        for (i, &f) in FIELD_ORDER.iter().enumerate() {
            if f == self {
                return FIELD_ORDER[(i + 1) % len];
            }
        }

        // Fallback: if not found (shouldn't happen), return Source
        Field::Source
    }

    pub fn back(self) -> Self {
        // Find the current field in FIELD_ORDER and return the next one,
        // wrapping around to the first.
        let len = FIELD_ORDER.len();
        for (i, &f) in FIELD_ORDER.iter().enumerate() {
            if f == self {
                return FIELD_ORDER[(i + len - 1) % len];
            }
        }

        // Fallback: if not found (shouldn't happen), return Source
        Field::Source
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
    pub recurring_interval: RecurringInterval,
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
            recurring_interval: RecurringInterval::Monthly,
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

    pub fn next_interval(&mut self) {
        self.recurring_interval = self.recurring_interval.next();
    }

    pub fn prev_interval(&mut self) {
        self.recurring_interval = self.recurring_interval.prev();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RecurringInterval, TransactionType};

    #[test]
    fn field_next_wraps() {
        assert_eq!(Field::Source.next(), Field::Amount);
        // Walk through full cycle and end up at Source again
        let mut f = Field::Source;
        for _ in 0..FIELD_ORDER.len() {
            f = f.next();
        }
        assert_eq!(f, Field::Source);
    }

    #[test]
    fn toggle_kind_swaps() {
        let mut form = TransactionForm::new();
        assert_eq!(form.kind, TransactionType::Debit);
        form.toggle_kind();
        assert_eq!(form.kind, TransactionType::Credit);
        form.toggle_kind();
        assert_eq!(form.kind, TransactionType::Debit);
    }

    #[test]
    fn tag_index_wraps_next_prev() {
        let mut form = TransactionForm::new();
        form.tag_index = 0;
        form.next_tag(3);
        assert_eq!(form.tag_index, 1);
        form.next_tag(3);
        form.next_tag(3);
        assert_eq!(form.tag_index, 0); // wrapped

        form.prev_tag(3);
        assert_eq!(form.tag_index, 2); // wrapped backwards
    }

    #[test]
    fn push_and_pop_chars_affect_fields() {
        let mut form = TransactionForm::new();
        form.active = Field::Source;
        form.push_char('a');
        form.push_char('b');
        assert_eq!(form.source, "ab");
        form.pop_char();
        assert_eq!(form.source, "a");

        form.active = Field::Amount;
        form.push_char('1');
        form.push_char('0');
        assert_eq!(form.amount, "10");
        form.pop_char();
        assert_eq!(form.amount, "1");
    }

    #[test]
    fn interval_next_prev_cycle() {
        let mut form = TransactionForm::new();
        assert_eq!(form.recurring_interval, RecurringInterval::Monthly);
        form.next_interval();
        assert_eq!(form.recurring_interval, RecurringInterval::Daily);
        form.prev_interval();
        assert_eq!(form.recurring_interval, RecurringInterval::Monthly);
    }
}
