mod error;
mod parser;
mod formatter;
pub use error::{Error, Result};
pub use parser::Parser;

pub async fn format<T: AsRef<str>>(source: T, parser: &Parser) -> Result<String> {
    formatter::format(source.as_ref(), parser)
}
