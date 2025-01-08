use alphavantage::cache_enabled::client::Client;
use alphavantage;
use bank::accounts::Account;
use bank::Bank;
use utils::APP_NAME;
use std::env;
use tokio;

mod utils;
mod bank;
mod brokerage;
mod  state;

struct TradingSimulator{
    brokerage: brokerage::Broker,
}

impl TradingSimulator{
    fn new(alphavantage_client: Client, bank: Bank) -> Self{
        let broker = brokerage::Broker::new(alphavantage_client, bank);
        TradingSimulator{
            brokerage: broker,
        }
    }
}

async fn save_state(bank: &Bank) -> Result<(), tokio::io::Error>{
    state::save_bank(&bank).await?;    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    let alphavantage_token = 
        env::var("ALPHAVANTAGE_TOKEN")
        .expect("You must specify a token for the AlphaVantage API with the ALPHAVANTAGE_TOKEN environment variable.");

    let alphavantage_client = Client::new(&alphavantage_token);
    let mut bank  = match state::load_bank().await{
        Ok(bank) => bank,
        Err(_) => Bank::empty(),
    };
    // dummy account
    bank.open_account(Some("Name".to_string()), bank::accounts::AccountType::Checking).unwrap();
    bank.open_account(Some("Name".to_string()), bank::accounts::AccountType::Investment).unwrap();

    Ok(())
}
