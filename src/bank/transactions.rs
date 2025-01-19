use serde::{Deserialize, Serialize};

use super::stock::Asset;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// A deposit into the account.
    Deposit,
    /// A withdrawal from the account.
    Withdraw,
    /// A sale of an asset.
    /// The first parameter is the asset that was sold.
    /// The second parameter is the quantity of the asset that was sold.
    Sale(Asset, f64),
    /// A purchase of an asset.
    /// The first parameter is the asset that was purchased.
    /// The second parameter is the quantity of the asset that was purchased.
    Purchase(Asset, f64),
    /// A dividend payment.
    /// The first parameter is the asset that paid the dividend.
    /// The second parameter is the quantity of stock that paid the dividend.
    Dividend(Asset, f64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// The type of transaction.
    pub transaction_type: TransactionType,
    /// The dollar amount of the transaction.
    pub amount: f64,
    /// The date and time of the transaction.
    pub date: chrono::DateTime<chrono::Utc>,
    /// A description of the transaction.
    pub description: Option<String>,
}

impl Transaction {
    pub fn new(transaction_type: TransactionType, amount: f64, date: chrono::DateTime<chrono::Utc>, description: Option<String>) -> Self {
        Transaction {
            transaction_type: transaction_type,
            amount: amount,
            date: date,
            description: description,
        }
    }
}