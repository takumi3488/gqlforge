use std::env::VarError;

use derive_more::From;
use http::header::InvalidHeaderValue;
use opentelemetry_sdk::trace::TraceError;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Tonic Transport Error: {}", _0)]
    TonicTransport(tonic::transport::Error),

    #[error("Set Global Default Error: {}", _0)]
    SetGlobalDefault(SetGlobalDefaultError),

    #[error("Trace Error: {}", _0)]
    Trace(TraceError),

    #[error("Opentelemetry Error: {}", _0)]
    Opentelemetry(opentelemetry_sdk::error::OTelSdkError),

    #[error("Exporter Build Error: {}", _0)]
    ExporterBuild(opentelemetry_otlp::ExporterBuildError),

    #[error("Var Error: {}", _0)]
    Var(VarError),

    #[error("Invalid header value: {}", _0)]
    InvalidHeaderValue(InvalidHeaderValue),
}
