use std::sync::Arc;

use alphavantage::{cache_enabled::{client::Client, tickers::{SearchResults, Entry}, time_series::{self, TimeSeries}}, time_series::IntradayInterval};
use chrono::{format::Fixed, DateTime, FixedOffset};
use tokio::sync::Mutex;
use crate::bank::{self, accounts::Account, error::BankError, Bank};

pub struct Broker {
    client: Client,
    bank: Arc<Mutex<Bank>>
}

impl Broker {
    pub fn new<T>(client: Client, bank: T) -> Self
    where
        T: Into<Arc<Mutex<Bank>>>,
    {
        Broker {
            client,
            bank: bank.into(),
        }
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub fn get_bank(&self) -> Arc<Mutex<Bank>> {
        self.bank.clone()
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

    pub async fn get_time_series_daily_full(&self, symbol: &str) ->Result<TimeSeries, bank::error::BankError>
    {
        self.client.get_time_series_daily_full(symbol).await
            .map_err(|e| bank::error::BankError::OtherTokio(e))?
            .map_err(|e| bank::error::BankError::OtherAlphaVantage(e))
    }

    pub async fn get_time_series_weekly_full(&self, symbol: &str) ->Result<TimeSeries, bank::error::BankError>
    {
        self.client.get_time_series_weekly_adjusted_full(symbol).await
            .map_err(|e| bank::error::BankError::OtherTokio(e))?
            .map_err(|e| bank::error::BankError::OtherAlphaVantage(e))
    }

    pub async fn get_time_series_monthly_full(&self, symbol: &str) ->Result<TimeSeries, bank::error::BankError>
    {
        self.client.get_time_series_monthly_adjusted_full(symbol).await
            .map_err(|e| bank::error::BankError::OtherTokio(e))?
            .map_err(|e| bank::error::BankError::OtherAlphaVantage(e))
    }

    /// Gets the price of a stock with the given symbol
    /// The price is the closing price of the most recent minute
    /// 
    /// # Errors
    /// 
    /// Returns an error if the symbol is invalid
    /// 
    /// # Returns
    /// 
    /// Returns the price of the stock
    pub async fn get_price(&self, symbol: &str, date_limit: Option<DateTime<FixedOffset>>) -> Result<f64, bank::error::BankError> {
        let price = self.get_time_series_intraday(symbol, IntradayInterval::FiveMinutes).await?
            .entries
            .iter()
            .filter(|entry|{
                if let Some(date_limit) = date_limit {
                    entry.date <= date_limit
                } else {
                    true
                }
            })
            .collect::<Vec<&time_series::Entry>>()
            .last()
            .ok_or_else(||BankError::Other(String::from("Cannot find price")))
            .map(|entry| entry.adjusted_close.unwrap_or(entry.close))?;

        Ok(price)
    }

    /// Buys a stock with the given symbol and quantity for the given account
    /// The price is the closing price of the most recent minute
    /// 
    /// # Arguments
    /// 
    /// * `symbol` - The symbol of the stock to buy
    /// * `quantity` - The quantity of the stock to buy
    /// * `account_id` - The id of the account to buy the stock from
    /// * `date_limit` - The date limit to get the price of the stock
    /// 
    /// # Errors
    /// 
    /// Returns an error if the symbol is invalid or the account does not have enough funds
    /// 
    /// # Returns
    /// 
    /// Returns the new balance of the account
    pub async fn buy(&mut self, symbol: &str, quantity: f64, account_id: u32, date_limit: Option<DateTime<FixedOffset>>) -> Result<f64, bank::error::BankError>{
        let price = self.get_price(symbol, date_limit).await?;
        
        let mut bank = self.bank
            .lock()
            .await;

        let account = bank.get_investment_account_mut(account_id)?;
        account.purchase_investment(symbol.to_string(), price, quantity)?;

        Ok(account.get_balance())
    }


    /// Query for a list of ticker symbols that match the given query
    pub async fn get_tickers(&self, query: &str) -> Result<SearchResults, bank::error::BankError> {
        self.client.get_tickers(query).await
            .map_err(|e| bank::error::BankError::OtherTokio(e))?
            .map_err(|e| bank::error::BankError::OtherAlphaVantage(e))
    }

    /// Gets the current value of the given quantity of the stock with the given symbol
    /// The price is the closing price of the most recent minute, before the given date limit
    pub async fn get_current_value(&self, symbol: &str, quantity: f64, date_limit: Option<DateTime<FixedOffset>>) -> Result<f64, bank::error::BankError> {
        let price = self.get_price(symbol, date_limit).await?;
        Ok(price * quantity)
    }
}