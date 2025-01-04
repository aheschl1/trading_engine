use alphavantage::Client;

pub struct Broker {
    client: Client,
    bank_balance: f64,
}

impl Broker {
    pub fn new(api_key: &str, initial_balance: f64) -> Self {
        let client = Client::new(api_key);
        Broker {
            client,
            bank_balance: initial_balance,
        }
    }
}