use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::{Arc, Mutex}};
use accounts::{CheckingAccount, AccountType, Account, InvestmentAccount};
use serde::{Deserialize, Serialize};
pub mod stock;
mod transactions;
pub mod accounts;

/// A bank that holds accounts
/// It does nothing as of now, but hold accounts
/// 
/// TODO: Add interest rates, fees, etc.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bank{
    checking_accounts: HashMap<u32, accounts::CheckingAccount>,
    investment_accounts: HashMap<u32, accounts::InvestmentAccount>,
}

impl Bank{
    /// Creates a new bank with the given accounts
    pub fn new(accounts: HashMap<u32, CheckingAccount>) -> Self{
        Bank{
            checking_accounts: accounts,
            investment_accounts: HashMap::<u32, InvestmentAccount>::new()
        }
    }

    /// Creates a new bank with no accounts
    pub fn empty() -> Self{
        Bank{
            checking_accounts: HashMap::new(),
            investment_accounts: HashMap::<u32, InvestmentAccount>::new()
        }
    }

    /// Opens a new account, with an optional nickname
    pub fn open_account(&mut self, nickname: Option<String>, account_type: AccountType) -> Result<u32, error::BankError>{
        // find the highest id
        match account_type{
            AccountType::Checking => {
                let id = self.checking_accounts.keys().max().unwrap_or(&0) + 1;
                let account = CheckingAccount::new(id, 0.0, nickname);
                self.checking_accounts.insert(id, account);
                Ok(id)
            },
            AccountType::Investment => {
                let id = self.investment_accounts.keys().max().unwrap_or(&0) + 1;
                let account = InvestmentAccount::new(id, 0.0, nickname);
                self.investment_accounts.insert(id, account);
                Ok(id)
            },
        }
    }

    /// Closes an account
    pub fn close_account(&mut self, id: u32) -> Result<(), error::BankError>{
        if !self.checking_accounts.contains_key(&id){
            return Err(error::BankError::AccountNotFound);
        }
        if self.checking_accounts.get(&id).unwrap().get_balance() != 0.0{
            return Err(error::BankError::CloseAccountWithBalance);
        }
        self.checking_accounts.remove(&id);
        Ok(())
    }

    pub async fn save(&self, path: &str) -> Result<(), std::io::Error>{
        let json = serde_json::to_string(self)?;
        tokio::fs::write(path, json).await
    }

    pub fn get_investment_account(&self, id: u32) -> Result<&InvestmentAccount, error::BankError>{
        if let Some(account) = self.investment_accounts.get(&id){
            return Ok(account);
        }
        Err(error::BankError::AccountNotFound)
    }

    pub fn get_investment_account_mut(&mut self, id: u32) -> Result<&mut InvestmentAccount, error::BankError>{
        if let Some(account) = self.investment_accounts.get_mut(&id){
            return Ok(account);
        }
        Err(error::BankError::AccountNotFound)
    }

    pub fn get_checking_account(&self, id: u32) -> Result<&CheckingAccount, error::BankError>{
        if let Some(account) = self.checking_accounts.get(&id){
            return Ok(account);
        }
        Err(error::BankError::AccountNotFound)
    }

    pub fn get_checking_account_mut(&mut self, id: u32) -> Result<&mut CheckingAccount, error::BankError>{
        if let Some(account) = self.checking_accounts.get_mut(&id){
            return Ok(account);
        }
        Err(error::BankError::AccountNotFound)
    }

    pub fn get_investment_accounts(&self) -> &HashMap<u32, InvestmentAccount>{
        &self.investment_accounts
    }

    pub fn get_checking_accounts(&self) -> &HashMap<u32, CheckingAccount>{
        &self.checking_accounts
    }

}

impl From<HashMap<u32, CheckingAccount>> for Bank{
    fn from(accounts: HashMap<u32, CheckingAccount>) -> Self{
        Bank::new(accounts)
    }
}

