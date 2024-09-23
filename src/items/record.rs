use chrono::{NaiveDate};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct RecordForLedger {
    pub id: u32,
    pub posted: bool,
    pub name: String,
    pub amount: f32,
    pub balance: f32,
    pub category: String,
    pub notes: String,
    pub date: NaiveDate,
    pub modified: bool,
}

#[derive(Debug, Deserialize)]
pub struct RecordForCSV {
    pub date: String,
    pub posted: String,
    pub name: String,
    pub debit: String,
    pub credit: String,
    pub balance: String,
    pub category: String,
    pub notes: String,
    pub id: u32
}

impl RecordForLedger {
    pub fn new() -> RecordForLedger {
        RecordForLedger {
            posted: false,
            name: "".to_string(),
            amount: 0.0,
            balance: 0.0,
            category: "".to_string(),
            notes: "".to_string(),
            id: 0,
            date: NaiveDate::default(),
            modified: false
        }
    }
    pub fn to(&self) -> RecordForCSV {
        RecordForCSV {
            date: self.date.format("%-m/%-d/%Y").to_string(),
            posted: match &self.posted {
                true =>  "x".to_string(),
                false => "".to_string()
            },
            name: self.name.clone(),
            debit: if self.amount <= 0.0 { format!("{:.2}", &self.amount.abs()) } else { "".to_string() },
            credit:  if self.amount >= 0.0 { format!("{:.2}", &self.amount.abs()) } else { "".to_string() },
            balance: format!("{:.2}", self.balance),
            category: self.category.clone(),
            notes: self.notes.clone(),
            id: self.id,
        }
    }

    pub fn from(r: RecordForCSV) -> RecordForLedger {
        let d = NaiveDate::parse_from_str(&*r.date, "%-m/%-d/%Y").unwrap();
        RecordForLedger{
            posted: r.posted == "x".to_string(),
            name: r.name.clone(),
            amount: r.credit.parse::<f32>().unwrap_or_else(|_| 0.0) - r.debit.parse::<f32>().unwrap_or_else(|_| 0.0),
            balance: r.balance.parse::<f32>().unwrap_or_else(|_| 0.0),
            category: r.category.clone(),
            notes: r.notes.clone(),
            id: r.id,
            date: d.clone(),
            modified: false,
        }
    }
}

