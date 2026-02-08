+++
title = "Telemetry Development"
description = "Working on telemetry features in GQLForge."
+++

# Telemetry Development

This guide covers developing and testing telemetry features within the GQLForge codebase.

## Architecture

GQLForge's telemetry system is built on the OpenTelemetry SDK for Rust. The main components are:

- **Trace provider**: Creates spans for incoming requests and outgoing upstream calls.
- **Metrics provider**: Records counters and histograms for request throughput and latency.
- **Exporters**: Send collected data to configured backends (OTLP, Prometheus, stdout).

## Local Development Setup

Use the stdout exporter during development to see telemetry output in your terminal:

```graphql
schema
  @telemetry(export: { stdout: { pretty: true } }) {
  query: Query
}
```

This prints formatted trace and metric data to standard output without needing an external collector.

## Testing with a Local Collector

For testing OTLP export, run the OpenTelemetry Collector locally:

```bash
docker run -p 4317:4317 otel/opentelemetry-collector:latest
```

Then point GQLForge at it:

```graphql
@telemetry(export: { otlp: { url: "http://localhost:4317" } })
```

## Running Telemetry Tests

Telemetry-related tests can be run with:

```bash
cargo test telemetry
```

These tests verify that spans are created correctly, attributes are attached, and exporters receive expected data.

## Adding a New Metric

1. Define the metric instrument (counter, histogram, etc.) in the telemetry module.
2. Record values at the appropriate points in the request lifecycle.
3. Write tests to verify the metric is emitted with correct labels.
4. Update the telemetry documentation if the metric is user-facing.
