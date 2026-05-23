use std::error::Error;
use std::io::{self, Write};

mod assets_service;
mod database_builder;
mod models;
mod schema;

use crate::assets_service::download_assets;
use crate::database_builder::create_database;

fn main() -> Result<(), Box<dyn Error>> {
    user_decision()?;
    Ok(())
}

fn user_decision() -> Result<(), Box<dyn Error>> {
    print!("Select an option:\n1. Download database assets\n2. Build database\n\n> ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    match input.trim() {
        "1" => {
            println!("Downloading assets...");
            download_assets()?;
            println!("Done!");
            Ok(())
        }
        "2" => {
            println!("Building database...");
            create_database()?;
            println!("Done!");
            Ok(())
        }
        _ => {
            println!("Invalid option. Please select 1 or 2.");
            Ok(())
        }
    }
}
