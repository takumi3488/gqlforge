mod error;
mod formatter;
mod parser;
pub use error::{Error, Result};
pub use parser::Parser;

/// # Errors
///
/// Returns an error if formatting fails.
pub fn format<T: AsRef<str>>(source: T, parser: &Parser) -> Result<String> {
    formatter::format(source.as_ref(), parser)
}
