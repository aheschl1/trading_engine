use alphavantage::Client;
use alphavantage;
use std::env;
use tokio;

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    let alphavantage_token = 
        env::var("ALPHAVANTAGE_TOKEN")
        .expect("You must specify a token for the AlphaVantage API with the ALPHAVANTAGE_TOKEN environment variable.");

    let alphavantage_client = Client::new(&alphavantage_token);
    // let search_results = alphavantage_client.get_tickers("AAPL").await;
    // println!("{:?}", search_results);
    Ok(())
}
