use anyhow::{Context, Result};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

use super::queries::create::*;

pub async fn init_db(url: &str) -> Result<SqlitePool> {
    let option = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(option)
        .await
        .with_context(|| format!("Failed connecting db with {url}"))?;

    // Create the pool state table
    sqlx::query(STABILIZER_STATE)
        .execute(&pool)
        .await
        .context("Failed to create stabilizer state table!")?;

    // Create the indexer state table
    sqlx::query(CURSOR)
        .execute(&pool)
        .await
        .context("Failed to create cursor table!")?;

    sqlx::query(BLOCKS)
        .execute(&pool)
        .await
        .context("Failed to create blocks table!")?;

    // Create the swaps Table
    sqlx::query(SWAPS)
        .execute(&pool)
        .await
        .context("Failed to create swaps table!")?;

    // Create the liqudity table
    sqlx::query(LIQUIDITY_EVENTS)
        .execute(&pool)
        .await
        .context("Failed to create liquidity events table!")?;
    Ok(pool)
}
