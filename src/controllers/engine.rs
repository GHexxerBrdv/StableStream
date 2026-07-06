use super::reorg_handler::*;
use crate::{IStabilizer, connection::queries::interact::*};
use alloy::{
    consensus::BlockHeader, eips::BlockNumberOrTag::Number, primitives::Address,
    providers::Provider, rpc::types::eth::Filter,
};
use anyhow::{Context, Result};
use sea_orm::DatabaseConnection;
use tracing::info;

pub async fn get_or_create_cursor(
    db: &DatabaseConnection,
    contract: Address,
    provider: &impl Provider,
    chain_id: i64,
) -> Result<u64> {
    let contract_address = contract.to_string();
    if let Some((cursor, _)) = get_cursor(db, &contract_address, chain_id as i64)
        .await
        .context("Failed to read last indexed block")?
    {
        Ok(cursor as u64)
    } else {
        let start_block: u64 = std::env::var("START_BLOCK")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);

        let block = provider
            .get_block_by_number(Number(start_block))
            .await
            .context("Failed to fetch block")?
            .unwrap();

        let header = block.header;
        let block_hash = header.hash.to_string();
        let block_timestamp = header.timestamp();

        insert_cursor(
            db,
            &contract_address,
            start_block as i64,
            &block_hash,
            block_timestamp as i64,
            chain_id,
        )
        .await
        .context("Failed to insert into pool state")?;
        info!(start_block, "Starting indexer from block");
        Ok(start_block)
    }
}

pub async fn sync_events(
    pool: &DatabaseConnection,
    contract: Address,
    provider: &impl Provider,
    last_indexed_block: u64,
    chain_id: i64,
) -> Result<u64> {
    let latest_block = provider
        .get_block_number()
        .await
        .context("Failed to fetch block number")?;

    if latest_block <= last_indexed_block {
        return Ok(last_indexed_block);
    }

    let contract_address = contract.to_string();
    let mut from_block = check_reorg(pool, &contract_address, chain_id, provider)
        .await
        .context("Failed to check/handle reorg")?
        + 1;

    let batch_size = 10u64;
    let interface = IStabilizer::new(contract, &provider);

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
                    chain_id,
                )
                .await
                .context("Failed to insert liquidity event")?;
                insert_stabilizer_state(
                    pool,
                    &contract_address,
                    &usdc_reserve,
                    &usdt_reserve,
                    &usdc_price,
                    &usdt_price,
                    block_number,
                    &block_hash,
                    block_timestamp,
                    &tx_hash,
                    log_index,
                    chain_id,
                )
                .await
                .context("Failed to insert stabilizer state")?;
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
                    chain_id,
                )
                .await
                .context("Failed to insert liquidity event")?;
                insert_stabilizer_state(
                    pool,
                    &contract_address,
                    &usdc_reserve,
                    &usdt_reserve,
                    &usdc_price,
                    &usdt_price,
                    block_number,
                    &block_hash,
                    block_timestamp,
                    &tx_hash,
                    log_index,
                    chain_id,
                )
                .await
                .context("Failed to insert stabilizer state")?;
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
                insert_stabilizer_state(
                    pool,
                    &contract_address,
                    &usdc_reserve,
                    &usdt_reserve,
                    &usdc_price,
                    &usdt_price,
                    block_number,
                    &block_hash,
                    block_timestamp,
                    &tx_hash,
                    log_index,
                    chain_id,
                )
                .await
                .context("Failed to insert stabilizer state")?;
                block_event = true;
            }

            if block_event {
                insert_block(
                    pool,
                    &contract_address,
                    block_number,
                    &block_hash,
                    block_timestamp,
                    chain_id,
                )
                .await
                .context("Failed to add block")?;
            }
        }

        let block_header = provider
            .get_block_by_number(Number(to_block))
            .await
            .context("Failed to fetch block")?
            .unwrap()
            .header;
        let block_hash = block_header.hash.to_string();
        let block_timestamp = block_header.timestamp as i64;
        update_cursor(
            pool,
            &contract_address,
            chain_id,
            to_block as i64,
            &block_hash,
            block_timestamp,
        )
        .await
        .context("Failed to update cursor")?;

        from_block = to_block + 1;
    }
    Ok(latest_block)
}
