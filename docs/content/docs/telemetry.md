+++
title = "Telemetry"
description = "Monitor your GQLForge server with OpenTelemetry."
+++

# Telemetry

GQLForge integrates with OpenTelemetry to provide traces, metrics, and logs for monitoring your GraphQL API.

## Configuration

Add the `@telemetry` directive to your schema:

```graphql
schema
@telemetry(
  export: { otlp: { url: "http://localhost:4317" } }
) {
  query: Query
}
```

## Exporters

GQLForge supports several telemetry exporters.

### OTLP (OpenTelemetry Protocol)

Send telemetry data to any OTLP-compatible collector:

```graphql
@telemetry(
  export: {
    otlp: {
      url: "http://collector:4317"
      headers: [{ key: "Authorization", value: "Bearer {{.env.OTEL_TOKEN}}" }]
    }
  }
)
```

### Prometheus

Expose a `/metrics` endpoint for Prometheus scraping:

```graphql
@telemetry(export: { prometheus: { path: "/metrics" } })
```

### Stdout

Print telemetry data to standard output, useful during development:

```graphql
@telemetry(export: { stdout: { pretty: true } })
```

## Capturing Request Headers

Use `request_headers` to include specific HTTP headers as span attributes:

```graphql
@telemetry(
  export: { otlp: { url: "http://collector:4317" } }
  request_headers: ["x-request-id", "x-correlation-id"]
)
```

This attaches the values of those headers to each trace span, making it easier to correlate requests across services.

## What Gets Tracked

GQLForge emits telemetry data for:

- Incoming GraphQL requests (duration, operation name, status)
- Upstream HTTP and gRPC calls (latency, status codes, URLs)
- Field-level resolution timing
- Error counts and details

## Full Example

```graphql
schema
@server(port: 8000)
@telemetry(
  export: {
    otlp: { url: "http://localhost:4317" }
    prometheus: { path: "/metrics" }
  }
  request_headers: ["x-request-id"]
) {
  query: Query
}
```

This configuration sends trace data via OTLP and exposes a Prometheus metrics endpoint simultaneously.
