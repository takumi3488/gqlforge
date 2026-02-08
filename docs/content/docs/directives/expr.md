+++
title = "@expr Directive"
description = "Return static values or computed expressions from a field."
+++

# @expr Directive

The `@expr` directive resolves a field to a static value or a dynamically computed expression using mustache templates. No external service call is made.

## Fields

| Field  | Type | Description                                                                 |
| ------ | ---- | --------------------------------------------------------------------------- |
| `body` | JSON | The value to return. Can be a literal, object, array, or mustache template. |

## Template Variables

Inside `body`, you can reference:

- `{{.value}}` -- the parent object
- `{{.args.name}}` -- field arguments
- `{{.value.fieldName}}` -- a specific field from the parent
- `{{.env.VAR_NAME}}` -- environment variables

## Examples

### Static value

```graphql
type Query {
  version: String @expr(body: "1.0.0")
}
```

### Computed from parent

```graphql
type Query {
  user(id: Int!): User @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
  email: String!
  displayName: String @expr(body: "User #{{.value.id}}: {{.value.name}}")
}
```

### Constructing an object

```graphql
type Query {
  config: AppConfig @expr(body: { name: "GQLForge", debug: false, maxRetries: 3 })
}

type AppConfig {
  name: String!
  debug: Boolean!
  maxRetries: Int!
}
```
