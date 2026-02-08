+++
title = "@http Directive"
description = "Resolve GraphQL fields by calling REST or HTTP endpoints."
+++

# @http Directive

The `@http` directive resolves a field by making an HTTP request to an external endpoint. It is the primary way to connect REST APIs to your GraphQL schema.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | String | Required | The endpoint URL. Supports mustache templates like `{{.args.id}}`. |
| `method` | Method | `GET` | HTTP method: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`. |
| `body` | String | `null` | Request body template, typically used with POST/PUT. |
| `encoding` | Encoding | `ApplicationJson` | Body encoding: `ApplicationJson` or `ApplicationXWwwFormUrlencoded`. |
| `headers` | [Header] | `[]` | Additional request headers. |
| `query` | [URLParam] | `[]` | URL query parameters appended to the request. |
| `batch_key` | [String] | `[]` | Field path used to group and batch multiple requests. |
| `dedupe` | Boolean | `false` | Deduplicate identical in-flight requests. |
| `select` | String | `null` | JSONPath-like selector to extract a subset of the response. |
| `on_response_body` | String | `null` | Name of a JS function to transform the response body. |
| `on_request` | String | `null` | Name of a JS function to transform the outgoing request. |

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  users: [User] @http(url: "https://jsonplaceholder.typicode.com/users")
  user(id: Int!): User
    @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
  email: String!
  posts: [Post]
    @http(url: "https://jsonplaceholder.typicode.com/users/{{.value.id}}/posts")
}

type Post {
  id: Int!
  title: String!
  body: String!
}
```

The `{{.args.id}}` template substitutes the field argument, and `{{.value.id}}` references the parent object's `id` field.
