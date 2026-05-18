use std::error::Error;

mod assets_service;
mod database_builder;
mod models;
mod schema;

use crate::assets_service::{download_file, extract_zip};

fn main() -> Result<(), Box<dyn Error>> {
    // download_file(
    //     "https://raw.githubusercontent.com/LibreOffice/dictionaries/refs/heads/master/pl_PL/th_pl_PL_v2.dat",
    //     "th_pl_PL_v2.dat",
    // )?;
    // download_file(
    //     "https://sjp.pl/sl/odmiany/sjp-odm-20260511.zip",
    //     "sjp-odm.zip",
    // )?;
    // extract_zip("Data/sjp-odm.zip")?;

    let mut builder = database_builder::DatabaseBuilder::new();
    builder.create_database()?;

    Ok(())
}
