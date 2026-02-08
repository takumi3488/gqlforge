---
title: "@protected Directive"
description: "Restrict field access to authenticated users."
sidebar_label: "@protected"
---

# @protected Directive

The `@protected` directive restricts access to a field so that only authenticated requests can resolve it. Unauthenticated requests receive an authorization error.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | String | `null` | The authentication provider ID to validate against. References a JWKS or Htpasswd linked via `@link`. |

## Prerequisites

You must configure at least one authentication provider using the `@link` directive with `type_of: Jwks` or `type_of: Htpasswd`.

## Example

```graphql
schema
  @link(id: "auth", src: "https://example.com/.well-known/jwks.json", type_of: Jwks)
  @server(port: 8000) {
  query: Query
}

type Query {
  publicPosts: [Post]
    @http(url: "https://jsonplaceholder.typicode.com/posts")

  myProfile: User
    @http(url: "https://jsonplaceholder.typicode.com/users/1")
    @protected(id: "auth")
}

type Post {
  id: Int!
  title: String!
}

type User {
  id: Int!
  name: String!
  email: String! @protected(id: "auth")
}
```

In this schema, `publicPosts` is accessible without authentication. The `myProfile` field and the `email` field on `User` require a valid JWT token verified against the linked JWKS endpoint.
