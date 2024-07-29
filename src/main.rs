use std::error::Error;
use csv::{Trim, ReaderBuilder, Writer};
use clap::Parser;
use std::collections::HashMap;

// Arguments that the program can be run with. 
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    tx_log: String,
}

#[derive(Debug, serde::Deserialize)]
struct Record {
    #[serde(alias = "type")]
    _type: String, 
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

#[derive(Default)]
struct Account {
    client: u16,
    available: f64,
    held: f64,
    locked: bool
}

#[derive(Debug, serde::Serialize)]
struct AccountSummary {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool
}

impl Account {
    fn new(client: u16) -> Self {
        Self { client: client, ..Default::default() }
    }

    fn total_balance(&self) -> f64 {
        self.available + self.held
    }

    fn deposit(&mut self, amount: f64) {
        self.available += amount;
    }

    fn withdraw(&mut self, amount: f64) -> Result<(), Box<dyn Error>> {
        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            Err("Insufficeint funds".into())
        }
    }

    fn to_summary(&self) -> AccountSummary {
        AccountSummary { 
            client: self.client,
            available: self.available,
            held: self.held,
            total: self.total_balance(),
            locked: self.locked,
        }
    }
}

type AccountDb = HashMap<u16, Account>;

fn handle_record(record: &Record, db: &mut AccountDb) -> Result<(), Box<dyn Error>> {
    let account = db.entry(record.client).or_insert(Account::new(record.client));
    let amount = record.amount.ok_or("No amount value present");

    match record._type.as_str() {
        "deposit" => {
            println!("Depositing");
            account.deposit(amount?);
            Ok(())
        },
        "withdrawal" => {
            println!("Withdrawing");
            account.withdraw(amount?)
        },
        _ => Ok(())
    }
}

fn display_accounts(db: &AccountDb) {
    let mut writer = Writer::from_writer(std::io::stdout());
    for (_, acc) in db.iter() {
        writer.serialize(acc.to_summary());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut account_db: AccountDb = HashMap::new();
    
    let mut rdr = ReaderBuilder::new()
                    .has_headers(true)
                    .flexible(true)
                    .trim(Trim::All)
                    .from_path(args.tx_log)?;

    for result in rdr.deserialize() {
        let record: Record = result?;
        handle_record(&record, &mut account_db);
    }

    display_accounts(&account_db);

    Ok(())
}
