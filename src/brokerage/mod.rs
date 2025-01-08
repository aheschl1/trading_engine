use alphavantage::{cache_enabled::{client::Client, time_series::TimeSeries}, time_series::IntradayInterval};
use crate::bank::{self, Bank, accounts::Account};

pub struct Broker {
    client: Client,
    bank: Bank
}

impl Broker {
    pub fn new(client: Client, bank: Bank) -> Self {
        Broker {
            client,
            bank
        }
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub fn get_bank(&self) -> &Bank {
        &self.bank
    }

    pub fn get_bank_mut(&mut self) -> &mut Bank {
        &mut self.bank
    }

    /// Gets the time series intraday data for the given symbol and interval
    /// 
    /// # Errors
    /// 
    /// Returns an error if the symbol is invalid
    /// 
    /// # Returns
    /// 
    /// Returns the time series data
    pub async fn get_time_series_intraday(&self, symbol: &str, interval: IntradayInterval) ->Result<TimeSeries, bank::error::BankError>
    {
        self.client.get_time_series_intraday(symbol, interval).await
            .map_err(|e| bank::error::BankError::OtherTokio(e))?
            .map_err(|e| bank::error::BankError::OtherAlphaVantage(e))
    }


    /// Gets the price of a stock with the given symbol
    /// The pricce is the closing price of the most recent minute
    /// 
    /// # Errors
    /// 
    /// Returns an error if the symbol is invalid
    /// 
    /// # Returns
    /// 
    /// Returns the price of the stock
    pub async fn get_price(&self, symbol: &str) -> Result<f64, bank::error::BankError> {
        let price = self.get_time_series_intraday(symbol, IntradayInterval::OneMinute).await?
            .entries
            .last().unwrap()
            .close;

        Ok(price)
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
        let price = self.get_price(symbol).await?;
        
        let account = self.bank.get_investment_account_mut(account_id)?;
        account.purchase_investment(symbol.to_string(), quantity, price)?;

        Ok(account.get_balance())
    }
}