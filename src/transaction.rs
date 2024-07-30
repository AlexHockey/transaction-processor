use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use std::error::Error;

/// The representation of a record in the transaction log.
#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(alias = "type")]
    pub _type: String,
    pub client: u16,
    pub tx: u32,

    // This field may or may not be present depending on the transaction type
    // (present for deposit or withdrawal, otherwise absent).
    pub amount: Option<f64>,
}

pub fn iter_over_file(file_path: &str) -> Result<impl Iterator<Item = Record>, Box<dyn Error>> {
    let rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(Trim::All)
        .from_path(file_path)?;

    let it = rdr.into_deserialize().filter_map(|elem| {
        match elem {
            Ok(rec) => Some(rec), 
            Err(e) => {
                println!("Hit an error {}", e);
                None
            }
        }
    });

    Ok(it)
}