use serde::Serialize;
use std::error::Error;

/// A structure represening a single user account.
#[derive(Default)]
pub struct Account {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub locked: bool,
}

/// A structure containing the details for how to display an account. This is a separate
/// struct as there are some fields on the main account that we don't want to display (such as
/// active disputes), and there is some information we want to display that is not directly
/// stored in the account (e.g. total balance).
#[derive(Debug, Serialize)]
pub struct AccountDisplay {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Account {
    /// Create a new account for the specified user.
    pub fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    /// Calculate the user's total balance.
    pub fn total_balance(&self) -> f64 {
        self.available + self.held
    }

    /// Deposit funds into the user's account.
    pub fn deposit(&mut self, amount: f64) {
        self.available += amount;
    }

    /// Withdraw funds from the account, returning an error if there are insufficient funds.
    pub fn withdraw(&mut self, amount: f64) -> Result<(), Box<dyn Error>> {
        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            Err("Insufficeint funds".into())
        }
    }

    /// Create a display representation for this account.
    pub fn to_display(&self) -> AccountDisplay {
        AccountDisplay {
            client: self.client,
            available: self.available,
            held: self.held,
            total: self.total_balance(),
            locked: self.locked,
        }
    }
}
