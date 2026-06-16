use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub async fn get_last_indexed_block(pool: &SqlitePool, pool_address: &str) -> Result<Option<i64>> {
    let row = sqlx::query!(
        "SELECT last_indexed_block FROM pool_state WHERE pool_address = ?",
        pool_address
    )
    .fetch_optional(pool)
    .await
    .with_context(|| format!("Failed to read pool: {pool_address}"))?;

    Ok(row.map(|r| r.last_indexed_block))
}

pub async fn update_last_indexed_block(
    pool: &SqlitePool,
    last_indexed_block: i64,
    pool_address: &str,
) -> Result<()> {
    sqlx::query!(
        "UPDATE pool_state SET last_indexed_block = ? WHERE pool_address = ?",
        last_indexed_block,
        pool_address
    )
    .execute(pool)
    .await
    .with_context(|| format!("Failed to update last indexed block for pool: {pool_address}"))?;
    Ok(())
}

pub async fn insert_pool_state(
    pool: &SqlitePool,
    pool_address: &str,
    last_indexed_block: i64,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO pool_state (pool_address, usdc_reserve, usdt_reserve, last_indexed_block) VALUES (?, '0', '0', ?)",
        pool_address,
        last_indexed_block
    )
    .execute(pool)
    .await
    .with_context(|| format!("Failed to insert into pool: {pool_address}"))?;
    Ok(())
}

pub async fn update_pool_state(
    pool: &SqlitePool,
    pool_address: &str,
    usdc_reserve: &str,
    usdt_reserve: &str,
    last_indexed_block: i64,
) -> Result<()> {
    sqlx::query!(
        "UPDATE pool_state SET usdc_reserve = ?, usdt_reserve = ?, last_indexed_block = ? WHERE pool_address = ?",
        usdc_reserve,
        usdt_reserve,
        last_indexed_block,
        pool_address
    )
    .execute(pool)
    .await
    .with_context(|| format!(" Failed to update pool: {pool_address}"))?;
    Ok(())
}

pub async fn insert_liquidity_event(
    pool: &SqlitePool,
    tx_hash: &str,
    log_index: i64,
    event_type: &str,
    amount_usdc: &str,
    amount_usdt: &str,
    amount_stb: &str,
    receiver: &str,
    block_number: i64,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO liquidity_events (tx_hash, log_index, event_type, amount_usdc, amount_usdt, amount_stb, receiver, block_number) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        tx_hash,
        log_index,
        event_type,
        amount_usdc,
        amount_usdt,
        amount_stb,
        receiver,
        block_number
    )
    .execute(pool)
    .await
    .with_context(|| "Failed to insert into liquidity event table")?;
    Ok(())
}

pub async fn insert_swap_event(
    pool: &SqlitePool,
    tx_hash: &str,
    log_index: i64,
    token: &str,
    amount: &str,
    quote_amount: &str,
    fees: &str,
    receiver: &str,
    block_number: i64,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO swaps (tx_hash, log_index, token, amount, quote_amount, fees, receiver, block_number) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        tx_hash,
        log_index,
        token,
        amount,
        quote_amount,
        fees,
        receiver,
        block_number
    )
    .execute(pool)
    .await
    .with_context(|| "Failed to insert into swap table")?;
    Ok(())
}
