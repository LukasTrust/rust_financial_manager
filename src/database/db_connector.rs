use rocket_db_pools::{diesel, Database};

#[derive(Database)]
#[database("postgres_db")]
pub struct DbConn(diesel::PgPool);
