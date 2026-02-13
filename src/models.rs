#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionType {
    Credit,
    Debit,
}

impl TransactionType {
    pub fn as_str(&self) -> &str {
        match self {
            TransactionType::Credit => "credit",
            TransactionType::Debit => "debit",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "credit" => TransactionType::Credit,
            _ => TransactionType::Debit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag(pub String);

impl Tag {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_str(s: &str) -> Self {
        Tag(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: i32,
    pub source: String,
    pub amount: f64,
    pub kind: TransactionType,
    pub tag: Tag,
    pub date: String,
}
#[derive(Debug, Clone)]
pub struct RecurringEntry {
    pub id: i32,
    pub source: String,
    pub amount: f64,
    pub kind: TransactionType,
    pub tag: Tag,
    pub last_inserted_month: String, // Format: "YYYY-MM"
    pub active: bool,
}