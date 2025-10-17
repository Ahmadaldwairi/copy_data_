//! SOL price fetcher from CoinGecko API

use anyhow::Result;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info};

#[derive(Debug, Clone, Deserialize)]
struct CoinGeckoResponse {
    solana: SolanaPrice,
}

#[derive(Debug, Clone, Deserialize)]
struct SolanaPrice {
    usd: f64,
}

#[derive(Clone)]
pub struct SolPriceCache {
    price: Arc<RwLock<f64>>,
}

impl SolPriceCache {
    pub fn new() -> Self {
        Self {
            price: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Get current cached SOL price in USD
    pub async fn get_price(&self) -> f64 {
        *self.price.read().await
    }

    /// Fetch SOL price from CoinGecko API
    async fn fetch_price() -> Result<f64> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd";
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; CopyTraderBot/1.0)")
            .build()?;
            
        let response = client
            .get(url)
            .send()
            .await?;
            
        let status = response.status();
        let response_text = response.text().await?;
        
        if !status.is_success() {
            anyhow::bail!("API returned status {}: {}", status, response_text);
        }
        
        // Try to parse the response
        match serde_json::from_str::<CoinGeckoResponse>(&response_text) {
            Ok(data) => Ok(data.solana.usd),
            Err(e) => {
                error!("Failed to parse CoinGecko response: {}", response_text);
                Err(e.into())
            }
        }
    }

    /// Start background task to update price every 30 seconds (avoid rate limiting)
    pub fn start_updater(self) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            let mut consecutive_failures = 0;
            
            // Fetch initial price with retries
            for attempt in 1..=3 {
                match Self::fetch_price().await {
                    Ok(price) => {
                        *self.price.write().await = price;
                        info!("Initial SOL price: ${:.2}", price);
                        consecutive_failures = 0;
                        break;
                    }
                    Err(e) => {
                        error!("Failed to fetch initial SOL price (attempt {}/3): {}", attempt, e);
                        if attempt < 3 {
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
            
            loop {
                interval.tick().await;
                
                match Self::fetch_price().await {
                    Ok(price) => {
                        let old_price = *self.price.read().await;
                        *self.price.write().await = price;
                        consecutive_failures = 0;
                        
                        let change_pct = if old_price > 0.0 {
                            ((price - old_price) / old_price) * 100.0
                        } else {
                            0.0
                        };
                        
                        if change_pct.abs() > 0.5 {
                            info!("SOL price updated: ${:.2} ({:+.2}%)", price, change_pct);
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        
                        if consecutive_failures <= 3 {
                            error!("Failed to fetch SOL price (failure {}/3): {}", consecutive_failures, e);
                        } else if consecutive_failures == 10 {
                            error!("SOL price fetch failing repeatedly (10+ times) - using last known price");
                        }
                        
                        // Exponential backoff on repeated failures
                        if consecutive_failures > 5 {
                            let backoff_secs = std::cmp::min(300, consecutive_failures * 30);
                            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                        }
                    }
                }
            }
        });
    }
}

impl Default for SolPriceCache {
    fn default() -> Self {
        Self::new()
    }
}
