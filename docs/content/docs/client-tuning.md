+++
title = "Client Tuning"
description = "Optimize HTTP client settings for production."
+++

# Client Tuning

GQLForge uses an HTTP connection pool for upstream requests. The `@upstream` directive provides settings to tune connection behavior for production workloads.

## Connection Pool

Control the size of the connection pool:

```graphql
schema @upstream(pool_size: 64, pool_max_idle_per_host: 16) {
  query: Query
}
```

| Setting                  | Default | Description                                |
| ------------------------ | ------- | ------------------------------------------ |
| `pool_size`              | 32      | Maximum number of connections in the pool  |
| `pool_max_idle_per_host` | 8       | Maximum idle connections per upstream host |

## Timeouts

Set timeout values to prevent slow upstreams from blocking requests:

```graphql
schema @upstream(connect_timeout: 5, timeout: 30) {
  query: Query
}
```

| Setting           | Default | Description                             |
| ----------------- | ------- | --------------------------------------- |
| `connect_timeout` | 10      | Seconds to wait for a TCP connection    |
| `timeout`         | 60      | Seconds to wait for a complete response |

## Keep-Alive

Configure TCP keep-alive to maintain persistent connections:

```graphql
schema @upstream(keep_alive_interval: 30, keep_alive_timeout: 60, keep_alive_while_idle: true) {
  query: Query
}
```

| Setting                 | Default | Description                                |
| ----------------------- | ------- | ------------------------------------------ |
| `keep_alive_interval`   | 60      | Seconds between keep-alive probes          |
| `keep_alive_timeout`    | 90      | Seconds to wait for a keep-alive response  |
| `keep_alive_while_idle` | false   | Send keep-alive probes on idle connections |

## Production Example

A production-tuned configuration combining all settings:

```graphql
schema
@server(port: 8000)
@upstream(
  pool_size: 128
  pool_max_idle_per_host: 32
  connect_timeout: 5
  timeout: 30
  keep_alive_interval: 30
  keep_alive_timeout: 60
  keep_alive_while_idle: true
  http2_only: true
  http_cache: true
) {
  query: Query
}
```

## Guidelines

- Increase `pool_size` when you have many concurrent upstream calls.
- Lower `connect_timeout` to fail fast when an upstream host is unreachable.
- Enable `keep_alive_while_idle` for long-lived connections to frequently called services.
- Enable `http2_only` when your upstreams support HTTP/2 to benefit from multiplexing.
