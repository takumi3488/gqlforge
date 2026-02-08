+++
title = "REST Endpoints"
description = "Expose REST endpoints from your GraphQL schema."
+++

# REST Endpoints

GQLForge can expose traditional REST API endpoints alongside your GraphQL API using the `@rest` directive.

## The @rest Directive

Apply `@rest` to a Query or Mutation field to make it available as a REST endpoint:

```graphql
type Query {
  user(id: Int!): User
  @rest(path: "/api/users/:id", method: GET)
  @http(url: "https://api.example.com/users/{{.args.id}}")
}
```

A GET request to `/api/users/42` now returns the same data as querying the `user` field in GraphQL.

### Directive Arguments

| Argument | Description                                             |
| -------- | ------------------------------------------------------- |
| `path`   | The URL path pattern. Use `:param` for path parameters. |
| `method` | HTTP method: `GET`, `POST`, `PUT`, `DELETE`             |

## Path Parameters

Path parameters defined with `:param` syntax are mapped to the corresponding GraphQL field arguments:

```graphql
type Query {
  post(userId: Int!, postId: Int!): Post
  @rest(path: "/api/users/:userId/posts/:postId", method: GET)
  @http(url: "https://api.example.com/users/{{.args.userId}}/posts/{{.args.postId}}")
}
```

## Mutations as REST Endpoints

You can expose mutations as POST, PUT, or DELETE endpoints:

```graphql
type Mutation {
  createUser(input: CreateUserInput!): User
  @rest(path: "/api/users", method: POST)
  @http(
    url: "https://api.example.com/users"
    method: POST
    body: "{{.args.input}}"
  )
}
```

The request body is automatically parsed and passed to the field arguments.

## Full Example

```graphql
schema @server(port: 8000) @upstream(base_url: "https://api.example.com") {
  query: Query
  mutation: Mutation
}

type Query {
  users: [User] @rest(path: "/api/users", method: GET) @http(url: "/users")
  user(id: Int!): User @rest(path: "/api/users/:id", method: GET) @http(url: "/users/{{.args.id}}")
}

type Mutation {
  deleteUser(id: Int!): Boolean
  @rest(path: "/api/users/:id", method: DELETE)
  @http(url: "/users/{{.args.id}}", method: DELETE)
}
```

This gives you both a GraphQL endpoint and a set of REST endpoints from the same schema definition.
