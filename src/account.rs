use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use rust_decimal::Decimal;

/// A structure represening a single user account.
#[derive(Default)]
pub struct Account {
    client: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,

    disputes: HashMap<u32, Decimal>,
}

/// A structure containing the details for how to display an account. This is a separate
/// struct as there are some fields on the main account that we don't want to display (such as
/// active disputes), and there is some information we want to display that is not directly
/// stored in the account (e.g. total balance).
#[derive(Debug, Serialize)]
pub struct AccountDisplay {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
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
    pub fn total_balance(&self) -> Decimal {
        self.available + self.held
    }

    /// Deposit funds into the user's account.
    pub fn deposit(&mut self, amount: Decimal) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;
        self.available += amount;
        Ok(())
    }

    /// Withdraw funds from the account, returning an error if there are insufficient funds.
    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;

        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            Err("Insufficeint funds".into())
        }
    }

    pub fn dispute(&mut self, tx_id: u32, amount: Decimal) -> Result<(), Box<dyn Error>> {
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

        let amount = self
            .disputes
            .get(&tx_id)
            .ok_or(format!("could not find dispute with TX ID {}", tx_id))?;
        self.available += amount;
        self.held -= amount;
        Ok(())
    }

    pub fn chargeback(&mut self, tx_id: u32) -> Result<(), Box<dyn Error>> {
        self.fail_if_locked()?;

        let amount = self
            .disputes
            .get(&tx_id)
            .ok_or(format!("could not find dispute with TX ID {}", tx_id))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_deposit_withdrawal() {
        let mut acc = Account::new(1);
        assert!(acc.deposit(dec!(1.0)).is_ok());
        assert!(acc.deposit(dec!(2.0)).is_ok());
        assert!(acc.withdraw(dec!(1.2)).is_ok());

        assert_eq!(acc.available, dec!(1.8));
        assert_eq!(acc.held, dec!(0.0));
        assert_eq!(acc.total_balance(), dec!(1.8));
    }

    #[test]
    fn test_dispute_resolve() {
        let mut acc = Account::new(1);

        assert!(acc.deposit(dec!(1.0)).is_ok());
        assert!(acc.deposit(dec!(2.0)).is_ok());
        assert!(acc.dispute(33, dec!(1.2)).is_ok());

        assert_eq!(acc.available, dec!(1.8));
        assert_eq!(acc.held, dec!(1.2));
        assert_eq!(acc.total_balance(), dec!(3.0));

        assert!(acc.resolve(33).is_ok());
        assert_eq!(acc.available, dec!(3.0));
        assert_eq!(acc.held, dec!(0.0));
        assert_eq!(acc.total_balance(), dec!(3.0));
    }

    #[test]
    fn test_dispute_chargeback() {
        let mut acc = Account::new(1);

        assert!(acc.deposit(dec!(1.0)).is_ok());
        assert!(acc.deposit(dec!(2.0)).is_ok());
        assert!(acc.dispute(33, dec!(1.2)).is_ok());
        assert!(acc.chargeback(33).is_ok());

        assert_eq!(acc.available, dec!(1.8));
        assert_eq!(acc.held, dec!(0.0));
        assert_eq!(acc.total_balance(), dec!(1.8));

        // Further transactions fail. 
        assert!(acc.deposit(dec!(1.0)).is_err());
        assert!(acc.withdraw(dec!(1.0)).is_err());
        assert!(acc.dispute(66, dec!(1.0)).is_err());
        assert!(acc.resolve(66).is_err());
    }

    #[test]
    fn test_resolve_unrecognized_dispute() {
        let mut acc = Account::new(1);

        assert!(acc.deposit(dec!(1.0)).is_ok());
        assert!(acc.deposit(dec!(2.0)).is_ok());
        assert!(acc.dispute(33, dec!(1.2)).is_ok());
        assert!(acc.resolve(36).is_err());
    }

    #[test]
    fn test_multiple_disputes() {
        let mut acc = Account::new(1);

        assert!(acc.deposit(dec!(1.0)).is_ok());
        assert!(acc.deposit(dec!(2.0)).is_ok());
        assert!(acc.dispute(33, dec!(1.2)).is_ok());
        assert!(acc.dispute(66, dec!(1.0)).is_ok());

        assert_eq!(acc.available, dec!(0.8));
        assert_eq!(acc.held, dec!(2.2));
        assert_eq!(acc.total_balance(), dec!(3.0));

        // Resolve the second dispute
        assert!(acc.resolve(66).is_ok());
        assert_eq!(acc.available, dec!(1.8));
        assert_eq!(acc.held, dec!(1.2));
        assert_eq!(acc.total_balance(), dec!(3.0));

        // Chargeback the first
        assert!(acc.chargeback(33).is_ok());
        assert_eq!(acc.available, dec!(1.8));
        assert_eq!(acc.held, dec!(0.0));
        assert_eq!(acc.total_balance(), dec!(1.8));
    }
}