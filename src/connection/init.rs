use anyhow::{Context, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::info;

pub async fn init_db(url: &str) -> Result<DatabaseConnection> {
    info!("Connecting to database");
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(2);

    let db: DatabaseConnection = Database::connect(opt)
        .await
        .context("Failed to connect database")?;
    info!("Database connected");
    
    Migrator::up(&db, None).await?;
    
    Ok(db)
}
