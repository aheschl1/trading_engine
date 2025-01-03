use alphavantage::Client;
use alphavantage;
use eframe::egui::CentralPanel;
use eframe::{App, CreationContext};
use std::env;
use tokio;

mod utils;
mod bank;

const APP_NAME: &str = "trading_simulator";

struct TradingSimulator{
    alphavantage_client: Client,
    bank: Option<bank::Bank>,
}

impl TradingSimulator{
    fn new(alphavantage_client: Client) -> Self{
        TradingSimulator{
            alphavantage_client: alphavantage_client,
            bank: None
        }
    }
}

impl App for TradingSimulator{
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Trading Simulator");
            ui.label("Welcome to the trading simulator!");
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Save the bank state
        if let Some(bank) = &self.bank{
            _storage.set_string("bank", serde_json::to_string(bank).unwrap());
        }
        _storage.set_string("alphavantage_token", "hello".to_string());
    }

    
}

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    let alphavantage_token = 
        env::var("ALPHAVANTAGE_TOKEN")
        .expect("You must specify a token for the AlphaVantage API with the ALPHAVANTAGE_TOKEN environment variable.");

    let alphavantage_client = Client::new(&alphavantage_token);
    let app = TradingSimulator::new(alphavantage_client);

    let options = eframe::NativeOptions::default();
    // ~/.config/trading_simulator should be the default path for the persistence file
    // options.persistence_path = Some(utils::expand_tilde(format!("~/.config/{APP_NAME}").as_str()));

    eframe::run_native(
        APP_NAME, 
        options,
        Box::new(|_| Ok(Box::new(app))),
    ).unwrap();
    Ok(())
}
