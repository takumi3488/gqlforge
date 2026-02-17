#[cfg(feature = "postgres")]
pub mod pool {
    use async_graphql_value::ConstValue;
    use deadpool_postgres::{Config, Pool, Runtime};
    use indexmap::IndexMap;

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

            // Convert String params to &(dyn ToSql + Sync) references.
            let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
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

    fn sanitize_graphql_name(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        for c in name.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                result.push(c);
            } else {
                result.push('_');
            }
        }
        if result.starts_with(|c: char| c.is_ascii_digit()) {
            result.insert(0, '_');
        }
        if result.is_empty() {
            result.push_str("_unnamed");
        }
        result
    }

    fn row_value_to_const(
        row: &tokio_postgres::Row,
        idx: usize,
        col: &tokio_postgres::Column,
    ) -> anyhow::Result<ConstValue> {
        use tokio_postgres::types::Type;

        let ty = col.type_();

        match *ty {
            Type::BOOL => {
                let v: Option<bool> = row.try_get(idx)?;
                Ok(v.map(ConstValue::Boolean).unwrap_or(ConstValue::Null))
            }
            Type::INT2 => {
                let v: Option<i16> = row.try_get(idx)?;
                Ok(v.map(|n| ConstValue::Number(n.into()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::INT4 => {
                let v: Option<i32> = row.try_get(idx)?;
                Ok(v.map(|n| ConstValue::Number(n.into()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::INT8 => {
                let v: Option<i64> = row.try_get(idx)?;
                Ok(v.map(|n| ConstValue::Number(n.into()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::FLOAT4 => {
                let v: Option<f32> = row.try_get(idx)?;
                Ok(match v {
                    Some(n) => match serde_json::Number::from_f64(n as f64) {
                        Some(num) => ConstValue::Number(num),
                        None => {
                            tracing::warn!(
                                "Column {} contains non-finite float value: {n}",
                                col.name()
                            );
                            ConstValue::Null
                        }
                    },
                    None => ConstValue::Null,
                })
            }
            Type::FLOAT8 => {
                let v: Option<f64> = row.try_get(idx)?;
                Ok(match v {
                    Some(n) => match serde_json::Number::from_f64(n) {
                        Some(num) => ConstValue::Number(num),
                        None => {
                            tracing::warn!(
                                "Column {} contains non-finite float value: {n}",
                                col.name()
                            );
                            ConstValue::Null
                        }
                    },
                    None => ConstValue::Null,
                })
            }
            Type::JSON | Type::JSONB => {
                let v: Option<serde_json::Value> = row.try_get(idx)?;
                Ok(
                    v.map(|j| ConstValue::from_json(j).unwrap_or(ConstValue::Null))
                        .unwrap_or(ConstValue::Null),
                )
            }
            _ => {
                // Fallback: try to get as String.
                let v: Option<String> = row.try_get(idx).unwrap_or(None);
                Ok(v.map(ConstValue::String).unwrap_or(ConstValue::Null))
            }
        }
    }
}
