use anyhow::{Context, Result};
use entity::{
    blocks::{ActiveModel as BlockActiveModel, Column as BlockColumn, Entity as BlockEntity},
    cursor::{ActiveModel as CursorActiveModel, Column as CursorColumn, Entity as CursorEntity},
    liquidity_events::{
        ActiveModel as LiquidityEventActiveModel, Column as LiquidityEventColumn,
        Entity as LiquidityEventEntity,
    },
    stabilizer_state::{
        ActiveModel as StabilizerStateActiveModel, Column as StabilizerStateColumn,
        Entity as StabilizerStateEntity,
    },
    swaps::{ActiveModel as SwapActiveModel, Column as SwapColumn, Entity as SwapEntity},
};
use migration::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

pub async fn insert_stabilizer_state(
    db: &DatabaseConnection,
    contract_address: &str,
    usdc_reserve: &str,
    usdt_reserve: &str,
    usdc_price: &str,
    usdt_price: &str,
    block_number: i64,
    block_hash: &str,
    block_timestamp: i64,
    tx_hash: &str,
    log_index: i64,
    chain_id: i64,
) -> Result<()> {
    // sqlx::query!(
    //     "INSERT INTO stabilizer_state (contract_address, usdc_reserve, usdt_reserve, usdc_price, usdt_price, block_number, block_hash, block_timestamp, tx_hash, log_index, chain_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    //     contract_address,
    //     usdc_reserve,
    //     usdt_reserve,
    //     usdc_price,
    //     usdt_price,
    //     block_number,
    //     block_hash,
    //     block_timestamp,
    //     tx_hash,
    //     log_index,
    //     chain_id,
    // )
    // .execute(pool)
    // .await
    // .context("Failed to insert into stabilizer state")?;
    let model = StabilizerStateActiveModel {
        contract_address: Set(contract_address.to_owned()),
        usdc_reserve: Set(usdc_reserve.to_owned()),
        usdt_reserve: Set(usdt_reserve.to_owned()),
        usdc_price: Set(usdc_price.to_owned()),
        usdt_price: Set(usdt_price.to_owned()),
        block_number: Set(block_number),
        block_hash: Set(block_hash.to_owned()),
        block_timestamp: Set(block_timestamp),
        tx_hash: Set(tx_hash.to_owned()),
        log_index: Set(log_index as i32),
        chain_id: Set(chain_id as i32),
    };
    model
        .insert(db)
        .await
        .context("Failed to insert into stabilizer state")?;
    Ok(())
}

pub async fn get_cursor(
    db: &DatabaseConnection,
    contract_address: &str,
    chain_id: i64,
) -> Result<Option<(i64, String)>> {
    let cursor = CursorEntity::find()
        .filter(CursorColumn::ContractAddress.eq(contract_address))
        .filter(CursorColumn::ChainId.eq(chain_id))
        .select_only()
        .column(CursorColumn::LastIndexedBlockNumber)
        .column(CursorColumn::LastIndexedBlockHash)
        .into_tuple()
        .one(db)
        .await
        .context("Failed to get cursor")?;
    Ok(cursor)
}

pub async fn insert_cursor(
    db: &DatabaseConnection,
    contract_address: &str,
    last_indexed_block_number: i64,
    last_indexed_block_hash: &str,
    last_indexed_block_timestamp: i64,
    chain_id: i64,
) -> Result<()> {
    let model = CursorActiveModel {
        contract_address: Set(contract_address.to_owned()),
        last_indexed_block_number: Set(last_indexed_block_number),
        last_indexed_block_hash: Set(last_indexed_block_hash.to_owned()),
        last_indexed_block_timestamp: Set(last_indexed_block_timestamp),
        chain_id: Set(chain_id as i32),
    };
    model
        .insert(db)
        .await
        .context("Failed to insert into cursor")?;
    Ok(())
}

