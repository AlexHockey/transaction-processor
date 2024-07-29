use std::error::Error;
use csv::{Trim, ReaderBuilder};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    tx_log: String,
}

#[derive(Debug, serde::Deserialize)]
struct Transaction {
    #[serde(alias = "type")]
    _type: String, 
    client: u32,
    tx: u32,
    amount: Option<f64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let mut rdr = ReaderBuilder::new()
                    .has_headers(true)
                    .flexible(true)
                    .trim(Trim::All)
                    .from_path(args.tx_log)?;

    for result in rdr.deserialize() {
        let record: Transaction = result?;
        println!("{:?}", record);
    }
    Ok(())
}
