
/// A holding represents a stock that is owned by an account
/// It has a price, quantity, symbol, and purchase date
/// A holding represents a stock that is owned by an account, purchased at a certain price and quantity
/// You can have multiple holdings of the same stock, but each holding is unique, as they have different purchase date times
pub trait Holding{
    fn get_price(&self) -> f64;
    fn get_quantity(&self) -> f64;
    fn get_symbol(&self) -> String;
    fn get_purchase_date(&self) -> chrono::DateTime<chrono::Utc>;
}

/// A stock holding represents a stock that is owned by an account
/// It has a price, quantity, symbol, and purchase date
/// A stock holding represents a stock that is owned by an account, purchased at a certain price and quantity
pub struct StockHolding{
    price: f64,
    quantity: f64,
    symbol: String,
    purchase_date: chrono::DateTime<chrono::Utc>,
}

impl StockHolding{
    pub fn new(price: f64, quantity: f64, symbol: String, purchase_date: chrono::DateTime<chrono::Utc>) -> Self{
        StockHolding{
            price: price,
            quantity: quantity,
            symbol: symbol,
            purchase_date: purchase_date,
        }
    }
}

impl Holding for StockHolding{
    fn get_price(&self) -> f64{
        self.price
    }

    fn get_quantity(&self) -> f64{
        self.quantity
    }

    fn get_symbol(&self) -> String{
        self.symbol.clone()
    }

    fn get_purchase_date(&self) -> chrono::DateTime<chrono::Utc>{
        self.purchase_date
    }
}