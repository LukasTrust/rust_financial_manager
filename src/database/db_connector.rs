use rocket_db_pools::{diesel, Database};

#[derive(Database)]
#[database("postgres_db")]
pub struct DbConn(diesel::PgPool);

#[derive(Database)]
#[database("test_db")]
pub struct TestDbConn(diesel::PgPool);
