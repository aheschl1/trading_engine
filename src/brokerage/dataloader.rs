use std::path::{Path, PathBuf};

use alphavantage::{time_series::{IntradayInterval, TimeSeries}, Client};
use chrono::Duration;
use eframe::egui::util::cache;
use tokio;
use crate::utils::{APP_NAME, CACHE_INVALIDATE_RATE, expand_tilde};

/// The dataloader is responsible for interacting with a stock client
/// The loaders goal is to minimize the number of API calls.
/// To do this, we check when the last call was. Maybe we can just return this data which is chached instead
/// 

pub struct Dataloader<'a>{
    client: &'a Client,
    cache_path: PathBuf,
    invalidate_rate: Duration
}

impl<'a> Dataloader<'a>{
    pub async fn new(client: &'a Client) -> Result<Self, tokio::io::Error>{
        // We set cache path, and invalidate rate.
        // If the cache DNE, then, create 
        let invalidate_rate = CACHE_INVALIDATE_RATE;
        let cache = expand_tilde(format!("~/.cache/{APP_NAME}").as_str());
        
        let _ = tokio::fs::create_dir(cache.clone()).await;

        Ok(Dataloader{
            client: client,
            cache_path: cache,
            invalidate_rate: invalidate_rate
        })
    }

    pub async fn get_time_series_intraday(&self, symbol: &str, interval: IntradayInterval) -> Result<TimeSeries, tokio::io::Error>{
        let cache_path = self.cache_path.join("time_series_intraday").join(symbol).join(interval.to_string()).join("data.json");
        self.prepare_cache_directory(&cache_path).await?;
        match self.check_cache_file_valid(&cache_path).await{
            Ok(true) => {
                let data = tokio::fs::read(cache_path).await?;
                let data = serde_json::from_slice(&data).unwrap();
                return Ok(data);
            },
            _ => ()
        }
        // Get the data from the client
        let data = self.client
            .get_time_series_intraday(symbol, interval).await
            .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::Other, e))?;
        // Write the data to the cache: spawn a task to write the data to the cache
        let string_data = serde_json::to_string(&data).unwrap();
        let _ = tokio::spawn(async move {
            tokio::fs::write(cache_path, string_data).await.unwrap();
        });
        Ok(data)
    }

    async fn prepare_cache_directory(&self, path: &PathBuf) -> Result<(), tokio::io::Error>{
        if !tokio::fs::try_exists(path).await?{
            let _ = tokio::fs::create_dir_all(path.parent().unwrap()).await;
        }
        Ok(())
    }

    /// Check if the cache file is valid
    /// A cache file is valid if it exists and the last write time is less than the invalidate rate
    async fn check_cache_file_valid(&self, path: &PathBuf) -> Result<bool, tokio::io::Error>{
        let expiry = self.invalidate_rate;
        if tokio::fs::try_exists(path).await?{
            let last_written = tokio::fs::metadata(path).await?.modified()?;
            let last_written = chrono::DateTime::<chrono::Utc>::from(last_written);
            let duration_since_last_written = chrono::Utc::now().signed_duration_since(last_written);
            if duration_since_last_written < expiry{
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests{
    use crate::utils::get_api_key;

    use super::*;
    use tokio::runtime::Runtime;
    use std::time::{Duration, SystemTime};
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::env;

    #[tokio::test]
    async fn test_dataloader_new(){
        let client = Client::new(get_api_key().as_str());
        let dataloader = Dataloader::new(&client).await.unwrap();
        assert_eq!(dataloader.invalidate_rate, CACHE_INVALIDATE_RATE);
        assert_eq!(dataloader.cache_path, PathBuf::from(expand_tilde("~/.cache/trading_simulator")));
    }

    #[tokio::test]
    async fn test_check_cache_file_valid(){
        let client = Client::new(get_api_key().as_str());
        let dataloader = Dataloader::new(&client).await.unwrap();
        // Create a file - remove it if it exists
        let path = PathBuf::from(expand_tilde("~/.cache/trading_simulator/test.txt"));
        let _ = fs::remove_file(&path);
        // Create a file
        let mut file = File::create(&path).unwrap();
        let _ = file.write_all(b"Hello, world!");
        // Check if the file is valid - should be valid
        let valid = dataloader.check_cache_file_valid(&path).await.unwrap();
        assert_eq!(valid, true);
        // Remove the file
        let _ = fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_check_cache_file_invalid(){
        let client = Client::new(&get_api_key());

        let dataloader = Dataloader::new(&client).await.unwrap();
        // Create a file - remove it if it exists
        let path = PathBuf::from(expand_tilde("~/.cache/trading_simulator/test.txt"));
        let _ = fs::remove_file(&path);
        // Create a file
        let mut file = File::create(&path).unwrap();
        let _ = file.write_all(b"Hello, world!");

        let _ = file.set_modified(SystemTime::from(chrono::Utc::now() - Duration::from_secs(100000)));

        let valid = dataloader.check_cache_file_valid(&path).await.unwrap();
        assert_eq!(valid, false);
        let _ = fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_get_time_series_intraday() {
        // remove the cache file if it exists
        let path = PathBuf::from(expand_tilde("~/.cache/trading_simulator/time_series_intraday/AAPL/1min/data.json"));
        let _ = fs::remove_file(&path);
    
        let client = Client::new(&get_api_key());
        let dataloader = Dataloader::new(&client).await.unwrap();
        let data = dataloader.get_time_series_intraday("AAPL", IntradayInterval::OneMinute).await.unwrap();
        assert_eq!(data.symbol, "AAPL");
    
        // wait for a short duration to ensure the cache is written
        tokio::time::sleep(Duration::from_secs(1)).await;
    
        // check if the cache file exists
        let path = PathBuf::from(expand_tilde("~/.cache/trading_simulator/time_series_intraday/AAPL/1min/data.json"));
        assert!(fs::metadata(path).is_ok());
    }
}