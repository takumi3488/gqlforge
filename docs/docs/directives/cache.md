---
title: "@cache Directive"
description: "Cache resolved field values for a specified duration."
sidebar_label: "@cache"
---

# @cache Directive

The `@cache` directive caches a field's resolved value in memory for a specified duration. Subsequent requests for the same field with the same arguments return the cached result, reducing upstream calls.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `max_age` | Int | Duration in **milliseconds** to keep the cached value before it expires. |

## Behavior

- The cache key is derived from the field name, arguments, and parent value.
- Expired entries are evicted lazily on the next access.
- Each server instance maintains its own in-memory cache.

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  users: [User]
    @http(url: "https://jsonplaceholder.typicode.com/users")
    @cache(max_age: 60000)

  user(id: Int!): User
    @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")
    @cache(max_age: 30000)
}

type User {
  id: Int!
  name: String!
  email: String!
}
```

The `users` field is cached for 60 seconds and the `user` field for 30 seconds. During that window, repeated queries are served from memory without contacting the upstream API.