pub async fn update_cursor(
    db: &DatabaseConnection,
    contract_address: &str,
    chain_id: i64,
    last_indexed_block_number: i64,
    last_indexed_block_hash: &str,
    last_indexed_block_timestamp: i64,
) -> Result<()> {
    // sqlx::query!(
    //     "UPDATE cursor SET last_indexed_block_number = ?, last_indexed_block_hash = ?, last_indexed_block_timestamp = ? WHERE contract_address = ? AND chain_id = ?",
    //     last_indexed_block_number,
    //     last_indexed_block_hash,
    //     last_indexed_block_timestamp,
    //     contract_address,
    //     chain_id,
    // )
    // .execute(pool)
    // .await
    // .with_context(|| format!("Failed to update cursor for contract: {contract_address} on chain: {chain_id}"))?;
    CursorEntity::update_many()
        .col_expr(
            CursorColumn::LastIndexedBlockNumber,
            Expr::val(last_indexed_block_number),
        )
        .col_expr(
            CursorColumn::LastIndexedBlockHash,
            Expr::val(last_indexed_block_hash.to_owned()),
        )
        .col_expr(
            CursorColumn::LastIndexedBlockTimestamp,
            Expr::val(last_indexed_block_timestamp),
        )
        .filter(CursorColumn::ContractAddress.eq(contract_address))
        .filter(CursorColumn::ChainId.eq(chain_id))
        .exec(db)
        .await
        .with_context(|| {
            format!("Failed to update cursor for contract: {contract_address} on chain: {chain_id}")
        })?;
    Ok(())
}

pub async fn get_blocks(
    db: &DatabaseConnection,
    contract_address: &str,
    chain_id: i64,
) -> Result<Option<Vec<(i64, String)>>> {
    // let rows = sqlx::query!(
    //     "SELECT block_number, block_hash FROM blocks WHERE contract_address = ? AND  chain_id = ? ORDER BY block_number DESC LIMIT 10",
    //     contract_address,
    //     chain_id
    // )
    // .fetch_all(pool)
    // .await
    // .context("Failed to fetch content")?;
    // let blocks = rows
    //     .into_iter()
    //     .map(|r| (r.block_number, r.block_hash))
    //     .collect();

    let blocks: Vec<(i64, String)> = BlockEntity::find()
        .filter(BlockColumn::ContractAddress.eq(contract_address))
        .filter(BlockColumn::ChainId.eq(chain_id))
        .order_by_desc(BlockColumn::BlockNumber)
        .limit(10)
        .select_only()
        .column(BlockColumn::BlockNumber)
        .column(BlockColumn::BlockHash)
        .into_tuple()
        .all(db)
        .await
        .context("Failed to get blocks")?;
    Ok(Some(blocks))
}

pub async fn insert_block(
    db: &DatabaseConnection,
    contract_address: &str,
    block_number: i64,
    block_hash: &str,
    block_timestamp: i64,
    chain_id: i64,
) -> Result<()> {
    // sqlx::query!(
    //     "INSERT INTO blocks (contract_address, block_number, block_hash, block_timestamp, chain_id) VALUES (?, ?, ?, ?, ?)",
    //     contract_address,
    //     block_number,
    //     block_hash,
    //     block_timestamp,
    //     chain_id,
    // )
    // .execute(pool)
    // .await
    // .context("Failed to insert block")?;
    let model = BlockActiveModel {
        contract_address: Set(contract_address.to_owned()),
        block_number: Set(block_number),
        block_hash: Set(block_hash.to_owned()),
        block_timestamp: Set(block_timestamp),
        chain_id: Set(chain_id as i32),
    };
    model.insert(db).await.context("Failed to insert block")?;
    Ok(())
}

