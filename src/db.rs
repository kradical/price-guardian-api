use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use std::env;

pub type PgConnectionManager = ConnectionManager<PgConnection>;
pub type PgPool = Pool<PgConnectionManager>;
pub type PgPooledConnection = PooledConnection<PgConnectionManager>;

fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn build_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_pool(&database_url).expect("Failed to create pool")
}
