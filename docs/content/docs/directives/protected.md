+++
title = "@protected Directive"
description = "Restrict field access to authenticated users with optional expression-based access control."
+++

# @protected Directive

The `@protected` directive restricts access to a field so that only authenticated requests can resolve it. Unauthenticated requests receive an authorization error. Optionally, an `expr` parameter allows fine-grained access control by evaluating expressions against JWT claims and query arguments.

## Fields

| Field  | Type   | Default | Description                                                                                           |
| ------ | ------ | ------- | ----------------------------------------------------------------------------------------------------- |
| `id`   | String | `null`  | The authentication provider ID to validate against. References a JWKS or Htpasswd linked via `@link`. |
| `expr` | String | `null`  | An access control expression evaluated against the request context.                                   |

## Prerequisites

You must configure at least one authentication provider using the `@link` directive with `type_of: Jwks` or `type_of: Htpasswd`.

## Example

```graphql
schema @link(id: "auth", src: "https://example.com/.well-known/jwks.json", type_of: Jwks) @server(port: 8000) {
  query: Query
}

type Query {
  publicPosts: [Post] @http(url: "https://jsonplaceholder.typicode.com/posts")

  myProfile: User @http(url: "https://jsonplaceholder.typicode.com/users/1") @protected(id: "auth")
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

## Expression-Based Access Control

The `expr` parameter enables fine-grained authorization by evaluating a boolean expression against the request context.

### Syntax

| Element         | Syntax               | Example                                                                       |
| --------------- | -------------------- | ----------------------------------------------------------------------------- |
| Path            | `claims.*`, `args.*` | `claims.role`, `args.userId`                                                  |
| String literal  | Single quotes        | `'admin'`                                                                     |
| Number literal  | Integer              | `42`                                                                          |
| Boolean literal | `true` / `false`     | `true`                                                                        |
| Equality        | `==`                 | `claims.role == 'admin'`                                                      |
| Inequality      | `!=`                 | `claims.role != 'guest'`                                                      |
| Logical AND     | `&&`                 | `claims.active == true && claims.role == 'admin'`                             |
| Logical OR      | `\|\|`               | `claims.role == 'admin' \|\| claims.role == 'moderator'`                      |
| Logical NOT     | `!`                  | `!(claims.role == 'guest')`                                                   |
| Grouping        | `()`                 | `(claims.role == 'admin' \|\| claims.role == 'mod') && claims.active == true` |

Operator precedence (highest to lowest): `!`, `&&`, `||`.

### Role-Based Access Control

```graphql
type Query {
  adminDashboard: Dashboard
  @http(url: "https://api.example.com/admin/dashboard")
  @protected(expr: "claims.role == 'admin'")
}
```

### Row-Level Security

```graphql
type Query {
  user(userId: ID!): User
  @http(url: "https://api.example.com/users/{{.args.userId}}")
  @protected(expr: "claims.sub == args.userId")
}
```

### Compound Conditions

```graphql
type Mutation {
  deletePost(postId: ID!): Boolean
  @http(url: "https://api.example.com/posts/{{.args.postId}}", method: "DELETE")
  @protected(expr: "claims.role == 'admin' || claims.role == 'moderator'")
}
```

### Notes

- Authentication is verified **before** the expression is evaluated. If the token is invalid, the request is rejected regardless of the expression.
- If a path references a claim or argument that does not exist, it resolves to `null`. Comparing `null` with any value via `==` returns `false`.
- Comparisons are **type-strict**: `true != 'true'` and `42 != '42'`. Ensure literals match the actual JWT claim types.
