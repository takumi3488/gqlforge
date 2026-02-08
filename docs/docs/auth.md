---
title: "Authentication"
description: "Protect your GraphQL API with authentication and authorization."
sidebar_label: "Authentication"
---

# Authentication

GQLForge provides built-in authentication and authorization through directives and linked provider files.

## Authentication Providers

Use the `@link` directive to register an authentication provider.

### HTTP Basic (Htpasswd)

```graphql
schema
  @link(type: Htpasswd, src: "./htpasswd") {
  query: Query
}
```

The referenced file uses the standard htpasswd format:

```
admin:$apr1$xyz$hashedpassword
viewer:$apr1$abc$anotherpassword
```

### JSON Web Keys (JWKS)

```graphql
schema
  @link(type: Jwks, src: "https://auth.example.com/.well-known/jwks.json") {
  query: Query
}
```

GQLForge fetches the JWKS endpoint and validates JWT tokens from incoming requests automatically.

## The @protected Directive

Apply `@protected` to restrict access to authenticated users.

### Field-Level Protection

```graphql
type Query {
  publicPosts: [Post] @http(url: "https://api.example.com/posts")
  myProfile: User @http(url: "https://api.example.com/me") @protected
}
```

Only authenticated requests can resolve `myProfile`. Unauthenticated requests receive an error.

### Type-Level Protection

```graphql
type AdminData @protected {
  revenue: Float
  activeUsers: Int
}
```

When `@protected` is placed on a type, every field that returns that type requires authentication.

## How It Works

1. GQLForge reads the `Authorization` header from incoming requests.
2. The token or credentials are validated against the configured provider (Htpasswd or JWKS).
3. If validation succeeds, the request proceeds normally.
4. If validation fails, fields marked with `@protected` return an authentication error.

## Combining Providers

You can link multiple providers in the same schema. GQLForge attempts validation against each provider in order until one succeeds.

```graphql
schema
  @link(type: Jwks, src: "https://auth.example.com/.well-known/jwks.json")
  @link(type: Htpasswd, src: "./htpasswd") {
  query: Query
}
```
