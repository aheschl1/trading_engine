use std::{collections::HashMap, sync::Arc};

use alphavantage::{cache_enabled::{client::Client, tickers::{Entry, SearchResults}, time_series::{self, TimeSeries}}, corprate_actions::{DividendEntry, DividendResults}, time_series::IntradayInterval};
use chrono::{format::Fixed, DateTime, FixedOffset, TimeZone, Utc};
use disk_cache::cache_async;
use tokio::sync::Mutex;
use utils::expand_tilde;
use crate::bank::{self, accounts::Account, error::BankError, stock::Asset, transactions::{Transaction, TransactionType}, Bank};

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

    /// Gets the ticker for the given symbol
    /// 
    /// # Errors
    /// 
    /// Returns an error if the symbol is invalids
    #[cache_async(cache_root = "~/.cache/trading_engine/get_ticker/{symbol}", invalidate_rate = 1210000)]
    pub async fn get_ticker(&self, symbol: String) -> Result<Entry, bank::error::BankError> {
        let tickers: Vec<Entry> = self.get_tickers(&symbol).await?
            .entries
            .into_iter()
            .filter(|entry| entry.symbol == symbol)
            .collect();
        if tickers.len() != 1 {
            return Err(BankError::Other(String::from("No ticker found")));
        }
        Ok(tickers[0].clone())
    }

    async fn is_market_open(&self, symbol: &str, date_limit: DateTime<FixedOffset>) -> Result<bool, BankError> {
        let ticker = self.get_ticker(symbol.to_string()).await.map_err(|e| BankError::OtherTokio(e))??;
        let market_open = ticker.market_open;
        let market_close = ticker.market_close;
        // now, make sure it is currently trading, based on date_limit
        if date_limit.time() < market_open || date_limit.time() > market_close {
            return Ok(false)
        }
        Ok(true)
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
    /// Returns an error if the symbol is invalid, the account does not have enough funds, or the market is closed
    /// 
    /// # Returns
    /// 
    /// Returns the new balance of the account
    pub async fn buy(&mut self, symbol: &str, quantity: f64, account_id: u32, date_limit: Option<DateTime<FixedOffset>>) -> Result<f64, bank::error::BankError>{
        let price = self.get_price(symbol, date_limit).await?;
        match self.is_market_open(symbol, date_limit.unwrap_or(chrono::Utc::now().into())).await?{
            true => {},
            false => return Err(BankError::Other(String::from("Market is closed"))),
        }
        let mut bank = self.bank
            .lock()
            .await;

        let account = bank.get_investment_account_mut(account_id)?;
        account.purchase_investment(symbol.to_string(), price, quantity)?;

        Ok(account.get_balance())
    }

    /// Sell a stock
    /// 
    /// # Arguments
    /// 
    /// * `symbol` - The symbol of the stock to sell
    /// * `quantity` - The quantity of the stock to sell
    /// * `account_id` - The id of the account to sell the stock from
    /// * `date_limit` - The date limit to get the price of the stock
    /// 
    /// # Errors
    /// 
    /// Returns an error if the symbol is invalid or the account does not have enough shares
    /// 
    /// # Returns
    /// 
    /// Returns the new balance of the account
    pub async fn sell(&mut self, symbol: &str, quantity: f64, account_id: u32, date_limit: Option<DateTime<FixedOffset>>) -> Result<f64, bank::error::BankError>{
        let price = self.get_price(symbol, date_limit).await?;
        
        let mut bank = self.bank
            .lock()
            .await;

        let account = bank.get_investment_account_mut(account_id)?;
        account.sell_investment(symbol.to_string(), price, quantity)?;

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

    async fn load_dividend_data(&self, symbol: &str) -> Result<DividendResults, bank::error::BankError> {
        self.client.get_dividend_data(symbol).await
            .map_err(|e| bank::error::BankError::OtherTokio(e))?
            .map_err(|e| bank::error::BankError::OtherAlphaVantage(e))
    }

    /// Checks for the earliest dividend payment date for all accounts
    /// This date is used to check for dividend payments.
    /// If we have never checked for a dividend before, then the earliest date is the simulation date
    /// This is because that entails that the service has never been used, thus, no assets are owned before now.
    /// 
    /// If we have checked for dividends before, then the earliest date is the last time we looked.
    async fn check_earliest_dividend(&self, date_time: DateTime<FixedOffset>) -> Result<DateTime<FixedOffset>, tokio::io::Error> {
        let cache_path = expand_tilde("~/.cache/trading_engine/earliest_dividend_date.json");
        // if the file does not exist, then we have never checked for dividends before
        if !cache_path.exists() {
            // write NOW
            let date = date_time.to_rfc3339();
            tokio::fs::write(cache_path, date).await?;
            return Ok(date_time);
        }
        // file exists, load the date time
        let date = tokio::fs::read_to_string(cache_path).await.unwrap();
        let date = DateTime::parse_from_rfc3339(&date)
            .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, e))?;
        Ok(date)
    }

    /// Checks for dividend payments for all accounts, and all assets, at a given date.
    /// If the date is not provided, it will use the current date
    /// If there is a payment on the date, it will add the payment to all investment accounts
    /// Before adding the payment, we check to make sure the transaction has not already been added
    /// 
    /// # Arguments
    /// 
    /// * `date` - The date to check for dividend payments
    /// 
    /// TODO: Test this somehow
    pub async fn check_dividend_payments(&self, date: Option<DateTime<FixedOffset>>) -> Result<(), bank::error::BankError> {
        let date = date.unwrap_or(chrono::Utc::now().into());
        let last_loaded = 
            self.check_earliest_dividend(date)
            .await
            .map_err(|e| BankError::OtherTokio(e))?;
        let mut valid_dividend_data_memoized : HashMap<String, Vec<DividendEntry>> = HashMap::new();
        // now, we have the last loaded date, we can check for dividend payments
        // only find payments after last loaded, and before or equal to todays date. ignore time.
        // we do last loaded because anything before last loaded it is impossible to have an asset
        let mut bank = self.bank.lock().await;
        for (_, account) in bank.get_investment_accounts_mut().iter_mut() {
            let mut transactions_to_add = Vec::new();
            for (symbol, holding) in account.assets.iter() {
                let valid_dividend_data = match valid_dividend_data_memoized.get(symbol){
                    Some(data) => data,
                    None => {
                        let data = self.parse_valid_dividend_data(symbol, &last_loaded, &date).await?;
                        valid_dividend_data_memoized.insert(symbol.clone(), data.clone());
                        valid_dividend_data_memoized.get(symbol).unwrap()
                    }
                };
                // filter out transactions that are not dividends for the current holding
                let holding_filtered_transactions: Vec<&Transaction> = account.transactions
                    .iter()
                    .filter(|&transaction| {
                        transaction.transaction_type == TransactionType::Dividend(holding.asset.clone(), holding.quantity)
                    })
                    .collect();
                for dividend in valid_dividend_data {
                    // make sure the transaction has not already occurred on the date
                    let dividend_transactions: Vec<&&Transaction> = 
                        holding_filtered_transactions
                        .iter()
                        .filter(|&transaction| {
                            transaction.date.date_naive() == dividend.payment_date.unwrap()
                        })
                        .collect();
                    if dividend_transactions.len() == 0 { // if the transaction has not already occurred
                        let naive_datetime = dividend.payment_date.unwrap().and_hms_opt(0, 0, 0).unwrap();
                        // we need to pay this dividend
                        let payout = holding.quantity.clone() * dividend.amount;
                        let transaction = Transaction::new(
                            TransactionType::Dividend(holding.asset.clone(), holding.quantity.clone()), 
                            payout, 
                            Utc.from_utc_datetime(&naive_datetime), 
                            Some(format!("Dividend payment for {} on {}", symbol, dividend.payment_date.unwrap().to_string()))
                        );
                        transactions_to_add.push(transaction); // add the transaction to todo list
                    }
                }
            }
            // add the transactions
            for transaction in transactions_to_add {
                account.add_transaction(transaction);
            }
        }
        Ok(())
    }

    /**
     * Parse the dividend data to only include dividends that are valid based on when we last loaded them.
     * 
     * For example, if the engine is called 1 day ago, we filter out all dividends that were paid before 1 day ago.
     */
    async fn parse_valid_dividend_data(&self, symbol: &str, last_loaded: &DateTime<FixedOffset>, date: &DateTime<FixedOffset>) -> Result<Vec<DividendEntry>, bank::error::BankError> {
        let valid_dividend_data: Vec<DividendEntry> = 
            self.load_dividend_data(&symbol).await?
            .data
            .into_iter()
            .filter(|dividend| {
                if let Some(dividend_date) = dividend.payment_date {
                    dividend_date > last_loaded.date_naive() && dividend_date <= date.date_naive()
                } else {
                    false
                }
            })
            .collect();
        Ok(valid_dividend_data)
    }
}

mod utils {
    use std::{env, path::PathBuf};

    pub fn expand_tilde(path: &str) -> PathBuf {
        if let Some(home_dir) = env::var_os("HOME") {
            PathBuf::from(path.replacen("~", &home_dir.to_string_lossy(), 1))
        } else {
            PathBuf::from(path) // Fallback to original path if HOME isn't set
        }
    }
    
}

#[cfg(test)]
pub(crate) mod tests{
    use std::env;
    use super::*;

    #[tokio::test]
    async fn test_dividend_check(){
        let client = Client::new(env::var("ALPHAVANTAGE_TOKEN").unwrap().as_str());
        let bank = Arc::new(Mutex::new(Bank::empty()));
        let broker = Broker::new(client, bank);
        let now = chrono::Utc::now();
        broker.check_dividend_payments(Some(now.fixed_offset())).await.unwrap();
    }
}