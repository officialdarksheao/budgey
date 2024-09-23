extern crate chrono;
extern crate serde;
extern crate csv;
extern crate calamine;
extern crate rust_xlsxwriter;

use std::env;
use app::App;

pub mod app;
pub mod items{
    pub mod schedule;
    pub mod record;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let mut app = App::new();
    match app.try_load(file_path.as_str()) {
        Ok(()) => {
            // start application

            println!("Loaded App!");

            let temp_ledger = app.ledger.clone();

            let mut keys: Vec<_> = temp_ledger.keys().collect();
            keys.sort();
            if keys.len() > 0 {
                let first_key = keys[0].clone();
                let mut balance = match app.ledger.get_mut(&first_key) {
                    None => 0.0,
                    Some(row) => row.balance,
                };
                for key in keys {
                    if let Some(next) = app.ledger.get_mut(key) {
                        next.posted = true;
                        if key != &first_key {
                            balance = balance + next.amount;
                            next.balance = balance;
                        }
                    }
                }
            } else {
                println!("No rows in Ledger!");
            }

            println!("Modifications Done!");

            match app.try_save() {
                Ok(_) => {
                    println!("Saved!");
                }
                Err(m) => {
                    println!("Error Saving! {:?}", m);
                }
            }

            println!("Done!");
        }
        Err(error) => {
            eprintln!("Error, could not load file! {}", error);
            std::process::exit(2);
        }
    }
}
