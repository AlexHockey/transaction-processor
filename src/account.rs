use serde::Serialize;
use std::error::Error;
use std::collections::HashMap;

/// A structure represening a single user account.
#[derive(Default)]
pub struct Account {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub locked: bool,

    disputes: HashMap<u32, f64>,
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
    pub fn deposit(&mut self, amount: f64) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;
        self.available += amount;
        Ok(())
    }

    /// Withdraw funds from the account, returning an error if there are insufficient funds.
    pub fn withdraw(&mut self, amount: f64) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;

        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            Err("Insufficeint funds".into())
        }
    }

    pub fn dispute(&mut self, tx_id: u32, amount: f64) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;

        if self.disputes.contains_key(&tx_id) {
            return Err(format!("dispute already in progress for transaction {}", tx_id).into());
        }

        if self.available >= amount {
            self.available -= amount;
            self.held += amount;
            self.disputes.insert(tx_id, amount);
            Ok(())
        } else {
            // Unclear what we should do if there aren't enough funds to hold for the dispute. 
            // I'll assume we can just ignore the transation.
            Err("Insufficeint funds".into())
        }
    }

    pub fn resolve(&mut self, tx_id: u32) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;

        let amount = self.disputes.get(&tx_id).ok_or(format!("could not find dispute with TX ID {}", tx_id))?;
        self.available += amount;
        self.held -= amount;
        Ok(())
    }

    pub fn chargeback(&mut self, tx_id: u32) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;

        let amount = self.disputes.get(&tx_id).ok_or(format!("could not find dispute with TX ID {}", tx_id))?;
        self.held -= amount;
        self.locked = true;
        Ok(())
    }

    // Helper function that returns an Err if the account is locked, which makes checking for this condition easier.
    fn fail_if_locked(&self) -> Result<(), Box<dyn Error>> {
        if self.locked {
            Err(format!("Account {} is locked", self.client).into())
        } else {
            Ok(())
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
