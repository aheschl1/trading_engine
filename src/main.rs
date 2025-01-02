use alphavantage::Client;
use alphavantage;
use eframe::egui::CentralPanel;
use eframe::{App, CreationContext};
use std::env;
use tokio;

mod bank;

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
}

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    let alphavantage_token = 
        env::var("ALPHAVANTAGE_TOKEN")
        .expect("You must specify a token for the AlphaVantage API with the ALPHAVANTAGE_TOKEN environment variable.");

    let alphavantage_client = Client::new(&alphavantage_token);
    let app = TradingSimulator::new(alphavantage_client);

    eframe::run_native(
        "Trading Simulator", 
        Default::default(),
        Box::new(|_| Ok(Box::new(app))),

    ).unwrap();
    Ok(())
}
