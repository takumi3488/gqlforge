---
title: "Telemetry Configuration"
description: "Configure OpenTelemetry exporters for GQLForge."
sidebar_label: "Telemetry"
---

# Telemetry Configuration

GQLForge supports OpenTelemetry-based observability. You can export traces, metrics, and logs to multiple backends.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `export` | ExportConfig | Configures where telemetry data is sent. |
| `request_headers` | [String] | HTTP request headers captured and attached as span attributes. |

## Export Targets

| Target | Description |
|--------|-------------|
| `Stdout` | Prints telemetry data to standard output. Useful for local development. |
| `Otlp` | Sends data to an OpenTelemetry Protocol (OTLP) compatible collector. |
| `Prometheus` | Exposes a `/metrics` endpoint for Prometheus scraping. |

## OTLP Configuration

When using `Otlp`, provide an `url` pointing to the collector endpoint.

## Examples

### Export to stdout

```graphql
schema
  @telemetry(export: {format: Stdout})
  @server(port: 8000) {
  query: Query
}
```

### Export to an OTLP collector

```graphql
schema
  @telemetry(
    export: {format: Otlp, url: "http://localhost:4317"}
    request_headers: ["X-Request-Id", "Authorization"]
  )
  @server(port: 8000) {
  query: Query
}
```

### Expose Prometheus metrics

```graphql
schema
  @telemetry(export: {format: Prometheus})
  @server(port: 8000) {
  query: Query
}
```

With this configuration, metrics are available at `http://localhost:8000/metrics` for Prometheus to scrape.