// into Arc<Mutex<Bank>>
impl Into<Arc<Mutex<Bank>>> for Bank {
    fn into(self) -> Arc<Mutex<Bank>> {
        Arc::new(Mutex::new(self))
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


pub mod error{
    use thiserror::Error;
    use tokio::time::error::Error;

    #[derive(Error, Debug)]
    pub enum BankError{
        #[error("Account not found")]
        AccountNotFound,
        #[error("Account already exists")]
        AccountAlreadyExists,
        #[error("Insufficient funds")]
        InsufficientFunds,
        #[error("Cannot close account with balance")]
        CloseAccountWithBalance,
        #[error("Insufficient quantity of investment")]
        InsufficientQuantity,
        #[error("Tokio error: {0}")]
        OtherTokio(tokio::io::Error),
        #[error("AlphaVantage error: {0}")]
        OtherAlphaVantage(alphavantage::error::Error),
        #[error("Other error: {0}")]
        Other(String),
    }
}

#[cfg(test)]
mod tests{
    use stock::Holding;

    use super::*;
    use std::{rc::Rc, str::FromStr};

    #[test]
    fn test_bank(){
        let mut bank = Bank::empty();
        let id = bank.open_account(None, AccountType::Checking).unwrap();
        let account = bank.checking_accounts.get(&id).unwrap();
        assert_eq!(account.get_id(), id);
        assert_eq!(account.get_balance(), 0.0);
        assert_eq!(account.get_nickname(), None);
    }

    #[test]
    fn test_bank_from_str(){
        let json = r#"{"checking_accounts":{}, "investment_accounts":{}}"#;
        let bank = Bank::from_str(json).unwrap();
        assert_eq!(bank.checking_accounts.len(), 0);
        assert_eq!(bank.investment_accounts.len(), 0);
    }

    #[test]
    fn test_bank_save(){
        let mut bank = Bank::empty();
        let id = bank.open_account(Some("Nickname".to_string()), AccountType::Checking).unwrap();
        let account = bank.checking_accounts.get(&id).unwrap();
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
        assert_eq!(bank2.checking_accounts.len(), 1);
        let account2 = bank2.checking_accounts.get(&id).unwrap();
        assert_eq!(account2.get_id(), id);
        assert_eq!(account2.get_balance(), 0.0);
        assert_eq!(account2.get_nickname(), Some("Nickname".to_string()));
    }

    #[test]
    fn test_complex_bank_save(){
        let mut bank = Bank::empty();
        let id = bank.open_account(Some("Nickname".to_string()), AccountType::Checking).unwrap();
        let account = bank.checking_accounts.get_mut(&id).unwrap();
        account.deposit(10.0);
        assert_eq!(account.get_balance(), 10.0);
        // 2nd checking
        let id2 = bank.open_account(Some("Nickname2".to_string()), AccountType::Checking).unwrap();
        let account = bank.checking_accounts.get_mut(&id2).unwrap();
        account.deposit(20.0);
        assert_eq!(account.get_balance(), 20.0);
        // open investment
        let id3 = bank.open_account(Some("Investment".to_string()), AccountType::Investment).unwrap();
        let account = bank.investment_accounts.get_mut(&id3).unwrap();
        account.deposit(40.0);
        assert_eq!(account.get_balance(), 40.0);
        // purchase stock
        account.purchase_investment("AAPL".to_string(), 10., 2.).unwrap();
        assert_eq!(account.get_balance(), 20.0);
        assert_eq!(account.get_investments().len(), 1);
        // purchase second stock
        account.purchase_investment("GOOGL".to_string(), 20., 1.).unwrap();
        assert_eq!(account.get_balance(), 0.0);
        assert_eq!(account.get_investments().len(), 2);


        let path = "tests/test_complex_bank_save.json";
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
            bank.save(path).await
        }).unwrap();

        let json = std::fs::read_to_string(path).unwrap();
        let bank2 = Bank::from_str(&json).unwrap();
        
        // check checking accounts
        assert_eq!(bank2.checking_accounts.len(), 2);
        let account2 = bank2.checking_accounts.get(&id).unwrap();
        assert_eq!(account2.get_id(), id);
        assert_eq!(account2.get_balance(), 10.0);
        assert_eq!(account2.get_nickname(), Some("Nickname".to_string()));
        let account2 = bank2.checking_accounts.get(&id2).unwrap();
        assert_eq!(account2.get_id(), id2);
        assert_eq!(account2.get_balance(), 20.0);
        assert_eq!(account2.get_nickname(), Some("Nickname2".to_string()));
        // check investment accounts
        assert_eq!(bank2.investment_accounts.len(), 1);
        let account2 = bank2.investment_accounts.get(&id3).unwrap();
        assert_eq!(account2.get_id(), id3);
        assert_eq!(account2.get_balance(), 0.);
        assert_eq!(account2.get_nickname(), Some("Investment".to_string()));
        assert_eq!(account2.get_investments().len(), 2);
        let holding = account2.get_investments().get("AAPL").unwrap();
        assert_eq!(holding.quantity, 2.);
        assert_eq!(holding.average_cost_per_unit, 10.);
        let holding = account2.get_investments().get("GOOGL").unwrap();
        assert_eq!(holding.quantity, 1.);
        assert_eq!(holding.average_cost_per_unit, 20.);

    }

    #[test]
    fn test_close_account(){
        let mut bank = Bank::empty();
        let id = bank.open_account(None, AccountType::Checking).unwrap();
        assert!(bank.close_account(id).is_ok());
        assert!(bank.close_account(id).is_err());

        let id = bank.open_account(None, AccountType::Checking).unwrap();
        let account = bank.checking_accounts.get_mut(&id).unwrap();
        account.deposit(10.0);
        assert!(bank.close_account(id).is_err());
    }

}
