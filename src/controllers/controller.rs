use crate::{
    IStabilizer,
    connection::queries::interact::*
};
use alloy::{consensus::BlockHeader, primitives::Address, providers::Provider, rpc::types::eth::Filter, signers::k256::elliptic_curve::rand_core::block};
use anyhow::{Context, Result};
use sqlx::{SqlitePool};
use tracing::{info, warn};

pub async fn get_or_create_cursor(
    pool: &SqlitePool,
    contract: Address,
    provider: &impl Provider,
    chain_id: i64
) -> Result<u64> {
    let contract_address = contract.to_string();
    if let Some((block, _)) = get_cursor(pool, &contract_address, chain_id as i64)
        .await
        .context("Failed to read last indexed block")?
    {
        Ok(block as u64)
    } else {
        let latest_block = provider
            .get_block_number()
            .await
            .context("Failed to fetch block number")?;

        let header = provider.get_header_by_number(alloy::eips::BlockNumberOrTag::Number(latest_block)).await.context("Failed to fetch block hash")?.unwrap();
        let block_hash = header.hash.to_string();
        let block_timestamp = header.timestamp();
        insert_cursor(pool, &contract_address, latest_block as i64, &block_hash, block_timestamp as i64, chain_id)
            .await
            .context("Failed to insert into pool state")?;
        info!(block = latest_block, "Starting indexer from latest block");
        Ok(latest_block)
    }
}

pub async fn sync_events(
    pool: &SqlitePool,
    contract: Address,
    provider: &impl Provider,
    last_indexed_block: u64,
    chain_id: i64
) -> Result<u64> {
    let latest_block = provider
        .get_block_number()
        .await
        .context("Failed to fetch block number")?;

    if latest_block <= last_indexed_block {
        return Ok(last_indexed_block);
    }

    let interface = IStabilizer::new(contract, &provider);

    // last_indexed_block = handle_reorg(
    //     pool,
    //     contract,
    //     last_indexed_block,
    //     chain_id
    // )?;

    let batch_size = 10u64;
    let mut from_block = last_indexed_block + 1;
    let contract_address = contract.to_string();

    while from_block <= latest_block {
        info!("Scanning for New Batch");
        let to_block = std::cmp::min(from_block + batch_size - 1, latest_block);

        let filter = Filter::new()
            .address(contract)
            .from_block(from_block)
            .to_block(to_block);

        let logs = provider
            .get_logs(&filter)
            .await
            .context("Failed to fetch logs")?;

        let mut batch_has_events = false;

        for log in logs {
            let tx_hash = log.transaction_hash.unwrap_or_default().to_string();
            let log_index = log.log_index.unwrap_or_default() as i64;
            let block_number = log.block_number.unwrap_or_default() as i64;
            let block_hash = log.block_hash.unwrap_or_default().to_string();
            let block_timestamp = log.block_timestamp.unwrap_or_default() as i64;

            let mut block_event = false;

            let IStabilizer::getStabilizerMatrixReturn {
                usdcReserveAmount,
                usdtReserveAmount,
                usdcPrice,
                usdtPrice,
            } = interface
                .getStabilizerMatrix()
                .call()
                .await
                .context("Failed to fetch stabilizer matrix")?;
            let usdc_reserve = usdcReserveAmount.to_string();
            let usdt_reserve = usdtReserveAmount.to_string();
            let usdc_price = usdcPrice.to_string();
            let usdt_price = usdtPrice.to_string();

            if let Ok(event) = log.log_decode::<IStabilizer::LiquidityAdded>() {
                let amount_usdc = event.data().amountUsdc.to_string();
                let amount_usdt = event.data().amountUsdt.to_string();
                let amount_stb = event.data().amountStb.to_string();
                let receiver = event.data().receiver.to_string();

                info!(tx_hash, "Liquidity added detected");
                insert_liquidity_event(
                    pool,
                    &tx_hash,
                    &block_hash,
                    log_index,
                    &contract_address,
                    "ADD",
                    &amount_usdc,
                    &amount_usdt,
                    &amount_stb,
                    &receiver,
                    block_number,
                    block_timestamp,
                    chain_id
                )
                .await
                .context("Failed to insert liquidity event")?;
                insert_stabilizer_state(pool, &contract_address, &usdc_reserve, &usdt_reserve, &usdc_price, &usdt_price, block_number, &block_hash, &tx_hash, log_index, chain_id)
                    .await.context("Failed to insert stabilizer state")?;
                batch_has_events = true;
                block_event = true;
            } else if let Ok(event) = log.log_decode::<IStabilizer::LiquidityRemoved>() {
                let amount_usdc = event.data().amountUsdc.to_string();
                let amount_usdt = event.data().amountUsdt.to_string();
                let amount_stb = event.data().amountStb.to_string();
                let receiver = event.data().receiver.to_string();
                info!(tx_hash, "Liquidity removed detected");
                insert_liquidity_event(
                    pool,
                    &tx_hash,
                    &block_hash,
                    log_index,
                    &contract_address,
                    "REMOVE",
                    &amount_usdc,
                    &amount_usdt,
                    &amount_stb,
                    &receiver,
                    block_number,
                    block_timestamp,
                    chain_id
                )
                .await
                .context("Failed to insert liquidity event")?;
                insert_stabilizer_state(pool, &contract_address, &usdc_reserve, &usdt_reserve, &usdc_price, &usdt_price, block_number, &block_hash, &tx_hash, log_index, chain_id)
                    .await.context("Failed to insert stabilizer state")?;
                batch_has_events = true;
                block_event = true;
            } else if let Ok(event) = log.log_decode::<IStabilizer::Exchange>() {
                let token = event.data().token.to_string();
                let amount = event.data().amount.to_string();
                let quote_amount = event.data().quoteAmount.to_string();
                let fees = event.data().fees.to_string();
                let receiver = event.data().receiver.to_string();
                info!(tx_hash, "Exchange detected");
                insert_swap_event(
                    pool,
                    &tx_hash,
                    &block_hash,
                    log_index,
                    &contract_address,
                    &token,
                    &amount,
                    &quote_amount,
                    &fees,
                    &receiver,
                    block_number,
                    block_timestamp,
                    chain_id,
                )
                .await
                .context("Failed to insert swap event")?;
                insert_stabilizer_state(pool, &contract_address, &usdc_reserve, &usdt_reserve, &usdc_price, &usdt_price, block_number, &block_hash, &tx_hash, log_index, chain_id)
                    .await.context("Failed to insert stabilizer state")?;
                batch_has_events = true;
                block_event = true;
            }

            if block_event {
                insert_block(
                    pool,
                    &contract_address,
                    block_number,
                    &block_hash,
                    block_timestamp,
                    chain_id
                )
                .await
                .context("Failed to add block")?;
            }
        }

        let block_hash = provider.get_header_by_number(alloy::eips::BlockNumberOrTag::Number(to_block)).await.context("Failed to fetch block header")?.unwrap().hash.to_string();
        let parent_hash = provider.get_header_by_number(alloy::eips::BlockNumberOrTag::Number(to_block)).await.context("Failed to fetch block header")?.unwrap().parent_hash.to_string();
        update_cursor(pool, &contract_address, chain_id, to_block as i64, &block_hash, &parent_hash)
            .await
            .context("Failed to update cursor")?;
        
        from_block = to_block + 1;
    }
    Ok(latest_block)
}

