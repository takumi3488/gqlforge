---
title: "HTTP Caching"
description: "Enable HTTP response caching in GQLForge."
sidebar_label: "HTTP Cache"
---

# HTTP Caching

GQLForge provides two layers of caching: upstream HTTP response caching and field-level caching.

## Upstream HTTP Cache

Enable caching of upstream HTTP responses using the `@upstream` directive:

```graphql
schema
  @upstream(http_cache: true) {
  query: Query
}
```

When enabled, GQLForge respects standard HTTP cache headers (`Cache-Control`, `ETag`, `Expires`) from upstream responses. Subsequent requests for the same resource are served from the cache until the entry expires or is invalidated.

## Field-Level Caching with @cache

For more granular control, apply the `@cache` directive to individual fields:

```graphql
type Query {
  popularPosts: [Post]
    @http(url: "https://api.example.com/posts/popular")
    @cache(max_age: 300)
}
```

### @cache Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `max_age` | Int | Time in seconds before the cached value expires |

## Combining Both Layers

You can use upstream HTTP caching and field-level caching together:

```graphql
schema
  @server(port: 8000)
  @upstream(http_cache: true, base_url: "https://api.example.com") {
  query: Query
}

type Query {
  settings: Settings @http(url: "/settings") @cache(max_age: 600)
  feed: [Post] @http(url: "/feed")
}
```

In this setup:

- The `settings` field is cached for 10 minutes at the field level.
- All upstream responses benefit from HTTP-level caching when the server sends appropriate cache headers.
- The `feed` field relies only on upstream HTTP cache headers.

## When to Use Each Approach

- **Upstream HTTP cache**: Use when your APIs return proper cache headers and you want transparent caching.
- **@cache directive**: Use when you need explicit control over TTL regardless of what the upstream returns.
