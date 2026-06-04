use anyhow::{Context, Result};

mod connection;
use connection::init::init_db;

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = "sqlite://./db/indexer.db";
    let pool = init_db(db_url)
        .await
        .with_context(|| "Failed to initialize database")?;
    println!("Pool successfully cretaed: {:?}", pool);
    Ok(())
}
