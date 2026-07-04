use anyhow::{Context, Result};
use std::time::Duration;
mod connection;
mod controllers;
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    sol,
};
use connection::init::init_db;
use controllers::engine::*;
use tracing::{error, info};

sol!(
    #[sol(rpc)]
    contract IStabilizer {
        event LiquidityAdded(uint256 amountUsdc, uint256 amountUsdt, uint256 amountStb, address receiver);
        event LiquidityRemoved(uint256 amountUsdc, uint256 amountUsdt, uint256 amountStb, address receiver);
        event Exchange(
            address token, uint256 amount, uint256 quoteAmount, uint256 fees, address receiver, address feeReceiver
        );
        function getStabilizerMatrix()
            external
            returns (uint256 usdcReserveAmount, uint256 usdtReserveAmount, uint256 usdcPrice, uint256 usdtPrice);
    }
);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = init_db(&db_url)
        .await
        .with_context(|| "Failed to initialize database")?;
    info!("Pool successfully created: {:?}", pool);
    let contract: Address = std::env::var("CONTRACT_ADDRESS")
        .with_context(|| "missing contract address in .env")?
        .parse()
        .with_context(|| "Invalid contract address")?;
    let rpc_url = std::env::var("RPC_URL")
        .with_context(|| "Missing rpc-url")?
        .parse()
        .with_context(|| "Invalid rpc-url")?;

    let provider = ProviderBuilder::new().connect_http(rpc_url);
    let chain_id = provider
        .get_chain_id()
        .await
        .context("Failed to fetch chain id")?;
    let mut last_indexed_block =
        get_or_create_cursor(&pool, contract, &provider, chain_id as i64).await?;
    info!(last_indexed_block, "Starting indexer at block height ...");
    loop {
        match sync_events(
            &pool,
            contract,
            &provider,
            last_indexed_block,
            chain_id as i64,
        )
        .await
        {
            Ok(new_block) => last_indexed_block = new_block,
            Err(e) => {
                error!("Error in indexer sync: {:?}", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}
