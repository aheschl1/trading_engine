use serde::{Deserialize, Serialize};


/// An asset is a holding that represents a stock or a cryptocurrency.
/// It has a total cost, a quantity, and a symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding{
    asset: Asset,
    pub total_cost: f64,
    pub quantity: f64,
}

impl Holding{
    pub fn new(total_cost: f64, quantity: f64, symbol: String) -> Self{
        Holding{
            total_cost: total_cost,
            quantity: quantity,
            asset: Asset::new(symbol),
        }
    }

    fn get_price(&self) -> f64{
        self.total_cost
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
    symbol: String,
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