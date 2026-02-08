+++
title = "@server Directive"
description = "Configure your GQLForge GraphQL server settings."
+++

# @server Directive

The `@server` directive configures core behavior of the GQLForge GraphQL server, including networking, validation, and runtime options.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `apollo_tracing` | Boolean | `false` | Enable Apollo Tracing extensions in responses for performance profiling. |
| `batch_requests` | Boolean | `false` | Allow batched GraphQL queries in a single HTTP request. |
| `headers` | Headers | `null` | Global response headers applied to every outgoing HTTP response. |
| `global_response_timeout` | Int | `null` | Maximum time in **milliseconds** before a request is terminated. |
| `hostname` | String | `"0.0.0.0"` | Network interface address the server binds to. |
| `introspection` | Boolean | `true` | Enable the GraphQL introspection system. Disable in production for security. |
| `enable_federation` | Boolean | `false` | Expose Apollo Federation entity service fields (`_entities`, `_service`). |
| `pipeline_flush` | Boolean | `true` | Flush the response pipeline after each chunk for lower latency. |
| `port` | Int | `8000` | TCP port the server listens on. |
| `query_validation` | Boolean | `true` | Validate incoming queries against the schema before execution. |
| `response_validation` | Boolean | `false` | Validate resolver responses against the expected return types. |
| `script` | ScriptOptions | `null` | Configuration for the embedded JavaScript runtime. |
| `showcase` | Boolean | `false` | Enable the built-in GraphQL playground UI at the server root. |

## Example

```graphql
schema
  @server(
    port: 4000
    hostname: "0.0.0.0"
    global_response_timeout: 30000
    introspection: true
    query_validation: true
    response_validation: false
    batch_requests: true
    showcase: true
  ) {
  query: Query
}

type Query {
  hello: String @expr(body: "Hello, world!")
}
```

This configuration starts the server on port 4000, enables batching, and exposes the playground UI.
