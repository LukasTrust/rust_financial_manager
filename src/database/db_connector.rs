use rocket_db_pools::{diesel, Database};

/// Database connection pool
/// This struct is used to create a connection pool to the database
/// and is used to create a connection to the database
#[derive(Database)]
#[database("postgres_db")]
pub struct DbConn(diesel::PgPool);
