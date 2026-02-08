+++
title = "Context Object"
description = "Understanding the request context available in GQLForge templates."
+++

# Context Object

## Overview

GQLForge uses Mustache-style templates to inject dynamic values into resolver configurations. These templates are enclosed in double curly braces (`{{...}}`) and provide access to the request context at resolution time.

## Context Variables

### `.value`

References the value of the parent object. This is used to access fields from the enclosing type when resolving nested fields.

```graphql
type Post {
  id: Int!
  userId: Int!
  user: User @http(path: "/users/{{.value.userId}}")
}
```

In this example, `{{.value.userId}}` resolves to the `userId` field of the current `Post` object.

### `.args`

Provides access to the arguments passed to the current field.

```graphql
type Query {
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
  posts(limit: Int): [Post] @http(path: "/posts?_limit={{.args.limit}}")
}
```

Each argument declared in the field signature is available under `.args`.

### `.headers`

Accesses HTTP headers from the incoming GraphQL request. Header names are case-insensitive.

```graphql
type Query {
  me: User
    @http(
      path: "/users/me"
      headers: [{ key: "Authorization", value: "{{.headers.authorization}}" }]
    )
}
```

This forwards the client's `Authorization` header to the upstream service.

### `.vars`

Reads environment variables defined in the server's runtime environment.

```graphql
type Query {
  config: Config
    @http(
      path: "/config"
      headers: [{ key: "X-Api-Key", value: "{{.vars.API_KEY}}" }]
    )
}
```

See [Environment Variables](@/docs/environment-variables.md) for more on using environment variables in your configuration.

## Template Usage in Directives

Context templates can be used in several places within resolver directives:

- **URL paths**: `@http(path: "/users/{{.args.id}}")`
- **Query parameters**: `@http(path: "/posts?author={{.value.authorId}}")`
- **Request headers**: `headers: [{ key: "X-Token", value: "{{.headers.x-token}}" }]`
- **Request body fields**: Within body templates for POST/PUT requests

## Nested Access

You can traverse nested structures using dot notation:

```graphql
type Query {
  search(filter: SearchInput!): [Result]
    @http(path: "/search?q={{.args.filter.query}}&page={{.args.filter.page}}")
}
```

## Notes

- If a referenced value is `null` or missing, the template renders as an empty string.
- Template expressions are evaluated at query execution time, not at server startup.
- Context variables are read-only and cannot be modified within templates.
