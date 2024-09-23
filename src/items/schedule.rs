use std::str::FromStr;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ScheduleForLedger {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub interval: Interval,
    pub amount: f32,
    pub active: bool,
    pub start: NaiveDate,
    pub end: Option<NaiveDate>,
    pub modified: bool,
}
#[derive(Debug, Clone)]
pub enum Interval {
    Week(u16),
    Month(u16),
}

impl FromStr for Interval {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Invalid format".to_string());
        }

        let amount: u16 = parts[0].parse().map_err(|_| "Invalid number".to_string())?;
        match parts[1].to_lowercase().as_str() {
            "week" | "weeks" => Ok(Interval::Week(amount)),
            "month" | "months" => Ok(Interval::Month(amount)),
            _ => Err("Invalid interval type".to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScheduleForCSV {
    pub name: String,
    pub category: String,
    pub interval: String,
    pub amount: String,
    pub start: String,
    pub end: String,
    pub id: u32,
}

impl ScheduleForLedger {
    pub fn new() -> ScheduleForLedger {
        ScheduleForLedger {
            id: 0,
            name: "".to_string(),
            category: "".to_string(),
            interval: Interval::Month(1),
            amount: 0.0,
            active: false,
            start: NaiveDate::default(),
            end: None,
            modified: false,
        }
    }

    pub fn to(&self) -> ScheduleForCSV {
        ScheduleForCSV {
            id: self.id,
            name: self.name.clone(),
            category: self.category.clone(),
            interval: match self.interval {
                Interval::Month(month) => format!("{} Month{}", month, if month == 1 { "" } else { "s" }),
                Interval::Week(week) => format!("{} Week{}", week, if week == 0 { "" } else { "s" })
            },
            amount: format!("{:.2}", self.amount),
            start: self.start.format("%-m/%-d/%Y").to_string(),
            end: match self.end {
                Some(end_timestamp) => end_timestamp.clone().format("%-m/%-d/%Y").to_string(),
                None => "".to_string()
            },
        }
    }

    pub fn from(s: ScheduleForCSV) -> ScheduleForLedger {
        let start_date = NaiveDate::parse_from_str(&*s.start, "%-m/%-d/%Y").unwrap();
        ScheduleForLedger {
            id: s.id,
            name: s.name.clone(),
            category: s.category.clone(),
            interval: s.interval.parse::<Interval>().unwrap_or_else(|_| Interval::Month(1)),
            amount: s.amount.parse::<f32>().unwrap_or_else(|_| 0.0),
            start: start_date.clone(),
            end: match s.end.as_str() {
                "" => None,
                end_date => {
                    match NaiveDate::parse_from_str(end_date, "%-m/%-d/%Y") {
                        Ok(datetime) => { Some(datetime.clone()) }
                        _ => None,
                    }
                }
            },
            active: true,
            modified: false,
        }
    }
}