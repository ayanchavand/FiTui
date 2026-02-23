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
#[derive(Debug, Clone, PartialEq)]
pub enum RecurringInterval {
    Daily,
    Weekly,
    Monthly,
}

impl RecurringInterval {
    pub fn as_str(&self) -> &str {
        match self {
            RecurringInterval::Daily => "daily",
            RecurringInterval::Weekly => "weekly",
            RecurringInterval::Monthly => "monthly",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "daily" => RecurringInterval::Daily,
            "weekly" => RecurringInterval::Weekly,
            "monthly" => RecurringInterval::Monthly,
            _ => RecurringInterval::Monthly, // Default to monthly
        }
    }

    pub fn display(&self) -> &str {
        match self {
            RecurringInterval::Daily => "Daily",
            RecurringInterval::Weekly => "Weekly",
            RecurringInterval::Monthly => "Monthly",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            RecurringInterval::Daily => RecurringInterval::Weekly,
            RecurringInterval::Weekly => RecurringInterval::Monthly,
            RecurringInterval::Monthly => RecurringInterval::Daily,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            RecurringInterval::Daily => RecurringInterval::Monthly,
            RecurringInterval::Weekly => RecurringInterval::Daily,
            RecurringInterval::Monthly => RecurringInterval::Weekly,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecurringEntry {
    pub id: i32,
    pub source: String,
    pub amount: f64,
    pub kind: TransactionType,
    pub tag: Tag,
    pub interval: RecurringInterval,
    pub original_date: String, // Format: "YYYY-MM-DD" - date when recurring entry was created
    pub last_inserted_date: String, // Format: depends on interval (YYYY-MM-DD for daily, YYYY-Www for weekly, YYYY-MM for monthly)
    pub active: bool,
}