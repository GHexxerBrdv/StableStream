use anyhow::{Context, Result};
use sqlx::{SqlitePool};
use std::time::Duration;
mod connection;
use alloy::{
    primitives::{Address, address},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::Filter,
    sol
};
use connection::init::init_db;
use tracing::{error, info};

sol!(
    #[sol(rpc)]
    contract IStabilizer {
        event LiquidityAdded(uint256 amountUsdc, uint256 amountUsdt, uint256 amountStb, address receiver);
        event LiquidityRemoved(uint256 amountUsdc, uint256 amountUsdt, uint256 amountStb, address receiver);
        event Exchange(
            address token, uint256 amount, uint256 quoteAmount, uint256 fees, address receiver, address feeReceiver
        );
        event OracleUpdate(address oldOracle, address newOracle);
        event AmpUpdate(uint256 amp);
        event FeeReceiverUpdate(address oldFeeReceiver, address newFeeReceiver);
        event CleanUp(address token, uint256 amount);
        event MaxImbalanceThresholdUpdate(uint256 oldThreshold, uint256 newThreshold);
        event MaxPriceDeviationThresholdUpdate(uint256 oldThreshold, uint256 newThreshold);

        function addLiquidity(uint256 amountUsdc, uint256 amountUsdt, uint256 minAmountStb, address receiver) external;
        function removeLiquidity(uint256 amountStb, uint256 minAmountUsdc, uint256 minAmountUsdt, address receiver) external;
        function exchange(address token, uint256 amount, uint256 minAmountOut, address receiver) external;
        function getStabilizerMatrix()
            external
            returns (uint256 usdcReserveAmount, uint256 usdtReserveAmount, uint256 usdcPrice, uint256 usdtPrice);
    }
);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./db/indexer.db".to_string());
    let pool = init_db(&db_url)
        .await
        .with_context(|| "Failed to initialize database")?;
    println!("Pool successfully created: {:?}", pool);

    let pool_address = address!("0xEb1598206b58D87d671d137228712d8919914D16"); // put deployed contract address here
    let rpc_url = std::env::var("POL")?.parse()?;
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let mut last_indexed_block = get_or_create_cursor(&pool, pool_address, &provider).await?;
    info!(last_indexed_block, "Starting indexer at block height ...");
    loop {
        match sync_events(&pool, pool_address, &provider, last_indexed_block).await {
            Ok(new_block) => last_indexed_block = new_block,
            Err(e) => {
                error!("Error in indexer sync: {:?}", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn get_or_create_cursor(pool: &SqlitePool, pool_address: Address, provider: &impl Provider) -> Result<u64> {
    let pool_str = pool_address.to_string();

    let row = sqlx::query!(
        "SELECT last_indexed_block FROM pool_state WHERE pool_address = ?",
        pool_str
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(r.last_indexed_block as u64),
        None => {
            let latest_block = provider.get_block_number().await?;
            sqlx::query!("INSERT INTO pool_state (pool_address, usdc_reserve, usdt_reserve, last_indexed_block) VALUES (?, '0', '0', ?)", pool_str, latest_block as i64)
                .execute(pool)
                .await?;
            info!(block = latest_block, "Starting indexer from latest block");
            Ok(latest_block)
        }
    }
}

async fn sync_events(
    pool: &SqlitePool,
    pool_address: Address,
    provider: &impl Provider,
    last_indexed_block: u64,
) -> Result<u64> {
    let latest_block = provider.get_block_number().await?;

    if latest_block <= last_indexed_block {
        return Ok(last_indexed_block);
    }

    let from_block = last_indexed_block + 1;
    let to_block = latest_block;

    info!(from_block, to_block, "Scanning block for events");

    let filter = Filter::new()
        .address(pool_address)
        .from_block(from_block)
        .to_block(to_block);

    let logs = provider.get_logs(&filter).await?;

    let mut state_changed = false;

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
            sqlx::query!(
                "INSERT INTO liquidity_events (tx_hash, log_index, event_type, amount_usdc, amount_usdt, amount_stb, receiver, block_number) VALUES (?, ?, 'ADD', ?, ?, ?, ?, ?)",
                tx_hash, log_index, amount_usdc, amount_usdt, amount_stb, receiver, block_number
            )
            .execute(pool)
            .await?;
            state_changed = true;
        } else if let Ok(event) = log.log_decode::<IStabilizer::LiquidityRemoved>() {
            let amount_usdc = event.data().amountUsdc.to_string();
            let amount_usdt = event.data().amountUsdt.to_string();
            let amount_stb = event.data().amountStb.to_string();
            let receiver = event.data().receiver.to_string();
            info!(tx_hash, "Liquidity removed detected");
            sqlx::query!(
                "INSERT INTO liquidity_events (tx_hash, log_index, event_type, amount_usdc, amount_usdt, amount_stb, receiver, block_number) VALUES (?, ?, 'REMOVE', ?, ?, ?, ?, ?)",
                tx_hash, log_index, amount_usdc, amount_usdt, amount_stb, receiver, block_number
            )
            .execute(pool)
            .await?;
            state_changed = true;
        } else if let Ok(event) = log.log_decode::<IStabilizer::Exchange>() {
            let token = event.data().token.to_string();
            let amount = event.data().amount.to_string();
            let quote_amount = event.data().quoteAmount.to_string();
            let fees = event.data().fees.to_string();
            let receiver = event.data().receiver.to_string();
            info!(tx_hash, "Exchange detected");
            sqlx::query!(
                "INSERT INTO swaps (tx_hash, log_index, token, amount, quote_amount, fees, receiver, block_number) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                tx_hash, log_index, token, amount, quote_amount, fees, receiver, block_number
            )
            .execute(pool)
            .await?;
            state_changed = true;
        }

        if state_changed {
            let contract = IStabilizer::new(pool_address, &provider);
            let IStabilizer::getStabilizerMatrixReturn {
                usdcReserveAmount,
                usdtReserveAmount,
                ..
            } = contract.getStabilizerMatrix().call().await?;

            let usdc_str = usdcReserveAmount.to_string();
            let usdt_str = usdtReserveAmount.to_string();
            let pool_str = pool_address.to_string();
            info!(usdc = usdc_str.as_str(), usdt = usdt_str.as_str(), "Updating pool reserve in DB");
            sqlx::query!(
                "UPDATE pool_state SET usdc_reserve = ?, usdt_reserve = ?, last_indexed_block = ? WHERE pool_address = ?",
                usdc_str, usdt_str, to_block as i64, pool_str
            ).execute(pool).await?;
        } else {
            let pool_str = pool_address.to_string();
            sqlx::query!(
                "UPDATE pool_state SET last_indexed_block = ? WHERE pool_address = ?",
                to_block as i64,
                pool_str
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(to_block)
}
