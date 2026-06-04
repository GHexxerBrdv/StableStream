pub const POOL_STATE: &str = "
  CREATE TABLE IF NOT EXISTS pool_state (
    pool_address TEXT PRIMARY KEY,
    usdc_reserve TEXT NOT NULL,
    usdt_reserve TEXT NOT NULL,
    last_indexed_block INTEGER NOT NULL
  );
";

pub const SWAPS: &str = "
  CREATE TABLE IF NOT EXISTS swaps (
    tx_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    token TEXT NOT NULL,
    amount TEXT NOT NULL,
    quote_amount TEXT NOT NULL,
    fees TEXT NOT NULL,
    receiver TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    PRIMARY KEY (tx_hash, log_index)
  );
";

pub const LIQUIDITY_EVENTS: &str = "
  CREATE TABLE IF NOT EXISTS liquidity_events (
    tx_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    amount_usdc TEXT NOT NULL,
    amount_usdt TEXT NOT NULL,
    amount_stb TEXT NOT NULL,
    receiver TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    PRIMARY KEY (tx_hash, log_index)
  );
";
