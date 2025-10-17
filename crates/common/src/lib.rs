//! Common types and utilities shared across crates.

pub mod config;
pub mod sol_price;
pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
    pub struct Wallet(pub String);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Config {
        pub rpc_url: String,
        pub grpc_url: String,
        pub db_url: String,
        pub chain: String,
    }
}

pub mod logging {
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    pub fn init() {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();
        let sub = FmtSubscriber::builder().with_env_filter(filter).finish();
        let _ = tracing::subscriber::set_global_default(sub);
    }
}
