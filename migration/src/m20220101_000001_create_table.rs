use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StabilizerState::Table)
                    .if_not_exists()
                    .col(string(StabilizerState::ContractAddress))
                    .col(string(StabilizerState::UsdcReserve))
                    .col(string(StabilizerState::UsdtReserve))
                    .col(string(StabilizerState::UsdcPrice))
                    .col(string(StabilizerState::UsdtPrice))
                    .col(big_integer(StabilizerState::BlockNumber))
                    .col(string(StabilizerState::BlockHash))
                    .col(big_integer(StabilizerState::BlockTimestamp))
                    .col(string(StabilizerState::TxHash))
                    .col(integer(StabilizerState::LogIndex))
                    .col(integer(StabilizerState::ChainId))
                    .primary_key(
                        Index::create()
                            .col(StabilizerState::ChainId)
                            .col(StabilizerState::TxHash)
                            .col(StabilizerState::LogIndex),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Cursor::Table)
                    .if_not_exists()
                    .col(string(Cursor::ContractAddress))
                    .col(big_integer(Cursor::LastIndexedBlockNumber))
                    .col(string(Cursor::LastIndexedBlockHash))
                    .col(big_integer(Cursor::LastIndexedBlockTimestamp))
                    .col(integer(Cursor::ChainId))
                    .primary_key(
                        Index::create()
                            .col(Cursor::ChainId)
                            .col(Cursor::ContractAddress),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Blocks::Table)
                    .if_not_exists()
                    .col(string(Blocks::ContractAddress))
                    .col(big_integer(Blocks::BlockNumber))
                    .col(string(Blocks::BlockHash))
                    .col(big_integer(Blocks::BlockTimestamp))
                    .col(integer(Blocks::ChainId))
                    .primary_key(
                        Index::create()
                            .col(Blocks::ChainId)
                            .col(Blocks::BlockNumber),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Swaps::Table)
                    .if_not_exists()
                    .col(string(Swaps::TxHash))
                    .col(string(Swaps::BlockHash))
                    .col(integer(Swaps::LogIndex))
                    .col(string(Swaps::ContractAddress))
                    .col(string(Swaps::Token))
                    .col(string(Swaps::Amount))
                    .col(string(Swaps::QuoteAmount))
                    .col(string(Swaps::Fees))
                    .col(string(Swaps::Receiver))
                    .col(big_integer(Swaps::BlockNumber))
                    .col(big_integer(Swaps::BlockTimestamp))
                    .col(integer(Swaps::ChainId))
                    .primary_key(
                        Index::create()
                            .col(Swaps::ChainId)
                            .col(Swaps::TxHash)
                            .col(Swaps::LogIndex),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(LiquidityEvents::Table)
                    .if_not_exists()
                    .col(string(LiquidityEvents::TxHash))
                    .col(string(LiquidityEvents::BlockHash))
                    .col(integer(LiquidityEvents::LogIndex))
                    .col(string(LiquidityEvents::ContractAddress))
                    .col(string(LiquidityEvents::EventType))
                    .col(string(LiquidityEvents::AmountUsdc))
                    .col(string(LiquidityEvents::AmountUsdt))
                    .col(string(LiquidityEvents::AmountStb))
                    .col(string(LiquidityEvents::Receiver))
                    .col(big_integer(LiquidityEvents::BlockNumber))
                    .col(big_integer(LiquidityEvents::BlockTimestamp))
                    .col(integer(LiquidityEvents::ChainId))
                    .primary_key(
                        Index::create()
                            .col(LiquidityEvents::ChainId)
                            .col(LiquidityEvents::TxHash)
                            .col(LiquidityEvents::LogIndex),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop tables in reverse order
        manager
            .drop_table(Table::drop().table(LiquidityEvents::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Swaps::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Blocks::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Cursor::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(StabilizerState::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum StabilizerState {
    #[sea_orm(iden = "stabilizer_state")]
    Table,
    ContractAddress,
    UsdcReserve,
    UsdtReserve,
    UsdcPrice,
    UsdtPrice,
    BlockNumber,
    BlockHash,
    BlockTimestamp,
    TxHash,
    LogIndex,
    ChainId,
}

#[derive(DeriveIden)]
enum Cursor {
    #[sea_orm(iden = "cursor")]
    Table,
    ContractAddress,
    LastIndexedBlockNumber,
    LastIndexedBlockHash,
    LastIndexedBlockTimestamp,
    ChainId,
}

#[derive(DeriveIden)]
enum Blocks {
    #[sea_orm(iden = "blocks")]
    Table,
    ContractAddress,
    BlockNumber,
    BlockHash,
    BlockTimestamp,
    ChainId,
}

#[derive(DeriveIden)]
enum Swaps {
    #[sea_orm(iden = "swaps")]
    Table,
    TxHash,
    BlockHash,
    LogIndex,
    ContractAddress,
    Token,
    Amount,
    QuoteAmount,
    Fees,
    Receiver,
    BlockNumber,
    BlockTimestamp,
    ChainId,
}

#[derive(DeriveIden)]
enum LiquidityEvents {
    #[sea_orm(iden = "liquidity_events")]
    Table,
    TxHash,
    BlockHash,
    LogIndex,
    ContractAddress,
    EventType,
    AmountUsdc,
    AmountUsdt,
    AmountStb,
    Receiver,
    BlockNumber,
    BlockTimestamp,
    ChainId,
}
