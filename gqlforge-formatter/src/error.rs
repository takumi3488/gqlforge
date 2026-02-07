use std::sync::Arc;

use thiserror::Error;

#[derive(Error)]
pub enum Error {
    #[error("Formatting failed: {0}")]
    FormattingFailed(String),

    #[error("No file extension found")]
    FileExtensionNotFound,

    #[error("Unsupported file type")]
    UnsupportedFiletype,

    #[error("{}\n\nCaused by:\n    {}", context, source)]
    Context {
        #[source]
        source: Arc<Error>,
        context: String,
    },
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl Error {
    pub fn with_context(self, context: String) -> Self {
        Error::Context { source: Arc::new(self), context }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
