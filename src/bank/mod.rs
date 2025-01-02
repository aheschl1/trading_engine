use std::{collections::HashMap, hash::Hash};

use account::Account;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bank{
    accounts: HashMap<u32, account::Account>,
}

impl Bank{
    /// Creates a new bank with the given accounts
    pub fn new(accounts: HashMap<u32, Account>) -> Self{
        Bank{
            accounts: accounts
        }
    }

    /// Creates a new bank with no accounts
    pub fn empty() -> Self{
        Bank{
            accounts: HashMap::new()
        }
    }

    /// Adds an account to the bank
    pub fn add_account(&mut self, account: Account) -> Result<(), error::BankError>{
        let id = account.get_id();
        if self.accounts.contains_key(&id){
            return Err(error::BankError::AccountAlreadyExists);
        }
        self.accounts.insert(id, account);
        Ok(())
    }

    /// Opens a new account, with an optional nickname
    pub fn open_account(&mut self, nickname: Option<String>) -> Result<u32, error::BankError>{
        // find the highest id
        let id = self.accounts.keys().max().unwrap_or(&0) + 1;
        let account = Account::new(id, 0.0, nickname);
        self.accounts.insert(id, account);
        Ok(id)
    }

    /// Closes an account
    pub fn close_account(&mut self, id: u32) -> Result<(), error::BankError>{
        if !self.accounts.contains_key(&id){
            return Err(error::BankError::AccountNotFound);
        }
        if self.accounts.get(&id).unwrap().get_balance() != 0.0{
            return Err(error::BankError::CloseAccountWithBalance);
        }
        self.accounts.remove(&id);
        Ok(())
    }
}

impl From<HashMap<u32, Account>> for Bank{
    fn from(accounts: HashMap<u32, Account>) -> Self{
        Bank::new(accounts)
    }
}

// from str
impl std::str::FromStr for Bank{
    type Err = serde_json::Error;

    /// Parses a JSON string into a Bank
    fn from_str(s: &str) -> Result<Bank, Self::Err>{
        serde_json::from_str(s)
    }
}

pub(crate) mod account{
    use serde::{de::value::Error, Deserialize, Serialize};

    use super::error;

    
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Account{
        id: u32,
        balance: f64,
        nickname: Option<String>
    }

    impl Account{
        pub fn new(id: u32, balance: f64, nickname: Option<String>) -> Self{
            Account{
                id: id,
                balance: balance,
                nickname: nickname
            }
        }

        pub fn get_id(&self) -> u32{
            self.id
        }

        pub fn get_balance(&self) -> f64{
            self.balance
        }

        pub fn get_nickname(&self) -> Option<String>{
            self.nickname.clone()
        }

        pub fn deposit(&mut self, amount: f64) -> f64{
            self.balance += amount;
            self.balance
        }

        pub fn withdraw(&mut self, amount: f64) -> Result<f64, error::BankError>{
            if self.balance < amount{
                return Err(error::BankError::InsufficientFunds);
            }
            self.balance -= amount;
            Ok(self.balance)
        }
    }

    // from json string - use serde
    impl std::str::FromStr for Account{
        type Err = serde_json::Error;

        /// Parses a JSON string into an Account
        fn from_str(s: &str) -> Result<Account, Self::Err>{
            serde_json::from_str(s)
        }
    }

}

pub mod error{
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum BankError{
        #[error("Account not found")]
        AccountNotFound,
        #[error("Account already exists")]
        AccountAlreadyExists,
        #[error("Insufficient funds")]
        InsufficientFunds,
        #[error("Cannot close account with balance")]
        CloseAccountWithBalance
    }
}