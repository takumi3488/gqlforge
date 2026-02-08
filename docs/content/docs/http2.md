+++
title = "HTTP/2 Support"
description = "Enable HTTP/2 for upstream connections."
+++

# HTTP/2 Support

GQLForge can use HTTP/2 for connections to upstream services, enabling multiplexed requests over a single TCP connection.

## Enabling HTTP/2

Set `http2_only` to `true` in the `@upstream` directive:

```graphql
schema
  @upstream(http2_only: true) {
  query: Query
}
```

When enabled, all outgoing connections to upstream APIs use HTTP/2 exclusively.

## When to Use HTTP/2

HTTP/2 is beneficial when:

- Your upstream services support HTTP/2.
- You are making many concurrent requests to the same host.
- You want to reduce connection overhead through multiplexing.

## When to Avoid HTTP/2

Keep `http2_only` disabled (the default) when:

- Your upstream services only support HTTP/1.1.
- You are connecting through proxies that do not support HTTP/2.

## Example

```graphql
schema
  @server(port: 8000)
  @upstream(http2_only: true, base_url: "https://api.example.com") {
  query: Query
}

type Query {
  users: [User] @http(url: "/users")
  posts: [Post] @http(url: "/posts")
}
```

With HTTP/2 enabled, both upstream calls can be multiplexed over one connection, reducing latency when resolving queries that fan out to multiple endpoints on the same host.
