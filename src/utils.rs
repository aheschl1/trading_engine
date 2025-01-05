use std::{self, path::PathBuf, env};

use chrono::Duration;

pub const APP_NAME: &str = "trading_simulator";
pub const CACHE_INVALIDATE_RATE: Duration = Duration::minutes(30);

pub fn get_api_key() -> String {env::var("ALPHAVANTAGE_TOKEN").expect("ALPHAVANTAGE_TOKEN must be set")}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(home_dir) = env::var_os("HOME") {
        PathBuf::from(path.replacen("~", &home_dir.to_string_lossy(), 1))
    } else {
        PathBuf::from(path) // Fallback to original path if HOME isn't set
    }
}