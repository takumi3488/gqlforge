use async_graphql_value::ConstValue;
use deadpool_postgres::{Config, Pool, Runtime};
use indexmap::IndexMap;

use super::conversion::{row_value_to_const, sanitize_graphql_name};
use super::types::TypedParam;
use crate::core::postgres::PostgresIO;

/// A connection pool backed by `deadpool-postgres`.
pub struct PostgresPool {
    pool: Pool,
}

impl PostgresPool {
    /// Create a new pool from a PostgreSQL connection string.
    pub fn new(connection_url: &str) -> anyhow::Result<Self> {
        let mut cfg = Config::new();
        cfg.url = Some(connection_url.to_string());

        let tls = crate::core::postgres::make_tls_connect()?;
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), tls)
            .map_err(|e| anyhow::anyhow!("Failed to create PostgreSQL pool: {e}"))?;

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl PostgresIO for PostgresPool {
    async fn execute(&self, query: &str, params: &[String]) -> anyhow::Result<ConstValue> {
        let client = self.pool.get().await?;

        // Convert String params via TypedParam for correct PostgreSQL type encoding.
        let typed_params: Vec<TypedParam> = params.iter().map(|p| TypedParam(p.clone())).collect();
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = typed_params
            .iter()
            .map(|p| p as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client.query(query, &param_refs).await?;

        // Convert rows to ConstValue (JSON array of objects).
        let mut result = Vec::new();
        for row in &rows {
            let mut obj = IndexMap::new();
            for (i, col) in row.columns().iter().enumerate() {
                let value = row_value_to_const(row, i, col)?;
                obj.insert(
                    async_graphql::Name::new(sanitize_graphql_name(col.name())),
                    value,
                );
            }
            result.push(ConstValue::Object(obj));
        }

        Ok(ConstValue::List(result))
    }
}
