pub mod data_loader;
#[cfg(feature = "postgres")]
pub mod introspector;
pub mod request_template;
pub mod schema;
#[cfg(feature = "postgres")]
pub mod sql_parser;

use async_graphql_value::ConstValue;
pub use request_template::RequestTemplate;
pub use schema::DatabaseSchema;

/// Build a rustls-based TLS connector for PostgreSQL connections.
#[cfg(feature = "postgres")]
pub(crate) fn make_tls_connect() -> anyhow::Result<tokio_postgres_rustls::MakeRustlsConnect> {
    let native = rustls_native_certs::load_native_certs();
    let mut root_store = rustls::RootCertStore::empty();
    for cert in native.certs {
        let _ = root_store.add(cert);
    }
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(tokio_postgres_rustls::MakeRustlsConnect::new(config))
}

/// Trait for executing SQL queries against PostgreSQL.
/// Concrete implementations live in the CLI crate (real connection pool)
/// or in test utilities (mock).
#[async_trait::async_trait]
pub trait PostgresIO: Send + Sync + 'static {
    /// Execute a parameterised SQL query and return the result rows as a
    /// `ConstValue` (typically a JSON array of objects).
    async fn execute(&self, query: &str, params: &[String]) -> anyhow::Result<ConstValue>;
}
