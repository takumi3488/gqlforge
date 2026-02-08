+++
title = "Honeycomb"
description = "Export GQLForge telemetry to Honeycomb."
+++

# Honeycomb Integration

GQLForge can send traces and metrics to Honeycomb through the OTLP exporter.

## Configuration

Honeycomb natively supports OTLP ingestion. Configure the exporter with your API key and dataset:

```graphql
schema
  @server(port: 8000)
  @telemetry(
    export: {
      otlp: {
        url: "https://api.honeycomb.io:443"
        headers: [
          { key: "x-honeycomb-team", value: "{{.env.HONEYCOMB_API_KEY}}" }
          { key: "x-honeycomb-dataset", value: "gqlforge-production" }
        ]
      }
    }
  ) {
  query: Query
}
```

## Environment Variables

Set your Honeycomb API key before starting the server:

```bash
export HONEYCOMB_API_KEY="your-api-key-here"
```

## Viewing Data in Honeycomb

Once telemetry is flowing, you can:

- **Query traces**: Filter and explore individual GraphQL operation traces.
- **Build dashboards**: Create visualizations for latency percentiles, error rates, and throughput.
- **Set triggers**: Configure alerts based on trace patterns or metric thresholds.

## Honeycomb Environments

If you use Honeycomb Environments, the dataset header may not be required. In that case, traces are routed based on your API key's environment:

```graphql
@telemetry(
  export: {
    otlp: {
      url: "https://api.honeycomb.io:443"
      headers: [
        { key: "x-honeycomb-team", value: "{{.env.HONEYCOMB_API_KEY}}" }
      ]
    }
  }
)
```

Refer to the [Telemetry](@/docs/telemetry.md) page for additional configuration options.
