+++
title = "Apollo Federation"
description = "Use GQLForge as an Apollo Federation subgraph."
+++

# Apollo Federation

GQLForge can act as a subgraph in an Apollo Federation architecture, allowing you to compose multiple GraphQL services into a unified supergraph.

## Enabling Federation

Set `enable_federation` to `true` in the `@server` directive:

```graphql
schema
  @server(port: 8000, enable_federation: true) {
  query: Query
}
```

This enables the Federation-specific introspection endpoints (`_service` and `_entities`) that the Apollo Gateway or Router expects.

## Defining Entities

Entities are types that can be referenced and extended across subgraphs. Mark them with the `@key` directive:

```graphql
type User @key(fields: "id") {
  id: Int!
  name: String!
  email: Email
}
```

The `fields` argument specifies which field uniquely identifies the entity. The Apollo Router uses this to resolve references across subgraphs.

## Entity Resolvers

When the Apollo Router needs to resolve a `User` by its key, GQLForge calls the resolver defined on the entity field:

```graphql
type User @key(fields: "id") {
  id: Int!
  name: String!
  email: Email
    @http(url: "https://api.example.com/users/{{.value.id}}")
}
```

GQLForge uses the `id` received from the router to fetch the full user data from the upstream service.

## Full Example

```graphql
schema
  @server(port: 8001, enable_federation: true)
  @upstream(base_url: "https://api.example.com") {
  query: Query
}

type Query {
  users: [User] @http(url: "/users")
}

type User @key(fields: "id") {
  id: Int!
  name: String!
  email: Email
}

type Post @key(fields: "id") {
  id: Int!
  title: String!
  author: User @http(url: "/users/{{.value.authorId}}")
}
```

## Composing the Supergraph

Once GQLForge is running as a subgraph, register it with your Apollo Router or Gateway using your standard composition workflow. The router will discover the federated schema and route queries to GQLForge for the types it owns.
