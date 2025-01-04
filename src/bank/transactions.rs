use serde::{Deserialize, Serialize};

use super::stock::Asset;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdraw,
    Sale(Asset, f64),
    Purchase(Asset, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub date: chrono::DateTime<chrono::Utc>,
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