pub async fn insert_swap_event(
    db: &DatabaseConnection,
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
    // sqlx::query!(
    //     "INSERT INTO swaps (tx_hash, block_hash, log_index, contract_address, token, amount, quote_amount, fees, receiver, block_number, block_timestamp, chain_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    //     tx_hash,
    //     block_hash,
    //     log_index,
    //     contract_address,
    //     token,
    //     amount,
    //     quote_amount,
    //     fees,
    //     receiver,
    //     block_number,
    //     block_timestamp,
    //     chain_id,
    // )
    // .execute(pool)
    // .await
    // .with_context(|| "Failed to insert into swap table")?;
    let model = SwapActiveModel {
        tx_hash: Set(tx_hash.to_owned()),
        block_hash: Set(block_hash.to_owned()),
        log_index: Set(log_index as i32),
        contract_address: Set(contract_address.to_owned()),
        token: Set(token.to_owned()),
        amount: Set(amount.to_owned()),
        quote_amount: Set(quote_amount.to_owned()),
        fees: Set(fees.to_owned()),
        receiver: Set(receiver.to_owned()),
        block_number: Set(block_number),
        block_timestamp: Set(block_timestamp),
        chain_id: Set(chain_id as i32),
    };
    model.insert(db).await.context("Failed to insert swap")?;
    Ok(())
}

pub async fn insert_liquidity_event(
    db: &DatabaseConnection,
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
    // sqlx::query!(
    //     "INSERT INTO liquidity_events (tx_hash, block_hash, log_index, contract_address, event_type, amount_usdc, amount_usdt, amount_stb, receiver, block_number, block_timestamp, chain_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    //     tx_hash,
    //     block_hash,
    //     log_index,
    //     contract_address,
    //     event_type,
    //     amount_usdc,
    //     amount_usdt,
    //     amount_stb,
    //     receiver,
    //     block_number,
    //     block_timestamp,
    //     chain_id,
    // )
    // .execute(pool)
    // .await
    // .with_context(|| "Failed to insert into liquidity event table")?;
    let model = LiquidityEventActiveModel {
        tx_hash: Set(tx_hash.to_owned()),
        block_hash: Set(block_hash.to_owned()),
        log_index: Set(log_index as i32),
        contract_address: Set(contract_address.to_owned()),
        event_type: Set(event_type.to_owned()),
        amount_usdc: Set(amount_usdc.to_owned()),
        amount_usdt: Set(amount_usdt.to_owned()),
        amount_stb: Set(amount_stb.to_owned()),
        receiver: Set(receiver.to_owned()),
        block_number: Set(block_number),
        block_timestamp: Set(block_timestamp),
        chain_id: Set(chain_id as i32),
    };
    model
        .insert(db)
        .await
        .context("Failed to insert liquidity event")?;
    Ok(())
}

pub async fn perform_removal(db: &DatabaseConnection, fork_point: i64) -> Result<()> {
    // sqlx::query!(
    //     "DELETE FROM liquidity_events where block_number > ?",
    //     fork_point
    // )
    // .execute(pool)
    // .await
    // .context("Failed to perform removal")?;
    LiquidityEventEntity::delete_many()
        .filter(LiquidityEventColumn::BlockNumber.gt(fork_point))
        .exec(db)
        .await
        .context("Failed to perform removal from liquidity_events")?;
    // sqlx::query!("DELETE FROM swaps where block_number > ?", fork_point)
    //     .execute(pool)
    //     .await
    //     .context("Failed to perform removal")?;
    SwapEntity::delete_many()
        .filter(SwapColumn::BlockNumber.gt(fork_point))
        .exec(db)
        .await
        .context("Failed to perform removal from swaps")?;
    // sqlx::query!("DELETE FROM blocks where block_number > ?", fork_point)
    //     .execute(pool)
    //     .await
    //     .context("Failed to perform removal")?;
    BlockEntity::delete_many()
        .filter(BlockColumn::BlockNumber.gt(fork_point))
        .exec(db)
        .await
        .context("Failed to perform removal from blocks")?;
    // sqlx::query!(
    //     "DELETE FROM stabilizer_state where block_number > ?",
    //     fork_point
    // )
    // .execute(pool)
    // .await
    // .context("Failed to perform removal")?;
    StabilizerStateEntity::delete_many()
        .filter(StabilizerStateColumn::BlockNumber.gt(fork_point))
        .exec(db)
        .await
        .context("Failed to perform removal from stabilizer_state")?;
    Ok(())
}
