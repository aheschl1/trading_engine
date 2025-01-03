use std::collections::HashMap;
use account::Account;
use serde::{Deserialize, Serialize};

/// A bank that holds accounts
/// It does nothing as of now, but hold accounts
/// 
/// TODO: Add interest rates, fees, etc.
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

    pub async fn save(&self, path: &str) -> Result<(), std::io::Error>{
        let json = serde_json::to_string(self)?;
        tokio::fs::write(path, json).await
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

// to str
impl std::string::ToString for Bank{
    fn to_string(&self) -> String{
        serde_json::to_string(self).unwrap()
    }
}

pub(crate) mod account{
    use serde::{de::value::Error, Deserialize, Serialize};
    use chrono;

    use super::error;

    
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Account{
        id: u32,
        balance: f64,
        nickname: Option<String>,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    impl Account{
        /// Creates a new account with the given id, balance, and optional nickname
        /// Will set the created_at field to the current time
        /// 
        /// # Arguments
        /// 
        /// * `id` - The id of the account
        /// * `balance` - The balance of the account
        /// * `nickname` - An optional nickname for the account
        pub fn new(id: u32, balance: f64, nickname: Option<String>) -> Self{
            Account{
                id: id,
                balance: balance,
                nickname: nickname,
                created_at: chrono::Utc::now(),
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

        pub fn get_created_at(&self) -> chrono::DateTime<chrono::Utc>{
            self.created_at
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

#[cfg(test)]
mod tests{
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_bank(){
        let mut bank = Bank::empty();
        let id = bank.open_account(None).unwrap();
        let account = bank.accounts.get(&id).unwrap();
        assert_eq!(account.get_id(), id);
        assert_eq!(account.get_balance(), 0.0);
        assert_eq!(account.get_nickname(), None);
    }

    #[test]
    fn test_bank_from_str(){
        let json = r#"{"accounts":{}}"#;
        let bank = Bank::from_str(json).unwrap();
        assert_eq!(bank.accounts.len(), 0);
    }

    #[test]
    fn test_bank_save(){
        let mut bank = Bank::empty();
        let id = bank.open_account(Some("Nickname".to_string())).unwrap();
        let account = bank.accounts.get(&id).unwrap();
        assert_eq!(account.get_id(), id);
        assert_eq!(account.get_balance(), 0.0);
        assert_eq!(account.get_nickname(), Some("Nickname".to_string()));

        let path = "tests/test_bank_save.json";
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
            bank.save(path).await
        }).unwrap();

        let json = std::fs::read_to_string(path).unwrap();
        let bank2 = Bank::from_str(&json).unwrap();
        assert_eq!(bank2.to_string(), bank.to_string());
        assert_eq!(bank2.accounts.len(), 1);
        let account2 = bank2.accounts.get(&id).unwrap();
        assert_eq!(account2.get_id(), id);
        assert_eq!(account2.get_balance(), 0.0);
        assert_eq!(account2.get_nickname(), Some("Nickname".to_string()));
    }

    #[test]
    fn test_bank_add_account(){
        let mut bank = Bank::empty();
        let now = chrono::Utc::now();
        let account = Account::new(1, 0.0, None);
        assert_eq!(account.get_created_at().timestamp(), now.timestamp());
        bank.add_account(account.clone()).unwrap();
        assert_eq!(bank.accounts.len(), 1);
        assert_eq!(bank.accounts.get(&1).unwrap().get_id(), 1);
        assert_eq!(bank.accounts.get(&1).unwrap().get_created_at(), account.get_created_at());
    }

    #[test]
    fn test_deposit(){
        let mut account = account::Account::new(1, 0.0, None);
        assert_eq!(account.deposit(10.0), 10.0);
        assert_eq!(account.deposit(5.0), 15.0);
    }

    #[test]
    fn test_withdraw(){
        let mut account = account::Account::new(1, 10.0, None);
        assert_eq!(account.withdraw(5.0).unwrap(), 5.0);
        assert_eq!(account.withdraw(5.0).unwrap(), 0.0);
        assert!(account.withdraw(1.0).is_err());
    }
}
