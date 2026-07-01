use crate::connection::queries::interact::*;
use alloy::{eips::BlockNumberOrTag::Number, providers::Provider};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tracing::{info, warn};

pub async fn check_reorg(
    pool: &SqlitePool,
    contract_address: &str,
    chain_id: i64,
    provider: &impl Provider,
) -> Result<u64> {
    info!("Checking for reorg");
    let (last_indexed_block, last_indexed_block_hash) =
        get_cursor(pool, contract_address, chain_id)
            .await
            .context("Failed get cursor")?
            .unwrap();
    //>/ @note get_header_by_number doe not work on polygon amoy
    // let block_hash = provider
    //     .get_header_by_number(alloy::eips::BlockNumberOrTag::Number(
    //         last_indexed_block as u64,
    //     ))
    //     .await
    //     .context("Failed to fetch block header")?
    //     .unwrap()
    //     .hash
    //     .to_string();

    let block_hash = provider
        .get_block_by_number(Number(last_indexed_block as u64))
        .await
        .context("Faield to fetch block")?
        .unwrap()
        .header
        .hash
        .to_string();

    if last_indexed_block_hash != block_hash {
        warn!("Block reorg detected at block: {last_indexed_block}");
        let fork_point = handle_reorg(pool, contract_address, chain_id, provider)
            .await
            .context("Failed handling reorg")?;
        return Ok(fork_point as u64); // return fork point if reorg detected
    }
    Ok(last_indexed_block as u64) // return cursor if no reorg
}

async fn handle_reorg(
    pool: &SqlitePool,
    contract_address: &str,
    chain_id: i64,
    provider: &impl Provider,
) -> Result<i64> {
    info!("Handling reorg");
    let blocks = get_blocks(pool, contract_address, chain_id)
        .await
        .context("Failed to fetch block data")?
        .unwrap();
    let mut fork_point = 0;
    for block in blocks {
        let new_block_hash = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(block.0 as u64))
            .await
            .context("Failed to fetch block header")?
            .unwrap()
            .header
            .hash
            .to_string();
        if new_block_hash == block.1 {
            fork_point = block.0;
            break;
        } // fork point will be zero in case of no hash match, the program assums that the reorg will be detected from last 10 stored blocks from blocks table(in which the protocol has emitted events)
    }

    if fork_point > 0 {
        info!("Fork point detected: {fork_point}");
        warn!("Reindexing from block: {fork_point}");
        perform_removal(pool, fork_point)
            .await
            .context("Failed removal")?;
    }

    Ok(fork_point) // return zero if no fork point is detected or return form point
}
