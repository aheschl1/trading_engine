use alphavantage::{time_series::IntradayInterval, cache_enabled::client::Client};
use crate::bank::{self, Bank, accounts::Account};

pub struct Broker<'a> {
    client: &'a Client,
    bank: Bank
}

impl<'a> Broker<'a> {
    pub fn new(client: &'a Client, bank: Bank) -> Self {
        Broker {
            client,
            bank
        }
    }

    pub fn get_client(&self) -> &Client {
        self.client
    }

    pub fn get_bank(&self) -> &Bank {
        &self.bank
    }

    pub fn get_bank_mut(&mut self) -> &mut Bank {
        &mut self.bank
    }

    /// Buys a stock with the given symbol and quantity for the given account
    /// The price is the closing price of the most recent minute
    /// 
    /// # Errors
    /// 
    /// Returns an error if the symbol is invalid or the account does not have enough funds
    /// 
    /// # Returns
    /// 
    /// Returns the new balance of the account
    pub async fn buy(&mut self, symbol: &str, quantity: f64, account_id: u32) -> Result<f64, bank::error::BankError>{
        let price = self.client.get_time_series_intraday(symbol, IntradayInterval::OneMinute).await
            .map_err(|e| bank::error::BankError::OtherTokio(e))?
            .map_err(|e| bank::error::BankError::OtherAlphaVantage(e))?
            .entries
            .last().unwrap()
            .close;
        
        let account = self.bank.get_investment_account_mut(account_id)?;
        account.purchase_investment(symbol.to_string(), quantity, price)?;

        Ok(account.get_balance())
    }
}