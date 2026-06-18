use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub async fn insert_stabilizer_state(
    pool: &SqlitePool,
    contract_address: &str,
    usdc_reserve: &str,
    usdt_reserve: &str,
    usdc_price: &str,
    usdt_price: &str,
    block_number: i64,
    block_hash: &str,
    tx_hash: &str,
    log_index: i64,
    chain_id: i64
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO stabilizer_state (contract_address, usdc_reserve, usdt_reserve, usdc_price, usdt_price, block_number, block_hash, tx_hash, log_index, chain_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        contract_address,
        usdc_reserve,
        usdt_reserve,
        usdc_price,
        usdt_price,
        block_number,
        block_hash,
        tx_hash,
        log_index,
        chain_id,
    )
    .execute(pool)
    .await
    .context("Failed to insert into stabilizer state")?;
    Ok(())
}

pub async fn get_cursor<'a>(pool: &SqlitePool, contract_address: &str, chain_id: i64) -> Result<Option<(i64, &'a str)>> {
    let row = sqlx::query!(
        "SELECT last_indexed_block, last_indexed_block_hash FROM cursor WHERE contract_address = ? AND chain_id = ?",
        contract_address,
        chain_id
    )
    .fetch_optional(pool)
    .await
    .with_context(|| format!("Failed to read contract: {contract_address} on chain: {chain_id}"))?;

    Ok(row.map(|r| (r.last_indexed_block, r.last_indexed_block_hash)))
}

pub async fn insert_cursor(
    pool: &SqlitePool,
    contract_address: &str,
    last_indexed_block_number: i64,
    last_indexed_block_hash: &str,
    last_indexed_block_timestamp: i64,
    chain_id: i64,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO cursor (contract_address, last_indexed_block_number, last_indexed_block_hash, last_indexed_block_timestamp, chain_id) VALUES (?, ?, ?, ?, ?)",
        contract_address,
        last_indexed_block_number,
        last_indexed_block_hash,
        last_indexed_block_timestamp,
        chain_id,
    )
    .execute(pool)
    .await
    .with_context(|| format!("Failed to insert indexer state for contract: {contract_address} on chain: {chain_id}"))?;
    Ok(())
}

pub async fn update_cursor(
    pool: &SqlitePool,
    contract_address: &str,
    chain_id: i64,
    last_indexed_block: i64,
    last_indexed_block_hash: &str,
    last_indexed_block_timestamp: &str
) -> Result<()> {
    sqlx::query!(
        "UPDATE cursor SET last_indexed_block = ?, last_indexed_block_hash = ?, last_indexed_block_timestamp = ? WHERE contract_address = ? AND chain_id = ?",
        last_indexed_block,
        last_indexed_block_hash,
        last_indexed_block_timestamp,
        contract_address,
        chain_id,
    )
    .extend(pool)
    .await
    .with_context(|| format!("Failed to update cursor for contract: {contract_address} on chain: {chain_id}"))?;
    Ok(())
}

pub async fn get_blocks(pool: &SqlitePool, contract_address: &str, chain_id: i64) -> Result<Option<Vec<(i64, String)>>> {
    let rows = sqlx::query!(
        "SELECT block_number, block_hash FROM blocks WHERE contract_address = ? AND  chain_id = ? ORDER BY block_number DESC LIMIT 12",
        contract_address,
        chain_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch content")?;
    let blocks = rows
        .into_iter()
        .map(|r| (r.block_number, r.block_hash))
        .collect();
    Ok(Some(blocks))
}

pub async fn insert_block(
    pool: &SqlitePool,
    contract_address: &str,
    block_number: i64,
    block_hash: &str,
    block_timestamp: i64,
    chain_id: i64
)  -> Result<()> {
    sqlx::query!(
        "INSERT INTO blocks (contract_address, block_number, block_hash, block_timestamp, chain_id) VALUES (?, ?, ?, ?, ?)",
        contract_address,
        block_number,
        block_hash,
        block_timestamp,
        chain_id,
    )
    .execute(pool)
    .await
    .context("Failed to insert block")?;
    Ok(())
}

pub async fn insert_swap_event(
    pool: &SqlitePool,
    tx_hash: &str,
    block_hash: &str,
    log_index: i64,
    contract_address: &str,
    token: &str,
    amount: &str,
    quote_amount: &str,
    fees: &str,
    receiver: &str,
    block_number: i64,
    block_timestamp: i64,
    chain_id: i64,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO swaps (tx_hash, block_hash, log_index, contract_address, token, amount, quote_amount, fees, receiver, block_number, block_timestamp, chain_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        tx_hash,
        block_hash,
        log_index,
        contract_address,
        token,
        amount,
        quote_amount,
        fees,
        receiver,
        block_number,
        block_timestamp,
        chain_id,
    )
    .execute(pool)
    .await
    .with_context(|| "Failed to insert into swap table")?;
    Ok(())
}

pub async fn insert_liquidity_event(
    pool: &SqlitePool,
    tx_hash: &str,
    block_hash: &str,
    log_index: i64,
    contract_address: &str,
    event_type: &str,
    amount_usdc: &str,
    amount_usdt: &str,
    amount_stb: &str,
    receiver: &str,
    block_number: i64,
    block_timestamp: i64,
    chain_id: i64,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO liquidity_events (tx_hash, block_hash, log_index, contract_address, event_type, amount_usdc, amount_usdt, amount_stb, receiver, block_number, block_timestamp, chain_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        tx_hash,
        block_hash,
        log_index,
        contract_address,
        event_type,
        amount_usdc,
        amount_usdt,
        amount_stb,
        receiver,
        block_number,
        block_timestamp,
        chain_id,
    )
    .execute(pool)
    .await
    .with_context(|| "Failed to insert into liquidity event table")?;
    Ok(())
}


pub async fn perform_removal(
    pool: &SqlitePool,
    fork_point: i64
) -> Result<()> {
    sqlx::query!(
        "DELETE * FROM liquidity_events where block_number > ?",
        fork_point
    )
    .execute(pool)
    .await
    .context("Failed to perform removal")?;
    sqlx::query!(
        "DELETE * FROM swaps where block_number > ?",
        fork_point
    )
    .execute(pool)
    .await
    .context("Failed to perform removal")?;
    sqlx::query!(
        "DELETE * FROM blocks where block_number > ?",
        fork_point
    )
    .execute(pool)
    .await
    .context("Failed to perform removal")?;
    Ok(())
}