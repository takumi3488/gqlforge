---
title: "@js Directive"
description: "Resolve fields using custom JavaScript functions."
sidebar_label: "@js"
---

# @js Directive

The `@js` directive resolves a field by executing a JavaScript function from a linked script file. This provides full programmatic control over the response.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | The name of the JavaScript function to invoke. |

## Prerequisites

You must link a JavaScript file using the `@link` directive with `type_of: Script`.

## Function Signature

The function receives a context object with:

- `args` -- the field arguments
- `value` -- the parent object
- `env` -- environment variables

It should return the resolved value.

## Example

### Schema

```graphql
schema
  @link(src: "./resolvers.js", type_of: Script)
  @server(port: 8000) {
  query: Query
}

type Query {
  greeting(name: String!): String @js(name: "greet")
  users: [User]
    @http(url: "https://jsonplaceholder.typicode.com/users")
}

type User {
  id: Int!
  name: String!
  email: String!
  initials: String @js(name: "getInitials")
}
```

### resolvers.js

```javascript
function greet({args}) {
  return `Hello, ${args.name}!`;
}

function getInitials({value}) {
  return value.name
    .split(" ")
    .map((part) => part[0])
    .join("");
}
```
