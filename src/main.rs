mod account;
mod transaction;

use account::Account;
use transaction::{iter_over_file, Operation, Transaction};

use clap::Parser;
use csv::Writer;
use std::collections::HashMap;
use std::error::Error;
use rust_decimal::Decimal;

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

// We store the accounts in a "database" implemented which is just a hashmap of client ID to Account.
type AccountDb = HashMap<u16, Account>;

// Store deposits in a "database" implemented as a hashmap of tx ID -> amount.
type DepositDb = HashMap<u32, Decimal>;

/// Process a single transaction record. Returns whether the operation succeeded or not.
fn handle_record(
    tx: &Transaction,
    accounts: &mut AccountDb,
    deposits: &mut DepositDb,
) -> Result<(), Box<dyn Error>> {
    let account = accounts.entry(tx.client).or_insert(Account::new(tx.client));

    match tx.op {
        Operation::Deposit(amount) => {
            if deposits.contains_key(&tx.id) {
                return Err(format!("Already have a transaction with ID {}", tx.id).into());
            }
            deposits.insert(tx.id, amount);
            account.deposit(amount)
        }
        Operation::Withdrawal(amount) => account.withdraw(amount),
        Operation::Dispute => {
            let amount = *deposits
                .get(&tx.id)
                .ok_or(format!("no transaction with ID {}", tx.id))?;
            account.dispute(tx.id, amount)
        }
        Operation::Resolve => account.resolve(tx.id),
        Operation::Chargeback => account.chargeback(tx.id),
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

    // Create a "database" to store the client accounts. In production this would probably be a separate
    // scalable and reliable database. For this problem just use a hashmap.
    let mut account_db: AccountDb = HashMap::new();

    // Create a "database" to store deposits that might be disputed.
    // Again, in production this would be a separate DB, but we'll use a hashmap.
    //
    // NOTE: It is unclear from the problem statement if withdrawals can also be disputed. Realistically it seems
    // like they could be. But the description for dispute handling suggests it only covers deposits. I've
    // assumed we only need to handle desposits.
    let mut deposit_db: DepositDb = HashMap::new();

    for tx in iter_over_file(args.tx_log.as_str())? {
        // If this fails we want to just skip over the record, ignoring the result.
        let _ = handle_record(&tx, &mut account_db, &mut deposit_db);
    }

    display_accounts(&account_db)?;

    Ok(())
}
