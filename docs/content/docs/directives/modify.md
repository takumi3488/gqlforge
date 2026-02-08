+++
title = "@modify Directive"
description = "Rename or omit fields in the public GraphQL schema."
+++

# @modify Directive

The `@modify` directive renames a field in the public schema or marks it as omitted. This is useful for adapting upstream API field names to your preferred GraphQL conventions.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | String | `null` | The new public name for the field. |
| `omit` | Boolean | `false` | If `true`, the field is hidden from the public schema but still available internally. |

## Examples

### Renaming a field

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
  username: String! @modify(name: "handle")
  email: String!
  phone: String!
}
```

Clients query `handle` instead of `username`, while the upstream API still returns `username`.

### Omitting a field

```graphql
type User {
  id: Int!
  name: String!
  email: String!
  internalCode: String @modify(omit: true)
}
```

The `internalCode` field is available to other resolvers but not visible in the public schema or introspection.
