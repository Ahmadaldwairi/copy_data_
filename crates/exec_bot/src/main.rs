use anyhow::Result;
use common::logging;
use db as database;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    info!("exec_bot starting up...");

    let pool = database::connect(None).await?;
    database::health_check(&pool).await?;
    info!("database reachable");

    info!("exec_bot stub running; add strategy next.");
    Ok(())
}
