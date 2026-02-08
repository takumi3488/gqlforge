pub use cache::*;
pub use data_loader::*;
pub use data_loader_request::*;
use http::HeaderValue;
pub use method::Method;
pub use query_encoder::QueryEncoder;
pub use request_context::RequestContext;
pub use request_handler::{API_URL_PREFIX, handle_request};
pub use request_template::RequestTemplate;
pub use response::*;

mod cache;
mod data_loader;
mod data_loader_request;
mod method;
mod query_encoder;
mod request_context;
mod request_handler;
mod request_template;
mod response;
pub mod showcase;
mod telemetry;
mod transformations;

pub static GQLFORGE_HTTPS_ORIGIN: HeaderValue =
    HeaderValue::from_static("https://gqlforge.pages.dev");
pub static GQLFORGE_HTTP_ORIGIN: HeaderValue =
    HeaderValue::from_static("http://gqlforge.pages.dev");
