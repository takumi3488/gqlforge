+++
title = "@upstream Directive"
description = "Configure upstream HTTP client behavior in GQLForge."
+++

# @upstream Directive

The `@upstream` directive controls how GQLForge connects to backend services. It governs connection pooling, timeouts, TLS, and request batching.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `on_request` | String | `null` | Name of a JS function invoked before every upstream request. |
| `allowed_headers` | [String] | `[]` | Client request headers forwarded to upstream services. |
| `batch` | Batch | `null` | Configuration for HTTP request batching. See below. |
| `connect_timeout` | Int | `60` | Maximum time in **seconds** to establish a TCP connection. |
| `http_cache` | Int | `null` | Maximum number of entries in the HTTP response cache. |
| `http2_only` | Boolean | `false` | Force HTTP/2 for all upstream connections. |
| `keep_alive_interval` | Int | `60` | Interval in **seconds** between TCP keep-alive probes. |
| `keep_alive_timeout` | Int | `60` | Time in **seconds** to wait for a keep-alive response. |
| `keep_alive_while_idle` | Boolean | `false` | Send keep-alive probes even when the connection is idle. |
| `pool_max_idle_per_host` | Int | `60` | Maximum idle connections retained per upstream host. |
| `pool_idle_timeout` | Int | `60` | Time in **seconds** before an idle connection is closed. |
| `proxy` | Proxy | `null` | HTTP proxy configuration. See below. |
| `tcp_keep_alive` | Int | `5` | Interval in **seconds** for OS-level TCP keep-alive. |
| `timeout` | Int | `60` | Total request timeout in **seconds**. |
| `user_agent` | String | `"GQLForge"` | Value of the `User-Agent` header sent to upstreams. |
| `verify_ssl` | Boolean | `true` | Verify TLS certificates for upstream connections. |

## Batch Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `delay` | Int | `0` | Time in milliseconds to wait before dispatching a batch. |
| `headers` | [String] | `[]` | Headers included when grouping requests into batches. |
| `max_size` | Int | `100` | Maximum number of requests in a single batch. |

## Proxy Configuration

| Field | Type | Description |
|-------|------|-------------|
| `url` | String | The proxy server URL (e.g. `http://proxy:8080`). |

## Example

```graphql
schema
  @upstream(
    connect_timeout: 10
    timeout: 30
    http_cache: 1000
    http2_only: false
    verify_ssl: true
    batch: {delay: 10, max_size: 50}
    allowed_headers: ["Authorization", "X-Request-Id"]
  ) {
  query: Query
}
```
