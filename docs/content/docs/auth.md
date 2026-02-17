+++
title = "Authentication"
description = "Protect your GraphQL API with authentication and authorization."
+++

# Authentication

GQLForge provides built-in authentication and authorization through directives and linked provider files.

## Authentication Providers

Use the `@link` directive to register an authentication provider.

### HTTP Basic (Htpasswd)

```graphql
schema @link(type: Htpasswd, src: "./htpasswd") {
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
schema @link(type: Jwks, src: "https://auth.example.com/.well-known/jwks.json") {
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

### Expression-Based Access Control

The `@protected` directive accepts an `expr` parameter for fine-grained authorization based on JWT claims and query arguments.

```graphql
type Query {
  adminDashboard: Dashboard
  @http(url: "https://api.example.com/admin/dashboard")
  @protected(expr: "claims.role == 'admin'")

  user(userId: ID!): User
  @http(url: "https://api.example.com/users/{{.args.userId}}")
  @protected(expr: "claims.sub == args.userId")
}
```

The expression is evaluated after token validation. If it returns `false`, the request is rejected with an authorization error.

For the full expression syntax and supported operators, see the [`@protected` directive reference](@/docs/directives/protected.md).

## JWT Claims in Templates

When using JWKS authentication, verified JWT claims are available in Mustache templates via `.claims`. This enables dynamic URL construction based on the authenticated user's identity.

```graphql
type Query {
  myPosts: [Post] @http(url: "https://api.example.com/posts?author={{.claims.sub}}") @protected
}
```

In this example, `{{.claims.sub}}` is replaced with the `sub` claim from the JWT, so each user only fetches their own posts.

For more details on available context variables, see the [Context Object](@/docs/context.md) page.

## How It Works

1. GQLForge reads the `Authorization` header from incoming requests.
2. The token or credentials are validated against the configured provider (Htpasswd or JWKS).
3. For JWKS providers, the decoded JWT claims are stored in the request context, making them available via `claims.*` in expressions and `{{.claims.*}}` in templates.
4. If the `@protected` directive specifies an `expr`, the expression is evaluated against the request context. If it returns `false`, the request is rejected with an authorization error.
5. If validation succeeds (and the expression passes, if specified), the request proceeds normally.
6. If validation fails, fields marked with `@protected` return an authentication error.

## Combining Providers

You can link multiple providers in the same schema. GQLForge attempts validation against each provider in order until one succeeds.

```graphql
schema
@link(type: Jwks, src: "https://auth.example.com/.well-known/jwks.json")
@link(type: Htpasswd, src: "./htpasswd") {
  query: Query
}
```
