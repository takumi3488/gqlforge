pub mod data_loader;
#[cfg(feature = "postgres")]
pub mod introspector;
pub mod request_template;
pub mod schema;
#[cfg(feature = "postgres")]
pub mod sql_parser;

pub use request_template::RequestTemplate;
pub use schema::DatabaseSchema;

use async_graphql_value::ConstValue;

/// Trait for executing SQL queries against PostgreSQL.
/// Concrete implementations live in the CLI crate (real connection pool)
/// or in test utilities (mock).
#[async_trait::async_trait]
pub trait PostgresIO: Send + Sync + 'static {
    /// Execute a parameterised SQL query and return the result rows as a
    /// `ConstValue` (typically a JSON array of objects).
    async fn execute(&self, query: &str, params: &[String]) -> anyhow::Result<ConstValue>;
}
