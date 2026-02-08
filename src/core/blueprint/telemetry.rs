use std::str::FromStr;

use gqlforge_valid::{Valid, Validator};
use http::header::{HeaderMap, HeaderName, HeaderValue};
use url::Url;

use super::{BlueprintError, TryFoldConfig};
use crate::core::config::{self, ConfigModule, KeyValue, PrometheusExporter, StdoutExporter};
use crate::core::directive::DirectiveCodec;
use crate::core::try_fold::TryFold;

#[derive(Debug, Clone)]
pub struct OtlpExporter {
    pub url: Url,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone)]
pub enum TelemetryExporter {
    Stdout(StdoutExporter),
    Otlp(OtlpExporter),
    Prometheus(PrometheusExporter),
}

#[derive(Debug, Default, Clone)]
pub struct Telemetry {
    pub export: Option<TelemetryExporter>,
    pub request_headers: Vec<String>,
}

fn to_url(url: &str) -> Valid<Url, BlueprintError> {
    match Url::parse(url).map_err(BlueprintError::UrlParse) {
        Ok(url) => Valid::succeed(url),
        Err(err) => Valid::fail(err),
    }
    .trace("url")
}

fn to_headers(headers: Vec<KeyValue>) -> Valid<HeaderMap, BlueprintError> {
    Valid::from_iter(headers.iter(), |key_value| {
        match HeaderName::from_str(&key_value.key).map_err(BlueprintError::InvalidHeaderName) {
            Ok(name) => Valid::succeed(name),
            Err(err) => Valid::fail(err),
        }
        .zip({
            match HeaderValue::from_str(&key_value.value)
                .map_err(BlueprintError::InvalidHeaderValue)
            {
                Ok(value) => Valid::succeed(value),
                Err(err) => Valid::fail(err),
            }
        })
    })
    .map(HeaderMap::from_iter)
    .trace("headers")
}

pub fn to_opentelemetry<'a>() -> TryFold<'a, ConfigModule, Telemetry, BlueprintError> {
    TryFoldConfig::<Telemetry>::new(|config, up| {
        if let Some(export) = config.telemetry.export.as_ref() {
            let export: Valid<TelemetryExporter, BlueprintError> = match export {
                config::TelemetryExporter::Stdout(config) => {
                    Valid::succeed(TelemetryExporter::Stdout(config.clone()))
                }
                config::TelemetryExporter::Otlp(config) => to_url(&config.url)
                    .zip(to_headers(config.headers.clone()))
                    .map(|(url, headers)| TelemetryExporter::Otlp(OtlpExporter { url, headers }))
                    .trace("otlp"),
                config::TelemetryExporter::Prometheus(config) => {
                    Valid::succeed(TelemetryExporter::Prometheus(config.clone()))
                }
            };

            export
                .map(|export| Telemetry {
                    export: Some(export),
                    request_headers: config.telemetry.request_headers.clone(),
                })
                .trace(config::Telemetry::trace_name().as_str())
        } else {
            Valid::succeed(up)
        }
    })
}
