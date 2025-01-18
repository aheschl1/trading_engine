use chrono::FixedOffset;
use serde::{Deserialize, Serialize};


/// An asset is a holding that represents a stock or a cryptocurrency.
/// It has a total cost, a quantity, and a symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding{
    asset: Asset,
    pub average_cost_per_unit: f64,
    pub quantity: f64,
}

impl Holding{
    pub fn new(total_cost: f64, quantity: f64, symbol: String) -> Self{
        Holding{
            average_cost_per_unit: total_cost,
            quantity: quantity,
            asset: Asset::new(symbol),
        }
    }

    fn get_price(&self) -> f64{
        self.average_cost_per_unit
    }

    fn get_quantity(&self) -> f64{
        self.quantity
    }

    fn get_symbol(&self) -> String{
        self.asset.get_symbol()
    }
}

/// An asset is a stock or a cryptocurrency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset{
    pub symbol: String,
}

impl Asset{
    pub fn new(symbol: String) -> Self{
        Asset{
            symbol: symbol
        }
    }

    pub fn get_symbol(&self) -> String{
        self.symbol.clone()
    }
}

pub struct Dividend{
    pub amount: f64,
    pub asset: Asset,
    pub date: chrono::DateTime<FixedOffset>,
}

impl Dividend{
    pub fn new(amount: f64, asset: Asset, date: chrono::DateTime<FixedOffset>) -> Self{
        Dividend{
            amount: amount,
            asset: asset,
            date: date,
        }
    }
}

