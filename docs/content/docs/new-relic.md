+++
title = "New Relic"
description = "Export GQLForge telemetry to New Relic."
+++

# New Relic Integration

GQLForge can export telemetry data directly to New Relic using their OTLP ingestion endpoint.

## Configuration

New Relic accepts OTLP data over HTTP. Configure the exporter with your license key:

```graphql
schema
  @server(port: 8000)
  @telemetry(
    export: {
      otlp: {
        url: "https://otlp.nr-data.net:4317"
        headers: [
          { key: "api-key", value: "{{.env.NEW_RELIC_LICENSE_KEY}}" }
        ]
      }
    }
  ) {
  query: Query
}
```

## Environment Variables

Set your New Relic license key before starting the server:

```bash
export NEW_RELIC_LICENSE_KEY="your-license-key-here"
```

## Viewing Data in New Relic

After configuration, telemetry data appears in:

- **Distributed Tracing**: View end-to-end traces for GraphQL operations.
- **APM**: Monitor service health, throughput, and error rates.
- **Metrics**: Query custom metrics via NRQL.

## EU Region

If your New Relic account is in the EU datacenter, use the EU endpoint:

```graphql
@telemetry(
  export: {
    otlp: {
      url: "https://otlp.eu01.nr-data.net:4317"
      headers: [
        { key: "api-key", value: "{{.env.NEW_RELIC_LICENSE_KEY}}" }
      ]
    }
  }
)
```

Refer to the [Telemetry](@/docs/telemetry.md) page for additional exporter options.
