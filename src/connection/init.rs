use anyhow::{Context, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn init_db(url: &str) -> Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(2);

    let db: DatabaseConnection = Database::connect(opt)
        .await
        .context("Failed to connect database")?;

    Migrator::up(&db, None).await?;
    Ok(db)
}
