use serde::{Deserialize, Serialize};

use crate::core::is_default;

/// The `groupBy` parameter groups multiple data requests into a single call. For more details please refer out [n + 1 guide](https://gqlforge.pages.dev/docs/guides/n+1#solving-using-batching).
#[derive(Clone, Debug, Eq, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct GroupBy {
    #[serde(default, skip_serializing_if = "is_default")]
    path: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    key: Option<String>,
}

impl GroupBy {
    #[must_use] 
    pub fn new(path: Vec<String>, key: Option<String>) -> Self {
        Self { path, key }
    }

    #[must_use] 
    pub fn path(&self) -> Vec<String> {
        if self.path.is_empty() {
            return vec![String::from(ID)];
        }
        self.path.clone()
    }

    #[must_use] 
    ///
    /// # Panics
    ///
    /// Panics if an internal assertion fails.
    pub fn key(&self) -> &str {
        if let Some(value) = &self.key { value } else {
            if self.path.is_empty() {
                return ID;
            }
            self.path.last().map_or(ID, String::as_str)
        }
    }
}

const ID: &str = "id";

impl Default for GroupBy {
    fn default() -> Self {
        Self { path: vec![ID.to_string()], key: None }
    }
}
