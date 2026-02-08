---
title: "@rest Directive"
description: "Expose GraphQL queries as REST API endpoints."
sidebar_label: "@rest"
---

# @rest Directive

The `@rest` directive exposes a GraphQL query or mutation as a traditional REST endpoint. This allows non-GraphQL clients to consume your API over standard HTTP methods and URL paths.

## How It Works

When you annotate a field with `@rest`, GQLForge registers an HTTP route that maps to the underlying GraphQL operation. Path parameters are extracted from the URL and passed as field arguments.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `path` | String | The URL path pattern (e.g. `/users/:id`). Path segments prefixed with `:` become arguments. |
| `method` | Method | HTTP method: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`. Defaults to `GET` for queries. |

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  users: [User]
    @http(url: "https://jsonplaceholder.typicode.com/users")
    @rest(path: "/api/users", method: GET)

  user(id: Int!): User
    @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")
    @rest(path: "/api/users/:id", method: GET)

  userPosts(userId: Int!): [Post]
    @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.userId}}/posts")
    @rest(path: "/api/users/:userId/posts", method: GET)
}

type User {
  id: Int!
  name: String!
  email: String!
}

type Post {
  id: Int!
  title: String!
  body: String!
}
```

With this configuration:

- `GET /api/users` returns all users.
- `GET /api/users/1` returns the user with id 1.
- `GET /api/users/1/posts` returns posts for user 1.

Each REST endpoint executes the corresponding GraphQL resolver internally.
