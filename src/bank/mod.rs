use std::collections::HashMap;
use account::{Account, Checking};
use serde::{Deserialize, Serialize};
mod stock;

/// A bank that holds accounts
/// It does nothing as of now, but hold accounts
/// 
/// TODO: Add interest rates, fees, etc.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bank{
    checking_accounts: HashMap<u32, account::Account>,
    // investment_accounts: HashMap<u32, account::Account>,
}

impl Bank{
    /// Creates a new bank with the given accounts
    pub fn new(accounts: HashMap<u32, Account>) -> Self{
        Bank{
            checking_accounts: accounts
        }
    }

    /// Creates a new bank with no accounts
    pub fn empty() -> Self{
        Bank{
            checking_accounts: HashMap::new()
        }
    }

    /// Adds an account to the bank
    pub fn add_account(&mut self, account: Account) -> Result<(), error::BankError>{
        let id = account.get_id();
        if self.checking_accounts.contains_key(&id){
            return Err(error::BankError::AccountAlreadyExists);
        }
        self.checking_accounts.insert(id, account);
        Ok(())
    }

    /// Opens a new account, with an optional nickname
    pub fn open_account(&mut self, nickname: Option<String>) -> Result<u32, error::BankError>{
        // find the highest id
        let id = self.checking_accounts.keys().max().unwrap_or(&0) + 1;
        let account = Account::new(id, 0.0, nickname);
        self.checking_accounts.insert(id, account);
        Ok(id)
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
    use eframe::egui::ahash::{HashMap, HashMapExt};
    use serde::{de::value::Error, Deserialize, Serialize};
    use chrono;

    use super::{error, stock::{Holding, StockHolding}};

    
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Account{
        id: u32,
        balance: f64,
        nickname: Option<String>,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    /// A trait for checking accounts
    /// A checking account is a simple account that can be deposited to and withdrawn from
    pub trait Checking {
        fn get_id(&self) -> u32;
        fn get_balance(&self) -> f64;
        fn get_nickname(&self) -> Option<String>;
        fn deposit(&mut self, amount: f64) -> f64;
        fn withdraw(&mut self, amount: f64) -> Result<f64, error::BankError>;
        fn get_created_at(&self) -> chrono::DateTime<chrono::Utc>;
    }

    impl Checking for Account{
        fn get_id(&self) -> u32{
            self.id
        }

        fn get_balance(&self) -> f64{
            self.balance
        }

        fn get_nickname(&self) -> Option<String>{
            self.nickname.clone()
        }

        fn deposit(&mut self, amount: f64) -> f64{
            self.balance += amount;
            self.balance
        }

        fn withdraw(&mut self, amount: f64) -> Result<f64, error::BankError>{
            if self.balance < amount{
                return Err(error::BankError::InsufficientFunds);
            }
            self.balance -= amount;
            Ok(self.balance)
        }

        fn get_created_at(&self) -> chrono::DateTime<chrono::Utc>{
            self.created_at
        }
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
    }

    // from json string - use serde
    impl std::str::FromStr for Account{
        type Err = serde_json::Error;

        /// Parses a JSON string into an Account
        fn from_str(s: &str) -> Result<Account, Self::Err>{
            serde_json::from_str(s)
        }
    }

    /// An investment account is a checking account that can also be invested in stocks, bonds, etc.
    /// Holds stocks, bonds, etc.
    pub struct InvestmentAccount{
        id: u32,
        balance: f64,
        nickname: Option<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        investments: HashMap<String, Vec<StockHolding>>
    }

    /// An investment account can also hold liquid funds, and is thus a checking account
    impl Checking for InvestmentAccount{

        fn get_id(&self) -> u32{
            self.id
        }

        fn get_balance(&self) -> f64{
            self.balance
        }

        fn get_nickname(&self) -> Option<String>{
            self.nickname.clone()
        }

        fn deposit(&mut self, amount: f64) -> f64{
            self.balance += amount;
            self.balance
        }

        fn withdraw(&mut self, amount: f64) -> Result<f64, error::BankError>{
            if self.balance < amount{
                return Err(error::BankError::InsufficientFunds);
            }
            self.balance -= amount;
            Ok(self.balance)
        }

        fn get_created_at(&self) -> chrono::DateTime<chrono::Utc>{
            self.created_at
        }
    }

    impl InvestmentAccount{
        /// Creates a new investment account with the given id, balance, and optional nickname
        /// Will set the created_at field to the current time
        /// 
        /// # Arguments
        /// 
        /// * `id` - The id of the account
        /// * `balance` - The balance of the account
        /// * `nickname` - An optional nickname for the account
        pub fn new(id: u32, balance: f64, nickname: Option<String>) -> Self{
            InvestmentAccount{
                id: id,
                balance: balance,
                nickname: nickname,
                created_at: chrono::Utc::now(),
                investments: HashMap::<String, Vec<StockHolding>>::new()
            }
        }

        pub fn get_investments(&self) -> &HashMap<String, Vec<StockHolding>>{
            &self.investments
        }

        pub fn purchase_investment(&mut self, symbol: String, price: f64, quantity: f64) -> Result<(), error::BankError>{
            let total_cost = price * quantity;
            let _ = self.withdraw(total_cost)?; // Ensure we have enough liquid funds
            // Here, we can afford the investment
            let holding = StockHolding::new(price, quantity, symbol.clone(), chrono::Utc::now());
            if !self.investments.contains_key(&symbol){
                self.investments.insert(symbol.clone(), Vec::<StockHolding>::new());
            }
            self.investments.get_mut(&symbol).unwrap().push(holding);
            Ok(())
        }

        /// Sells an investment
        /// Given a symbol, price, and quantity, will sell the investment if possible
        /// 
        /// # Arguments
        /// 
        /// * `symbol` - The symbol of the investment
        /// * `price` - The price of the investment at the time of sale
        /// * `quantity` - The quantity of the investment to sell
        /// 
        /// # Returns
        /// 
        /// * `Ok(())` - If the investment was sold successfully
        /// * `Err(BankError::InsufficientQuantity)` - If the quantity of the investment is insufficient
        pub fn sell_investment(&mut self, symbol: String, price: f64, quantity: f64) -> Result<(), error::BankError>{
            if !self.investments.contains_key(&symbol){
                return Err(error::BankError::InsufficientQuantity);
            }
            let total_units = self.investments.get(&symbol).unwrap().iter().map(|h| h.get_quantity()).sum::<f64>();
            if total_units < quantity{
                return Err(error::BankError::InsufficientQuantity);
            }
            let holdings = self.investments.get_mut(&symbol).unwrap();
            let mut remaining_quantity = quantity;
            let total_sale = price * quantity;
            while remaining_quantity > 0.0{
                let holding = holdings.pop().unwrap();
                let sale_quantity = if remaining_quantity > holding.get_quantity() {holding.get_quantity()}else{remaining_quantity};
                if sale_quantity < holding.get_quantity(){
                    let new_holding = StockHolding::new(
                        holding.get_price(), 
                        holding.get_quantity() - sale_quantity,
                        holding.get_symbol(), 
                        holding.get_purchase_date()
                    );
                    holdings.push(new_holding);
                }
                remaining_quantity -= sale_quantity;
            }
            // if we sold the last of the symbol, remove the key
            if holdings.len() == 0{
                self.investments.remove(&symbol);
            }
            self.deposit(total_sale);
            Ok(())
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
        CloseAccountWithBalance,
        #[error("Insufficient quantity of investment")]
        InsufficientQuantity,
    }
}

#[cfg(test)]
mod tests{
    use stock::Holding;

    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_bank(){
        let mut bank = Bank::empty();
        let id = bank.open_account(None).unwrap();
        let account = bank.checking_accounts.get(&id).unwrap();
        assert_eq!(account.get_id(), id);
        assert_eq!(account.get_balance(), 0.0);
        assert_eq!(account.get_nickname(), None);
    }

    #[test]
    fn test_bank_from_str(){
        let json = r#"{"checking_accounts":{}}"#;
        let bank = Bank::from_str(json).unwrap();
        assert_eq!(bank.checking_accounts.len(), 0);
    }

    #[test]
    fn test_bank_save(){
        let mut bank = Bank::empty();
        let id = bank.open_account(Some("Nickname".to_string())).unwrap();
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
    fn test_bank_add_account(){
        let mut bank = Bank::empty();
        let now = chrono::Utc::now();
        let account = Account::new(1, 0.0, None);
        assert_eq!(account.get_created_at().timestamp(), now.timestamp());
        bank.add_account(account.clone()).unwrap();
        assert_eq!(bank.checking_accounts.len(), 1);
        assert_eq!(bank.checking_accounts.get(&1).unwrap().get_id(), 1);
        assert_eq!(bank.checking_accounts.get(&1).unwrap().get_created_at(), account.get_created_at());
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

    #[test]
    fn test_close_account(){
        let mut bank = Bank::empty();
        let id = bank.open_account(None).unwrap();
        assert!(bank.close_account(id).is_ok());
        assert!(bank.close_account(id).is_err());

        let id = bank.open_account(None).unwrap();
        let account = bank.checking_accounts.get_mut(&id).unwrap();
        account.deposit(10.0);
        assert!(bank.close_account(id).is_err());
    }

    #[test]
    fn test_investment_account(){
        let mut account = account::InvestmentAccount::new(1, 0.0, None);
        assert_eq!(account.deposit(10.0), 10.0);
        assert_eq!(account.deposit(5.0), 15.0);
        assert_eq!(account.withdraw(5.0).unwrap(), 10.0);
        assert_eq!(account.withdraw(5.0).unwrap(), 5.0);
        assert!(account.withdraw(6.0).is_err());
    }

    #[test]
    fn test_investment_account_purchase(){
        let mut account = account::InvestmentAccount::new(1, 100.0, None);
        account.purchase_investment("AAPL".to_string(), 100.0, 1.0).unwrap();
        assert_eq!(account.get_balance(), 0.0);
        assert_eq!(account.get_investments().len(), 1);
        let holdings = account.get_investments().get("AAPL").unwrap();
        assert_eq!(holdings.len(), 1);
        let holding = holdings.get(0).unwrap();
        assert_eq!(holding.get_price(), 100.0);
        assert_eq!(holding.get_quantity(), 1.0);
        assert_eq!(holding.get_symbol(), "AAPL");
    }

    #[test]
    fn test_investment_account_sell(){
        let mut account = account::InvestmentAccount::new(1, 100.0, None);
        account.purchase_investment("AAPL".to_string(), 100.0, 1.0).unwrap();
        account.sell_investment("AAPL".to_string(), 101.0, 1.0).unwrap();
        assert_eq!(account.get_balance(), 101.0);
        assert_eq!(account.get_investments().len(), 0);
    }
}
