use std::fmt;

use super::{Error, Result};

#[derive(Clone)]
pub enum Parser {
    Gql,
    Yml,
    Json,
    Md,
    Ts,
    Js,
}

impl fmt::Display for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parser::Gql => write!(f, "gql"),
            Parser::Yml => write!(f, "yml"),
            Parser::Json => write!(f, "json"),
            Parser::Md => write!(f, "md"),
            Parser::Ts => write!(f, "ts"),
            Parser::Js => write!(f, "js"),
        }
    }
}

impl Parser {
    pub fn detect(path: &str) -> Result<Self> {
        let ext = path
            .split('.')
            .next_back()
            .ok_or(Error::FileExtensionNotFound)?
            .to_lowercase();
        match ext.as_str() {
            "gql" | "graphql" => Ok(Parser::Gql),
            "yml" | "yaml" => Ok(Parser::Yml),
            "json" => Ok(Parser::Json),
            "md" => Ok(Parser::Md),
            "ts" => Ok(Parser::Ts),
            "js" => Ok(Parser::Js),
            _ => Err(Error::UnsupportedFiletype),
        }
    }
}
