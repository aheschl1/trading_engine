mod dataloader;

use alphavantage::{time_series::IntradayInterval, Client};
use crate::bank::Bank;

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

    pub async fn buy(&mut self, symbol: &str, quantity: f64) {
        let price = self.client.get_time_series_intraday(symbol, IntradayInterval::OneMinute).await;
    }
}