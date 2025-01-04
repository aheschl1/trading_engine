use std::str::FromStr;

use eframe::egui::ahash::{HashMap, HashMapExt};
use serde::{Deserialize, Serialize};
use chrono;

use super::{error, stock::{self, Holding}, transactions::{self, Transaction}};

pub(crate) enum AccountType{
    Checking,
    Investment,
}

impl PartialEq for AccountType{
    fn eq(&self, other: &Self) -> bool{
        match (self, other){
            (AccountType::Checking, AccountType::Checking) => true,
            (AccountType::Investment, AccountType::Investment) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckingAccount{
    id: u32,
    balance: f64,
    nickname: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    transactions: Vec<Transaction>
}

/// A trait for checking accounts
/// A checking account is a simple account that can be deposited to and withdrawn from
pub trait Account {
    fn get_id(&self) -> u32;
    fn get_balance(&self) -> f64;
    fn get_nickname(&self) -> Option<String>;
    fn deposit(&mut self, amount: f64) -> f64;
    fn withdraw(&mut self, amount: f64) -> Result<f64, error::BankError>;
    fn get_account_type(&self) -> AccountType;
    fn get_created_at(&self) -> chrono::DateTime<chrono::Utc>;
}

impl Account for CheckingAccount{
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
        self.transactions.push(Transaction::new(
            transactions::TransactionType::Deposit,
            amount,
            chrono::Utc::now(),
            None,
        ));
        self.balance
    }

    fn withdraw(&mut self, amount: f64) -> Result<f64, error::BankError>{
        if self.balance < amount{
            return Err(error::BankError::InsufficientFunds);
        }
        self.balance -= amount;
        self.transactions.push(Transaction::new(
            transactions::TransactionType::Withdraw,
            amount,
            chrono::Utc::now(),
            None,
        ));
        Ok(self.balance)
    }

    fn get_created_at(&self) -> chrono::DateTime<chrono::Utc>{
        self.created_at
    }

    fn get_account_type(&self) -> AccountType{
        AccountType::Checking
    }
}

impl CheckingAccount{
    /// Creates a new account with the given id, balance, and optional nickname
    /// Will set the created_at field to the current time
    /// 
    /// # Arguments
    /// 
    /// * `id` - The id of the account
    /// * `balance` - The balance of the account
    /// * `nickname` - An optional nickname for the account
    pub fn new(id: u32, balance: f64, nickname: Option<String>) -> Self{
        CheckingAccount{
            id: id,
            balance: balance,
            nickname: nickname,
            created_at: chrono::Utc::now(),
            transactions: Vec::<Transaction>::new(),
        }
    }

    pub(crate) fn from_checking<T: Account>(account: T) -> Self{
        CheckingAccount{
            id: account.get_id(),
            balance: account.get_balance(),
            nickname: account.get_nickname(),
            created_at: account.get_created_at(),
            transactions: Vec::<Transaction>::new(),
        }
    }
}

// from json string - use serde
impl std::str::FromStr for CheckingAccount{
    type Err = serde_json::Error;

    /// Parses a JSON string into an Account
    fn from_str(s: &str) -> Result<CheckingAccount, Self::Err>{
        serde_json::from_str(s)
    }
}

/// An investment account is a checking account that can also be invested in stocks, bonds, etc.
/// Holds stocks, bonds, etc.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvestmentAccount{
    id: u32,
    balance: f64,
    nickname: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    assets: HashMap<String, Holding>,
    transactions: Vec<Transaction>,
}

impl FromStr for InvestmentAccount{
    type Err = serde_json::Error;

    /// Parses a JSON string into an InvestmentAccount
    fn from_str(s: &str) -> Result<InvestmentAccount, Self::Err>{
        serde_json::from_str(s)
    }
}

/// An investment account can also hold liquid funds, and is thus a checking account
impl Account for InvestmentAccount{

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
        self.transactions.push(Transaction::new(
            transactions::TransactionType::Deposit,
            amount,
            chrono::Utc::now(),
            None,
        ));
        self.balance
    }

    fn withdraw(&mut self, amount: f64) -> Result<f64, error::BankError>{
        if self.balance < amount{
            return Err(error::BankError::InsufficientFunds);
        }
        self.transactions.push(Transaction::new(
            transactions::TransactionType::Withdraw,
            amount,
            chrono::Utc::now(),
            None,
        ));
        self.balance -= amount;
        Ok(self.balance)
    }

    fn get_created_at(&self) -> chrono::DateTime<chrono::Utc>{
        self.created_at
    }

    fn get_account_type(&self) -> AccountType{
        AccountType::Investment
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
            assets: HashMap::new(),
            transactions: Vec::<Transaction>::new(),
        }
    }

    pub fn get_investments(&self) -> &HashMap<String, Holding>{
        &self.assets
    }

    pub fn purchase_investment(&mut self, symbol: String, price: f64, quantity: f64) -> Result<(), error::BankError>{
        // Check if the account has enough balance
        let total_cost = price * quantity;
        if self.balance < total_cost{
            return Err(error::BankError::InsufficientFunds);
        }
        self.balance -= total_cost;
        if let Some(holding) = self.assets.get_mut(symbol.as_str()){
            holding.quantity += quantity;
            holding.total_cost += total_cost;
        }else{
            let holding = Holding::new(total_cost, quantity, symbol.clone());
            self.assets.insert(symbol.clone(), holding);
        }
        // Update the transactions
        let transaction = Transaction::new(
            transactions::TransactionType::Purchase(stock::Asset::new(symbol.clone()), quantity),
            total_cost,
            chrono::Utc::now(),
            None,
        );
        self.transactions.push(transaction);
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
        // check that you actually have the investment
        if !self.assets.contains_key(symbol.as_str()){
            return Err(error::BankError::InsufficientQuantity);
        }
        let holding = self.assets.get_mut(symbol.as_str()).unwrap();
        if holding.quantity < quantity{
            return Err(error::BankError::InsufficientQuantity);
        }
        let total_cost = price * quantity;
        self.balance += total_cost;
        holding.quantity -= quantity;
        holding.total_cost -= total_cost; // TODO: check if this is correct logic
        // Update the transactions
        let transaction = Transaction::new(
            transactions::TransactionType::Sale(stock::Asset::new(symbol.clone()), quantity),
            total_cost,
            chrono::Utc::now(),
            None,
        );
        self.transactions.push(transaction);
        Ok(())
    }

    pub fn from_checking<T: Account>(account: T) -> Self{
        InvestmentAccount{
            id: account.get_id(),
            balance: account.get_balance(),
            nickname: account.get_nickname(),
            created_at: account.get_created_at(),
            assets: HashMap::new(),
            transactions: Vec::<Transaction>::new(),
        }
    }

}