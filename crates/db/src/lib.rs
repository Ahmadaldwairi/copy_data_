//! Database access layer with sqlx for SQLite or Postgres.

pub mod raw_events;
pub mod discovery;

use anyhow::Result;
#[cfg(feature = "sqlite")]
pub type Pool = sqlx::SqlitePool;
#[cfg(feature = "postgres")]
pub type Pool = sqlx::PgPool;

#[cfg(feature = "sqlite")]
const DEFAULT_DB_URL: &str = "sqlite://copytrader.db";
#[cfg(feature = "postgres")]
const DEFAULT_DB_URL: &str = "postgres://postgres:postgres@localhost/copytrader";

pub async fn connect(db_url: Option<&str>) -> Result<Pool> {
    let url = db_url.unwrap_or(DEFAULT_DB_URL);
    let pool = Pool::connect(url).await?;
    Ok(pool)
}

pub async fn health_check(pool: &Pool) -> Result<()> {
    #[cfg(feature = "sqlite")]
    {
        let _row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(pool)
            .await?;
    }
    #[cfg(feature = "postgres")]
    {
        let _row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(pool)
            .await?;
    }
    Ok(())
}
