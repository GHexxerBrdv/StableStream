use crate::{
    IStabilizer,
    connection::queries::interact::{
        get_last_indexed_block, insert_liquidity_event, insert_pool_state, insert_swap_event,
        update_pool_state,
    },
};
use alloy::{primitives::Address, providers::Provider, rpc::types::eth::Filter};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tracing::info;

pub async fn get_or_create_cursor(
    pool: &SqlitePool,
    pool_address: Address,
    provider: &impl Provider,
) -> Result<u64> {
    let pool_str = pool_address.to_string();

    if let Some(block) = get_last_indexed_block(pool, &pool_str)
        .await
        .context("Failed to read last indexed block")?
    {
        Ok(block as u64)
    } else {
        let latest_block = provider
            .get_block_number()
            .await
            .context("Failed to fetch block number")?;
        insert_pool_state(pool, &pool_str, latest_block as i64)
            .await
            .context("Failed to insert into pool state")?;
        info!(block = latest_block, "Starting indexer from latest block");
        Ok(latest_block)
    }
}

pub async fn sync_events(
    pool: &SqlitePool,
    pool_address: Address,
    provider: &impl Provider,
    last_indexed_block: u64,
) -> Result<u64> {
    let latest_block = provider
        .get_block_number()
        .await
        .context("Failed to fetch block number")?;

    if latest_block <= last_indexed_block {
        return Ok(last_indexed_block);
    }

    let batch_size = 10u64;
    let mut from_block = last_indexed_block + 1;
    let pool_str = pool_address.to_string();

    while from_block <= latest_block {
        let to_block = std::cmp::min(from_block + batch_size - 1, latest_block);

        let filter = Filter::new()
            .address(pool_address)
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

            if let Ok(event) = log.log_decode::<IStabilizer::LiquidityAdded>() {
                let amount_usdc = event.data().amountUsdc.to_string();
                let amount_usdt = event.data().amountUsdt.to_string();
                let amount_stb = event.data().amountStb.to_string();
                let receiver = event.data().receiver.to_string();

                info!(tx_hash, "Liquidity added detected");
                insert_liquidity_event(
                    pool,
                    &tx_hash,
                    log_index,
                    "ADD",
                    &amount_usdc,
                    &amount_usdt,
                    &amount_stb,
                    &receiver,
                    block_number,
                )
                .await
                .context("Failed to insert liquidity event")?;
                batch_has_events = true;
            } else if let Ok(event) = log.log_decode::<IStabilizer::LiquidityRemoved>() {
                let amount_usdc = event.data().amountUsdc.to_string();
                let amount_usdt = event.data().amountUsdt.to_string();
                let amount_stb = event.data().amountStb.to_string();
                let receiver = event.data().receiver.to_string();
                info!(tx_hash, "Liquidity removed detected");
                insert_liquidity_event(
                    pool,
                    &tx_hash,
                    log_index,
                    "REMOVE",
                    &amount_usdc,
                    &amount_usdt,
                    &amount_stb,
                    &receiver,
                    block_number,
                )
                .await
                .context("Failed to insert liquidity event")?;

                batch_has_events = true;
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
                    log_index,
                    &token,
                    &amount,
                    &quote_amount,
                    &fees,
                    &receiver,
                    block_number,
                )
                .await
                .context("Failed to insert swap event")?;
                batch_has_events = true;
            }
        }

        if batch_has_events {
            let contract = IStabilizer::new(pool_address, &provider);
            let IStabilizer::getStabilizerMatrixReturn {
                usdcReserveAmount,
                usdtReserveAmount,
                ..
            } = contract
                .getStabilizerMatrix()
                .call()
                .await
                .context("Failed to fetch stabilizer matrix from blockchain")?;

            let usdc_str = usdcReserveAmount.to_string();
            let usdt_str = usdtReserveAmount.to_string();
            info!(
                usdc = usdc_str.as_str(),
                usdt = usdt_str.as_str(),
                "Updating pool reserve in DB"
            );
            update_pool_state(pool, &pool_str, &usdc_str, &usdt_str, to_block as i64)
                .await
                .context("Failed to update pool state")?;
        }
        from_block = to_block + 1;
    }
    Ok(latest_block)
}
