// pub const STABILIZER_STATE: &str = "
//   CREATE TABLE IF NOT EXISTS stabilizer_state (
//     contract_address TEXT NOT NULL,
//     usdc_reserve TEXT NOT NULL,
//     usdt_reserve TEXT NOT NULL,
//     usdc_price TEXT NOT NULL,
//     usdt_price TEXT NOT NULL,
//     block_number INTEGER NOT NULL,
//     block_hash TEXT NOT NULL,
//     block_timestamp INTEGER NOT NULL,
//     tx_hash TEXT NOT NULL,
//     log_index INTEGER NOT NULL,
//     chain_id INTEGER NOT NULL,
//     PRIMARY KEY (chain_id, tx_hash, log_index)
//   );
// ";

// pub const CURSOR: &str = "
//   CREATE TABLE IF NOT EXISTS cursor (
//     contract_address TEXT NOT NULL,
//     last_indexed_block_number INTEGER NOT NULL,
//     last_indexed_block_hash TEXT NOT NULL,
//     last_indexed_block_timestamp INTEGER NOT NULL,
//     chain_id INTEGER NOT NULL,
//     PRIMARY KEY (chain_id, contract_address)
//   );
// ";

// pub const BLOCKS: &str = "
//   CREATE TABLE IF NOT EXISTS blocks (
//     contract_address TEXT NOT NULL,
//     block_number INTEGER NOT NULL,
//     block_hash TEXT NOT NULL,
//     block_timestamp INTEGER NOT NULL,
//     chain_id INTEGER NOT NULL,
//     PRIMARY KEY (chain_id, block_number)
//   );
// ";

// pub const SWAPS: &str = "
//   CREATE TABLE IF NOT EXISTS swaps (
//     tx_hash TEXT NOT NULL,
//     block_hash TEXT NOT NULL,
//     log_index INTEGER NOT NULL,
//     contract_address TEXT NOT NULL,
//     token TEXT NOT NULL,
//     amount TEXT NOT NULL,
//     quote_amount TEXT NOT NULL,
//     fees TEXT NOT NULL,
//     receiver TEXT NOT NULL,
//     block_number INTEGER NOT NULL,
//     block_timestamp INTEGER NOT NULL,
//     chain_id INTEGER NOT NULL,
//     PRIMARY KEY (chain_id, tx_hash, log_index)
//   );
// ";

// pub const LIQUIDITY_EVENTS: &str = "
//   CREATE TABLE IF NOT EXISTS liquidity_events (
//     tx_hash TEXT NOT NULL,
//     block_hash TEXT NOT NULL,
//     log_index INTEGER NOT NULL,
//     contract_address TEXT NOT NULL,
//     event_type TEXT NOT NULL,
//     amount_usdc TEXT NOT NULL,
//     amount_usdt TEXT NOT NULL,
//     amount_stb TEXT NOT NULL,
//     receiver TEXT NOT NULL,
//     block_number INTEGER NOT NULL,
//     block_timestamp INTEGER NOT NULL,
//     chain_id INTEGER NOT NULL,
//     PRIMARY KEY (chain_id, tx_hash, log_index)
//   );
// ";
