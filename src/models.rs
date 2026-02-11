#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Tag {
    Food,
    Travel,
    Shopping,
    Bills,
    Salary,
    Other,
}

impl Tag {
    pub fn as_str(&self) -> &str {
        match self {
            Tag::Food => "food",
            Tag::Travel => "travel",
            Tag::Shopping => "shopping",
            Tag::Bills => "bills",
            Tag::Salary => "salary",
            Tag::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "food" => Tag::Food,
            "travel" => Tag::Travel,
            "shopping" => Tag::Shopping,
            "bills" => Tag::Bills,
            "salary" => Tag::Salary,
            _ => Tag::Other,
        }
    }

    pub fn all() -> Vec<Tag> {
        vec![
            Tag::Food,
            Tag::Travel,
            Tag::Shopping,
            Tag::Bills,
            Tag::Salary,
            Tag::Other,
        ]
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
