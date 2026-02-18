use std::sync::Arc;

use async_graphql_value::ConstValue;
use gqlforge::core::postgres::PostgresIO;

/// A mock implementation of `PostgresIO` that returns a fixed response.
pub struct MockPostgresIO {
    response: ConstValue,
}

impl MockPostgresIO {
    pub fn new(response: ConstValue) -> Arc<Self> {
        Arc::new(Self { response })
    }
}

#[async_trait::async_trait]
impl PostgresIO for MockPostgresIO {
    async fn execute(&self, _query: &str, _params: &[String]) -> anyhow::Result<ConstValue> {
        Ok(self.response.clone())
    }
}
