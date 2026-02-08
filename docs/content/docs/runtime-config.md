+++
title = "Runtime Configuration"
description = "Overview of GQLForge runtime configuration directives."
+++

# Runtime Configuration

## Overview

GQLForge's runtime behavior is controlled through directives applied to the `schema` definition in your GraphQL configuration. These directives govern how the server listens for connections, how it communicates with upstream services, and how it integrates with external systems.

## Core Directives

### `@server`

Controls the GraphQL server's runtime settings.

```graphql
schema @server(port: 8000, hostname: "0.0.0.0") {
  query: Query
}
```

Common options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `port` | Int | 8000 | Port the server listens on |
| `hostname` | String | `"127.0.0.1"` | Network interface to bind to |
| `workers` | Int | (auto) | Number of worker threads |
| `globalResponseTimeout` | Int | — | Maximum response time in milliseconds |

### `@upstream`

Configures defaults for all outbound HTTP connections to upstream services.

```graphql
schema
  @server(port: 8000)
  @upstream(
    baseURL: "https://jsonplaceholder.typicode.com"
    httpCache: 42
  ) {
  query: Query
}
```

Common options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `baseURL` | String | — | Base URL prepended to all relative resolver paths |
| `httpCache` | Int | — | Cache TTL in seconds for upstream responses |
| `connectTimeout` | Int | — | Connection timeout in milliseconds |
| `timeout` | Int | — | Request timeout in milliseconds |
| `proxy` | String | — | HTTP proxy URL for outbound requests |

### `@link`

Imports external files into the configuration. Supports multiple file types.

```graphql
schema
  @server(port: 8000)
  @link(type: Config, src: "./users.graphql")
  @link(type: Protobuf, src: "./service.proto") {
  query: Query
}
```

| Option | Type | Description |
|--------|------|-------------|
| `type` | LinkType | Type of the linked resource (`Config`, `Protobuf`, `Script`, etc.) |
| `src` | String | Path to the external file |
| `id` | String | Optional identifier for referencing in other directives |

## Telemetry

GQLForge supports telemetry export for observability. Configure tracing and metrics through the `@telemetry` directive:

```graphql
schema
  @server(port: 8000)
  @telemetry(export: { otlp: { url: "http://localhost:4317" } }) {
  query: Query
}
```

This sends traces and metrics to an OpenTelemetry collector. The telemetry data includes resolver execution times, upstream request durations, and error rates.

## Combining Directives

Multiple directives are applied together on the `schema` definition:

```graphql
schema
  @server(port: 8000, hostname: "0.0.0.0")
  @upstream(baseURL: "https://api.example.com", httpCache: 60)
  @link(type: Config, src: "./types.graphql")
  @telemetry(export: { otlp: { url: "http://localhost:4317" } }) {
  query: Query
  mutation: Mutation
}
```

## Further Reading

- [CLI Reference](@/docs/cli.md) for starting the server with specific options
- [Configuration Conventions](@/docs/conventions.md) for file structure guidelines
- [Environment Variables](@/docs/environment-variables.md) for injecting secrets into your configuration
