use clap::Parser;
use csv::{ReaderBuilder, Trim, Writer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

/// Program to process a transaction log stored in a CSV file.
///
/// The program applies transactionsi in chronological order and outputs the resulting
/// client account details (including balances).
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the file containing the transaction log
    tx_log: String,
}

/// The representation of a record in the transaction log.
#[derive(Debug, Deserialize)]
struct Record {
    #[serde(alias = "type")]
    _type: String,
    client: u16,
    tx: u32,

    // This field may or may not be present depending on the transaction type
    // (present for deposit or withdrawal, otherwise absent).
    amount: Option<f64>,
}

/// A structure represening a single user account.
#[derive(Default)]
struct Account {
    client: u16,
    available: f64,
    held: f64,
    locked: bool,
}

/// A structure containing the details for how to display an account. This is a separate
/// struct as there are some fields on the main account that we don't want to display (such as
/// active disputes), and there is some information we want to display that is not directly
/// stored in the account (e.g. total balance).
#[derive(Debug, Serialize)]
struct AccountDisplay {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Account {
    /// Create a new account for the specified user.
    fn new(client: u16) -> Self {
        Self {
            client: client,
            ..Default::default()
        }
    }

    /// Calculate the user's total balance.
    fn total_balance(&self) -> f64 {
        self.available + self.held
    }

    /// Deposit funds into the user's account.
    fn deposit(&mut self, amount: f64) {
        self.available += amount;
    }

    /// Withdraw funds from the account, returning an error if there are insufficient funds.
    fn withdraw(&mut self, amount: f64) -> Result<(), Box<dyn Error>> {
        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            Err("Insufficeint funds".into())
        }
    }

    /// Create a display representation for this account.
    fn to_display(&self) -> AccountDisplay {
        AccountDisplay {
            client: self.client,
            available: self.available,
            held: self.held,
            total: self.total_balance(),
            locked: self.locked,
        }
    }
}

/// We store the accounts in a "database" implemented which is just a hashmap of client ID to Account.
type AccountDb = HashMap<u16, Account>;

/// Process a single transaction record. Returns whether the operation succeeded or not.
fn handle_record(record: &Record, db: &mut AccountDb) -> Result<(), Box<dyn Error>> {
    let account = db
        .entry(record.client)
        .or_insert(Account::new(record.client));
    let amount = record.amount.ok_or("No amount value present");

    match record._type.as_str() {
        "deposit" => {
            account.deposit(amount?);
            Ok(())
        }
        "withdrawal" => account.withdraw(amount?),
        _ => Ok(()),
    }
}

/// Display all the stored accounts.
fn display_accounts(db: &AccountDb) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_writer(std::io::stdout());
    for (_, acc) in db.iter() {
        writer.serialize(acc.to_display())?
    }
    Ok(())
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
        // If parsing the transaction log fails there is nothing we can do, so just exit.
        let record: Record = result?;

        // If this fails we want to just skip over the record, ignoring the result.
        let _ = handle_record(&record, &mut account_db);
    }

    display_accounts(&account_db)?;

    Ok(())
}
