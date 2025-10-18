//! Configuration loader for runtime settings

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub solana: SolanaConfig,
    pub pumpfun: PumpFunConfig,
    pub database: DatabaseConfig,
    #[serde(default)]
    pub risk: RiskConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    #[serde(default = "default_grpc_url")]
    pub grpc_url: String,
    pub chain: String,
}

fn default_grpc_url() -> String {
    "http://localhost:10000".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunConfig {
    pub program_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default)]
    pub discovery_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskConfig {
    #[serde(default = "default_per_trade_sol_cap")]
    pub per_trade_sol_cap: f64,
    #[serde(default = "default_per_hour_loss_cap_sol")]
    pub per_hour_loss_cap_sol: f64,
    #[serde(default = "default_max_concurrent_per_mint")]
    pub max_concurrent_per_mint: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionConfig {
    #[serde(default = "default_base_slippage_bps")]
    pub base_slippage_bps: u16,
    #[serde(default)]
    pub use_tpu: bool,
    #[serde(default)]
    pub use_jito: bool,
}

fn default_per_trade_sol_cap() -> f64 { 1.0 }
fn default_per_hour_loss_cap_sol() -> f64 { 5.0 }
fn default_max_concurrent_per_mint() -> usize { 1 }
fn default_base_slippage_bps() -> u16 { 75 }

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .context("Failed to read config file")?;
        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config TOML")?;
        Ok(config)
    }
}
