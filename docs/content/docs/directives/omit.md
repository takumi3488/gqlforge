+++
title = "@omit Directive"
description = "Exclude fields from the public-facing GraphQL schema."
+++

# @omit Directive

The `@omit` directive removes a field from the public GraphQL schema entirely. Unlike `@modify(omit: true)`, a field marked with `@omit` is fully excluded from introspection and client queries.

## Fields

This directive has no fields. Apply it directly to a field definition.

## Use Cases

- Hide internal implementation fields that should never be queried by clients.
- Remove upstream fields that are irrelevant to your API consumers.
- Strip sensitive data fields from the public schema.

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  users: [User] @http(url: "https://jsonplaceholder.typicode.com/users")
}

type User {
  id: Int!
  name: String!
  email: String!
  phone: String!
  website: String @omit
  address: Address @omit
}

type Address {
  street: String
  city: String
}
```

In this example, `website` and `address` are fetched from the upstream API but are not exposed to GraphQL clients.
