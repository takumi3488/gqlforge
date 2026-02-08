+++
title = "Apollo Studio"
description = "Connect GQLForge to Apollo Studio for monitoring."
+++

# Apollo Studio Integration

You can send GQLForge telemetry data to Apollo Studio for operation-level monitoring and performance tracking.

## Setup

Apollo Studio accepts telemetry via its OTLP-compatible ingestion endpoint. Configure the `@telemetry` directive to export traces:

```graphql
schema
@server(port: 8000)
@telemetry(
  export: {
    otlp: {
      url: "https://usage-reporting.api.apollographql.com"
      headers: [
        { key: "x-api-key", value: "{{.env.APOLLO_KEY}}" }
      ]
    }
  }
) {
  query: Query
}
```

## Environment Variables

Set the following environment variable before starting your server:

```bash
export APOLLO_KEY="service:my-graph:your-api-key-here"
```

## What You Get

Once connected, Apollo Studio provides:

- **Operation traces**: See how individual GraphQL operations perform.
- **Field-level metrics**: Identify slow or frequently used fields.
- **Error tracking**: Monitor error rates across operations.
- **Schema checks**: Validate schema changes against real traffic patterns.

## Federation Compatibility

If you are using GQLForge as a Federation subgraph, Apollo Studio can track subgraph-specific metrics when combined with the Apollo Router.

Refer to the [Telemetry](@/docs/telemetry.md) page for more configuration options.
