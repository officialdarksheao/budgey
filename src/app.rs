use std::collections::HashMap;
use std::fs;
use std::path::Path;
use calamine::{Data, open_workbook, Reader, Xlsx};
use chrono::Local;
use rust_xlsxwriter::{Format, Workbook};
use crate::items::{record::*, schedule::*};

#[derive(Debug)]
pub enum CurrentScreen {
    Ledger,
    Schedule,
    Exiting,
}
#[derive(Debug)]
pub enum CurrentRecordField {
    Posted,
    Name,
    Amount,
    Balance,
    Category,
    Notes,
    Date,
}
#[derive(Debug)]
pub enum CurrentScheduleField {
    Name,
    Category,
    IntervalType,
    IntervalAmount,
    Amount,
    Start,
    End,
    Active,
}

#[derive(Debug)]
pub enum CurrentlyEditing {
    Record(CurrentRecordField),
    Schedule(CurrentScheduleField),
}

#[derive(Debug)]
pub enum Editable {
    Record(RecordForLedger),
    Schedule(ScheduleForLedger),
}

trait FromRow {
    fn from_row(row: &[Data]) -> Self;
}

impl FromRow for ScheduleForLedger {
    fn from_row(row: &[Data]) -> Self {
        let s = ScheduleForCSV {
            name: row[0].to_string(),
            category: row[1].to_string(),
            interval: row[2].to_string(),
            amount: row[3].to_string(),
            start: row[4].to_string(),
            end: row[5].to_string(),
            id: row[6].to_string().parse().unwrap_or_else(|_| 0),
        };
        ScheduleForLedger::from(s)
    }
}

impl FromRow for RecordForLedger {
    fn from_row(row: &[Data]) -> Self {
        let r = RecordForCSV {
            date: row[0].to_string(),
            posted: row[1].to_string(),
            name: row[2].to_string(),
            debit: row[3].to_string(),
            credit: row[4].to_string(),
            balance: row[5].to_string(),
            category: row[6].to_string(),
            notes: row[7].to_string(),
            id: row[8].to_string().parse().unwrap_or_else(|_| 0),
        };
        RecordForLedger::from(r)
    }
}

#[derive(Debug)]
pub struct App {
    pub csv_path: String,
    pub edit_buffer: Option<Editable>,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,
    pub ledger: HashMap<u32, RecordForLedger>,
    pub schedule: HashMap<u32, ScheduleForLedger>,
    pub ledger_id: u32,
    pub schedule_id: u32,
}

