use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use std::error::Error;

/// The representation of a record in the transaction log.
/// Note that this is private to the module and is just used for deserailization.
/// The module converts these to instances of Transaction which are use the type system
/// to ensure correctness.  
#[derive(Debug, Deserialize)]
struct Record {
    #[serde(alias = "type")]
    _type: String,
    client: u16,
    tx: u32,

    /// This field may or may not be present depending on the transaction type
    /// (present for deposit or withdrawal, otherwise absent).
    amount: Option<f64>,
}

/// Struct representing a single transaction. All transactions have a id and reference a client.
/// Some also have type-specific fields.
pub struct Transaction {
    pub id: u32,
    pub client: u16,
    pub op: Operation,
}

/// The different types of operations that transactions can represent, plus any associated data.
pub enum Operation {
    Deposit(f64),
    Withdrawal(f64),
    Dispute,
    Resolve,
    Chargeback,
}

/// Convert a raw record into a transaction.
///
/// Naively this should be possible with serde using an internally tagged enum, but according to
/// https://github.com/BurntSushi/rust-csv/issues/211 this is not supported. So instead implement
/// TryFrom for the conversion.
impl TryFrom<Record> for Transaction {
    type Error = Box<dyn Error>;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        let op = match record._type.as_str() {
            "deposit" => Operation::Deposit(record.amount.ok_or("No amount value present")?),
            "withdrawal" => Operation::Withdrawal(record.amount.ok_or("No amount value present")?),
            "dispute" => Operation::Dispute,
            "resolve" => Operation::Resolve,
            "chargeback" => Operation::Chargeback,
            _ => return Err(format!("Unregognized transaction type {}", record._type).into()),
        };

        Ok(Transaction {
            id: record.tx,
            client: record.client,
            op,
        })
    }
}

/// Iterate over the transancations in a transaction log csv file.
pub fn iter_over_file(
    file_path: &str,
) -> Result<impl Iterator<Item = Transaction>, Box<dyn Error>> {
    // Build a reader.
    // - The CSV has a header we need to strip.
    // - The CSV has variable numbers of columns so we need `flexible` to be set.
    // - The CSV fields contain whitespace which much be stripped.
    let rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(Trim::All)
        .from_path(file_path)?;

    // Deserailize, skipping any errors.
    // TODO: Add logging when encountering errors.
    let it = rdr
        .into_deserialize()
        .filter_map(|elem| elem.ok())
        .filter_map(|rec: Record| rec.try_into().ok());

    Ok(it)
}
