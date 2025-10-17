use anyhow::{Context, Result};
use common::{config::Config, logging, sol_price::SolPriceCache};
use db::{self as database, raw_events::batch_insert_raw_events};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};
use tokio_stream::StreamExt;
use tracing::{error, info, warn};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterTransactions, SubscribeUpdateTransaction,
};
use solana_sdk::pubkey::Pubkey;

const BATCH_SIZE: usize = 100;
const BATCH_INTERVAL_SECS: u64 = 5;
const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();

    info!("? Pump.fun ingestion bot starting up (Yellowstone gRPC)...");

    // Load config
    let config = Config::load("configs/config.example.toml")?;
    info!(" Loaded config: grpc_url={}, db_url=***", config.solana.grpc_url);

    // Connect to database
    let pool = database::connect(Some(&config.database.url)).await?;
    database::health_check(&pool).await?;
    info!(" Database connection healthy");

    // Initialize SOL price cache
    let sol_price_cache = SolPriceCache::new();
    sol_price_cache.clone().start_updater();
    info!(" SOL price updater started (fetching every 10 seconds)");

    // Load tracked wallets
    let (tracked_wallets, wallet_aliases) = load_tracked_wallets(&pool).await?;
    info!("? Loaded {} tracked wallets", tracked_wallets.len());

    let program_id = Pubkey::from_str(&config.pumpfun.program_id)?;
    info!("? Monitoring Pump.fun program: {}", program_id);

    // Shared buffer for batching
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let buffer_clone = buffer.clone();
    let pool_clone = pool.clone();

    // Spawn batch flusher
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(BATCH_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let mut buf = buffer_clone.lock().await;
            if !buf.is_empty() {
                let events: Vec<_> = buf.drain(..).collect();
                drop(buf);
                if let Err(e) = batch_insert_raw_events(&pool_clone, &events).await {
                    error!("Failed to flush events: {}", e);
                } else if !events.is_empty() {
                    info!("? Flushed {} events to database", events.len());
                }
            }
        }
    });

    // Main ingestion loop with reconnection
    loop {
        info!("? Connecting to Yellowstone gRPC: {}", config.solana.grpc_url);
        match run_grpc_stream(
            &config.solana.grpc_url,
            &program_id,
            &tracked_wallets,
            &wallet_aliases,
            buffer.clone(),
            sol_price_cache.clone(),
        )
        .await
        {
            Ok(_) => {
                warn!("Stream ended normally, reconnecting...");
            }
            Err(e) => {
                error!("Stream error: {}, reconnecting in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn load_tracked_wallets(pool: &database::Pool) -> Result<(Vec<String>, HashMap<String, String>)> {
    let rows = sqlx::query("SELECT wallet, alias FROM wallets WHERE is_tracked")
        .fetch_all(pool)
        .await?;

    let mut wallets = Vec::new();
    let mut aliases = HashMap::new();
    
    for row in rows {
        use sqlx::Row;
        let wallet: String = row.get("wallet");
        let alias: Option<String> = row.get("alias");
        
        wallets.push(wallet.clone());
        if let Some(alias) = alias {
            aliases.insert(wallet, alias);
        }
    }

    Ok((wallets, aliases))
}

async fn run_grpc_stream(
    endpoint: &str,
    program_id: &Pubkey,
    tracked_wallets: &[String],
    wallet_aliases: &HashMap<String, String>,
    buffer: Arc<Mutex<Vec<db::raw_events::RawEvent>>>,
    sol_price_cache: SolPriceCache,
) -> Result<()> {
    // Connect to gRPC using the same pattern as your working bot
    let mut client = GeyserGrpcClient::build_from_shared(endpoint.to_string())?
        .x_token::<String>(None)?
        .connect()
        .await?;

    info!(" Connected to Yellowstone gRPC");

    // Create subscription request - exactly like your working bot
    let mut transactions: HashMap<String, SubscribeRequestFilterTransactions> = HashMap::new();
    transactions.insert(
        "pump_transactions".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: Some(false),
            signature: None,
            account_include: vec![program_id.to_string()],
            account_exclude: vec![],
            account_required: vec![],
        },
    );

    let request = SubscribeRequest {
        accounts: HashMap::new(),
        slots: HashMap::new(),
        transactions,
        transactions_status: HashMap::new(),
        blocks: HashMap::new(),
        blocks_meta: HashMap::new(),
        entry: HashMap::new(),
        commitment: Some(CommitmentLevel::Confirmed as i32),
        accounts_data_slice: vec![],
        ping: None,
        from_slot: None,
    };

    // Subscribe and get stream
    let mut stream = client.subscribe_once(request).await?;

    info!("? Subscribed to Yellowstone gRPC stream");
    info!("? Listening for Pump.fun transactions...");

    let mut tx_count = 0;

    // Process messages
    loop {
        match stream.next().await {
            Some(Ok(msg)) => {
                if let Some(update) = msg.update_oneof {
                    match update {
                        UpdateOneof::Transaction(tx_update) => {
                            tx_count += 1;

                            if tx_count % 100 == 0 {
                                info!("? Processed {} transactions", tx_count);
                            }

                            // Get current SOL price
                            let sol_price = sol_price_cache.get_price().await;

                            // Process transaction
                            if let Err(e) =
                                process_transaction(&tx_update, tracked_wallets, wallet_aliases, program_id, buffer.clone(), sol_price).await
                            {
                                warn!("Failed to process transaction: {}", e);
                            }
                        }
                        _ => {
                            // Ignore other update types
                        }
                    }
                }
            }
            Some(Err(e)) => {
                error!("Stream error: {}", e);
                return Err(e.into());
            }
            None => {
                warn!("Stream ended");
                return Ok(());
            }
        }
    }
}

async fn process_transaction(
    tx: &SubscribeUpdateTransaction,
    tracked_wallets: &[String],
    wallet_aliases: &HashMap<String, String>,
    program_id: &Pubkey,
    buffer: Arc<Mutex<Vec<db::raw_events::RawEvent>>>,
    sol_price: f64,
) -> Result<()> {
    // Extract transaction data
    let transaction = tx.transaction.as_ref().context("No transaction")?;
    let meta = transaction.meta.as_ref().context("No meta")?;
    let tx_data = transaction.transaction.as_ref().context("No tx data")?;
    let message = tx_data.message.as_ref().context("No message")?;

    // Get signature
    let sig = bs58::encode(&transaction.signature).into_string();

    // Get account keys from transaction
    let mut account_keys = Vec::new();
    for key in &message.account_keys {
        if let Ok(pubkey) = Pubkey::try_from(key.as_slice()) {
            account_keys.push(pubkey.to_string());
        }
    }

    // Find tracked wallets in this transaction
    let found_wallets: Vec<String> = account_keys
        .iter()
        .filter(|key| tracked_wallets.contains(key))
        .cloned()
        .collect();

    // Skip if no tracked wallets
    if found_wallets.is_empty() {
        return Ok(());
    }

    // Extract fee (in lamports)
    let fee_sol = Some(meta.fee as f64 / LAMPORTS_PER_SOL);

    // Get pre and post balances for SOL amount calculations
    let pre_balances = &meta.pre_balances;
    let post_balances = &meta.post_balances;

    // Found tracked wallet activity
    info!("TRACKED WALLET DETECTED! Signature: {}...", &sig[..8]);
    
    // Display wallet names (aliases) if available, otherwise first 8 chars
    let wallet_names: Vec<String> = found_wallets
        .iter()
        .map(|w| {
            wallet_aliases
                .get(w)
                .cloned()
                .unwrap_or_else(|| w[..8].to_string())
        })
        .collect();
    info!("   Wallets: {:?}", wallet_names);

    // Decode Pump.fun instructions
    let mut decoded_actions = Vec::new();
    for instruction in &message.instructions {
        let program_idx = instruction.program_id_index as usize;
        if program_idx < account_keys.len() && account_keys[program_idx] == program_id.to_string() {
            // This is a Pump.fun instruction
            let decoded = decoder::decode_instruction(&instruction.data, &account_keys)?;
            decoded_actions.push(decoded);
        }
    }

    // If no decodable instructions found, skip
    if decoded_actions.is_empty() {
        warn!("     No Pump.fun instructions found in transaction");
        return Ok(());
    }

    info!("    Decoded {} actions: {:?}", 
        decoded_actions.len(), 
        decoded_actions.iter().map(|d| d.action.as_str()).collect::<Vec<_>>()
    );

    // Create events for each tracked wallet and decoded action
    let slot = tx.slot as i64;
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);

    let mut event_count = 0;
    for wallet in &found_wallets {
        // Find wallet's balance index in the transaction
        let wallet_idx = account_keys.iter().position(|k| k == wallet);
        
        // Calculate SOL balance change for this wallet
        let (sol_spent, sol_received) = if let Some(idx) = wallet_idx {
            if idx < pre_balances.len() && idx < post_balances.len() {
                let pre_balance = pre_balances[idx] as f64 / LAMPORTS_PER_SOL;
                let post_balance = post_balances[idx] as f64 / LAMPORTS_PER_SOL;
                let balance_change = post_balance - pre_balance;
                
                // If balance decreased, user spent SOL (BUY)
                // If balance increased, user received SOL (SELL)
                if balance_change < 0.0 {
                    (Some(-balance_change), None) // Spent SOL
                } else {
                    (None, Some(balance_change)) // Received SOL
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        for decoded in &decoded_actions {
            // For amount_in: store token amount
            // For amount_out: store SOL amount (spent on BUY, received on SELL)
            let (amount_in, amount_out) = match decoded.action {
                decoder::Action::Buy => {
                    (decoded.token_amount.map(|amt| amt as f64), sol_spent)
                }
                decoder::Action::Sell => {
                    (decoded.token_amount.map(|amt| amt as f64), sol_received)
                }
                decoder::Action::Create => {
                    (None, None)
                }
                _ => (None, None),
            };

            let event = db::raw_events::RawEvent {
                ts_ns,
                slot: Some(slot),
                sig: Some(sig.clone()),
                wallet: wallet.clone(),
                program: program_id.to_string(),
                action: decoded.action.as_str().to_string(),
                mint: decoded.mint.clone(),
                base_mint: None,
                quote_mint: None,
                amount_in,
                amount_out,
                price_est: Some(sol_price),
                fee_sol,
                ix_accounts_json: None,
                meta_json: None,
                leader_wallet: None,
            };

            // Log the trade details with SOL amounts
            match decoded.action {
                decoder::Action::Buy => {
                    if let (Some(tokens), Some(sol)) = (decoded.token_amount, sol_spent) {
                        info!("    BUY: {} tokens for {:.4} SOL (${:.2})", 
                            tokens, sol, sol * sol_price);
                    }
                }
                decoder::Action::Sell => {
                    if let (Some(tokens), Some(sol)) = (decoded.token_amount, sol_received) {
                        info!("    SELL: {} tokens for {:.4} SOL (${:.2})", 
                            tokens, sol, sol * sol_price);
                    }
                }
                decoder::Action::Create => {
                    info!("    CREATE: New token mint");
                }
                _ => {}
            }

            let mut buf = buffer.lock().await;
            buf.push(event);
            event_count += 1;

            // Auto-flush if buffer is full
            if buf.len() >= BATCH_SIZE {
                info!(" Buffer full ({} events), triggering flush", buf.len());
            }
        }
    }

    info!("Created {} events for database", event_count);

    Ok(())
}