impl App {
    pub fn new() -> App {
        App {
            csv_path: "".to_string(),
            edit_buffer: None,
            current_screen: CurrentScreen::Ledger,
            currently_editing: None,
            ledger: HashMap::new(),
            schedule: HashMap::new(),
            ledger_id: 0,
            schedule_id: 0,
        }
    }

    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::Schedule(field) => {
                    match field {
                        CurrentScheduleField::Name => self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::Category)),
                        CurrentScheduleField::Category =>  self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::IntervalType)),
                        CurrentScheduleField::IntervalType =>  self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::IntervalAmount)),
                        CurrentScheduleField::IntervalAmount =>  self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::Amount)),
                        CurrentScheduleField::Amount =>  self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::Start)),
                        CurrentScheduleField::Start =>  self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::End)),
                        CurrentScheduleField::End =>  self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::Active)),
                        CurrentScheduleField::Active =>  self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::Name)),
                    }
                }
                CurrentlyEditing::Record(field) => {
                    match field {
                        CurrentRecordField::Posted => self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Name)),
                        CurrentRecordField::Name => self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Amount)),
                        CurrentRecordField::Amount => self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Balance)),
                        CurrentRecordField::Balance => self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Category)),
                        CurrentRecordField::Category => self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Notes)),
                        CurrentRecordField::Notes => self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Date)),
                        CurrentRecordField::Date => self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Posted)),
                    }
                }
            }
        } else {
            match &self.current_screen {
                CurrentScreen::Ledger => {
                    self.currently_editing = Some(CurrentlyEditing::Record(CurrentRecordField::Name));
                }
                CurrentScreen::Schedule => {
                    self.currently_editing = Some(CurrentlyEditing::Schedule(CurrentScheduleField::Name));
                }
                CurrentScreen::Exiting => {}
            }
        }
    }

    pub fn try_load(&mut self, path: &str) -> Result<(), String> {
        let mut workbook: Xlsx<_> = open_workbook(path).expect("Cannot Open File");

        self.csv_path = String::from(path);

        if let Ok(sheet) = workbook.worksheet_range("Ledger") {
            self.ledger = HashMap::new();
            self.ledger_id = 0;
            for row in sheet.rows().skip(1) {
                let next = RecordForLedger::from_row(row);
                self.ledger.insert(next.id, next);

            }
        }

        if let Ok(sheet) = workbook.worksheet_range("Schedule") {
            self.schedule = HashMap::new();
            self.schedule_id = 0;
            for row in sheet.rows().skip(1) {
                let next: ScheduleForLedger = ScheduleForLedger::from_row(row);
                self.schedule.insert(next.id, next);
            }
        }

        Ok(())
    }

    pub fn try_save(&mut self) -> std::io::Result<()> {
        let path = Path::new(&self.csv_path);
        if path.exists() && path.extension().and_then(|s| s.to_str()) == Some("xlsx") {
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let parent_dir = path.parent().unwrap_or_else(|| Path::new(""));
            let timestamp = Local::now().format("%m-%d-%Y-%H-%M-%S").to_string();
            let new_file_name = format!("{}_bak_{}.xlsx", file_stem, timestamp);
            let new_file_path = parent_dir.join(new_file_name);
            fs::rename(path, new_file_path)?;
        }

        let bold = Format::new().set_bold();

        let mut workbook = Workbook::new();
        let ledger = workbook.add_worksheet().set_name("Ledger").unwrap();
        ledger.write_string_with_format(0,0,"Date", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,1, "Posted", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,2, "Name", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,3, "Debit", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,4, "Credit", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,5, "Balance", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,6, "Category", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,7, "Notes", &bold).expect("Failed to write");
        ledger.write_string_with_format(0,8, "ID", &bold).expect("Failed to write");

        let mut keys: Vec<_> = self.ledger.keys().collect();
        keys.sort();
        let mut c:u32 = 1;

        for key in keys {
            if let Some(value) = self.ledger.get(key){
                let next = value.to();
                ledger.write_string(c,0, next.date.as_str()).expect("Failed to write");
                ledger.write_string(c,1, next.posted.as_str()).expect("Failed to write");
                ledger.write_string(c,2, next.name.as_str()).expect("Failed to write");
                ledger.write_string(c,3, next.debit.as_str()).expect("Failed to write");
                ledger.write_string(c,4, next.credit.as_str()).expect("Failed to write");
                ledger.write_string(c,5, next.balance.as_str()).expect("Failed to write");
                ledger.write_string(c,6, next.category.as_str()).expect("Failed to write");
                ledger.write_string(c,7, next.notes.as_str()).expect("Failed to write");
                ledger.write_number(c, 8, next.id as f64).expect("Failed to write");

                c = c + 1;
            }
        }

        let schedule = workbook.add_worksheet().set_name("Schedule").unwrap();
        schedule.write_string_with_format(0,0,"Name",&bold).expect("Failed to write");
        schedule.write_string_with_format(0,1,"Category",&bold).expect("Failed to write");
        schedule.write_string_with_format(0,2,"Interval",&bold).expect("Failed to write");
        schedule.write_string_with_format(0,3,"Amount",&bold).expect("Failed to write");
        schedule.write_string_with_format(0,4,"Start",&bold).expect("Failed to write");
        schedule.write_string_with_format(0,5,"End",&bold).expect("Failed to write");
        schedule.write_string_with_format(0,6,"ID",&bold).expect("Failed to write");

        let mut keys: Vec<_> = self.schedule.keys().collect();
        keys.sort();
        let mut c: u32 = 1;

        for key in keys {
            if let Some(value) = self.schedule.get(key) {
                let next = value.to();
                schedule.write_string(c,0,next.name.as_str()).expect("Failed to write");
                schedule.write_string(c,1,next.category.as_str()).expect("Failed to write");
                schedule.write_string(c,2,next.interval.as_str()).expect("Failed to write");
                schedule.write_string(c,3,next.amount.as_str()).expect("Failed to write");
                schedule.write_string(c,4,next.start.as_str()).expect("Failed to write");
                schedule.write_string(c,5,next.end.as_str()).expect("Failed to write");
                schedule.write_number(c,6,next.id as f64).expect("Failed to write");

                c = c + 1;
            }
        }

        workbook.save(self.csv_path.as_str()).expect("Problem Saving");

        Ok(())
    }
}