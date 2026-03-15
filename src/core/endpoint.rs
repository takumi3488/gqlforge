use derive_setters::Setters;
use http::header::HeaderMap;

use crate::core::config::Encoding;
use crate::core::http::Method;
use crate::core::json::JsonSchema;

#[derive(Clone, Debug, Setters)]
pub struct Endpoint {
    pub path: String,
    pub query: Vec<(String, String, bool)>,
    pub method: Method,
    pub input: JsonSchema,
    pub output: JsonSchema,
    pub headers: HeaderMap,
    pub body: Option<serde_json::Value>,
    pub description: Option<String>,
    pub encoding: Encoding,
}

impl Endpoint {
    #[must_use] 
    pub fn new(url: String) -> Endpoint {
        Self {
            path: url,
            query: Vec::new(),
            method: Method::default(),
            input: JsonSchema::default(),
            output: JsonSchema::default(),
            headers: HeaderMap::default(),
            body: None,
            description: None,
            encoding: Encoding::default(),
        }
    }
}
