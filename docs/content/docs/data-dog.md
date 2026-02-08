+++
title = "Datadog"
description = "Export GQLForge telemetry to Datadog."
+++

# Datadog Integration

GQLForge can send traces and metrics to Datadog through the OTLP exporter via the Datadog Agent.

## Prerequisites

- Datadog Agent v6.32+ or v7.32+ running with OTLP ingestion enabled.
- The agent should be configured to accept OTLP over gRPC (default port 4317).

## Datadog Agent Configuration

In your Datadog Agent `datadog.yaml`, enable OTLP ingestion:

```yaml
otlp_config:
  receiver:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
```

## GQLForge Configuration

Point the OTLP exporter to your Datadog Agent:

```graphql
schema
  @server(port: 8000)
  @telemetry(
    export: {
      otlp: {
        url: "http://localhost:4317"
      }
    }
    request_headers: ["x-request-id"]
  ) {
  query: Query
}
```

## What Gets Reported

Once configured, Datadog receives:

- Distributed traces for each GraphQL operation
- Upstream HTTP/gRPC call spans with latency and status
- Error details and counts

You can view this data in the Datadog APM section under your service name.

## Environment-Based Configuration

For containerized deployments, point to the agent's hostname:

```graphql
@telemetry(
  export: {
    otlp: {
      url: "http://{{.env.DD_AGENT_HOST}}:4317"
    }
  }
)
```

Set `DD_AGENT_HOST` to the address of your Datadog Agent container or sidecar.
