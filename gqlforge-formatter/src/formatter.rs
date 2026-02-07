use pretty_graphql::config::FormatOptions;

use super::{Error, Result};
use crate::Parser;

pub fn format(source: &str, parser: &Parser) -> Result<String> {
    match parser {
        Parser::Gql => {
            let options = FormatOptions::default();
            pretty_graphql::format_text(source, &options)
                .map_err(|e| Error::FormattingFailed(e.to_string()))
        }
        _ => Err(Error::UnsupportedFiletype),
    }
}