async fn check_reorg(
    pool: &SqlitePool,
    contract_address: &str,
    chain_id: i64,
    provider: &impl Provider
) -> Result<u64>{
    info!("Checking for reorg");
    let (last_indexed_block, last_indexed_block_hash) = get_cursor(pool, contract_address, chain_id).await.context("Failed get cursor")?.unwrap();
    let block_hash = provider.get_header_by_number(alloy::eips::BlockNumberOrTag::Number(last_indexed_block as u64)).await.context("Failed to fetch block header")?.unwrap().hash.to_string();
    if last_indexed_block_hash != &block_hash {
        warn!("Block reorg detected at block: {last_indexed_block}");
        handle_reorg(pool, contract_address, chain_id, provider).await.context("Failed handling reorg")?;
    }
    Ok(last_indexed_block as u64)
}

async fn handle_reorg(pool: &SqlitePool, contract_address: &str, chain_id: i64, provider: &impl Provider) -> Result<i64> {
    info!("Handling reorg");
    let blocks = get_blocks(pool, contract_address, chain_id).await.context("Failed to fetch block data")?.unwrap();
    let mut fork_point = 0;
    for block in blocks {
        let new_block_hash = provider.get_header_by_number(alloy::eips::BlockNumberOrTag::Number(block.0 as u64)).await.context("Failed to fetch block header")?.unwrap().hash.to_string();
        if new_block_hash == block.1 {
            fork_point = block.0;
        }
    }

    if fork_point > 0 {
        info!("Fork point detected: {fork_point}");
        warn!("Reindexing from block: {fork_point}");
        perform_removal(pool, fork_point).await.context("Failed removal")?;
    }
    
    Ok(fork_point)
}