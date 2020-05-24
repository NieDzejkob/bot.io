use anyhow::{Context as _, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use serde::Deserialize;
use serenity::prelude::*;

#[derive(Deserialize, Default)]
pub struct DatabaseConfig {
    pool_size: Option<u32>,
}

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
type Connection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn connect(config: &DatabaseConfig) -> Result<Pool> {
    let database_url = dotenv::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let manager = ConnectionManager::new(database_url);
    let mut pool_builder = Pool::builder();

    if let Some(size) = config.pool_size {
        pool_builder = pool_builder.max_size(size);
    }

    pool_builder.build(manager).context("Create database pool")
}

pub fn get_connection(ctx: &mut Context) -> Result<Connection> {
    ctx.data.read().get::<DB>().unwrap().get().context("Get database connection from pool")
}

/// Token struct for use in serenity's `Client::data`
pub struct DB;
impl TypeMapKey for DB {
    type Value = Pool;
}
