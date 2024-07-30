mod transaction;
mod account;

use transaction::{iter_over_file, Record};
use account::Account;

use clap::Parser;
use csv::Writer;
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

    for record in iter_over_file(args.tx_log.as_str())? {
        // If this fails we want to just skip over the record, ignoring the result.
        let _ = handle_record(&record, &mut account_db);
    }

    display_accounts(&account_db)?;

    Ok(())
}
