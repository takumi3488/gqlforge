+++
title = "JavaScript Extensions"
description = "Extend GQLForge with custom JavaScript functions."
+++

# JavaScript Extensions

GQLForge allows you to write custom resolver logic in JavaScript using the `@js` directive.

## Linking a Script File

First, register your JavaScript file with the schema:

```graphql
schema @link(type: Script, src: "./transforms.js") {
  query: Query
}
```

## The @js Directive

Apply `@js` to a field and specify the function name to call:

```graphql
type Query {
  formattedDate(timestamp: Int!): String @js(name: "formatDate")
}
```

## Writing JavaScript Functions

In your linked script file, export functions that accept a context object and return the resolved value:

```javascript
// transforms.js

function formatDate({ args }) {
  const date = new Date(args.timestamp * 1000);
  return date.toISOString().split("T")[0];
}

function fullName({ value }) {
  return `${value.firstName} ${value.lastName}`;
}
```

### Context Object

Each function receives a context object with these properties:

| Property  | Description                             |
| --------- | --------------------------------------- |
| `args`    | The GraphQL field arguments             |
| `value`   | The parent object's resolved value      |
| `headers` | Request headers from the incoming query |

## Chaining with Other Directives

`@js` can be combined with `@http` to transform upstream responses:

```graphql
type User {
  displayName: String @js(name: "fullName")
}

type Query {
  user(id: Int!): User @http(url: "https://api.example.com/users/{{.args.id}}")
}
```

Here, GQLForge fetches user data via HTTP, then runs `fullName` on the result to compute `displayName`.

## Limitations

- The JavaScript runtime is a lightweight embedded engine, not a full Node.js environment.
- Async operations (network calls, file I/O) are not supported inside JS functions.
- Keep functions small and focused on data transformation.
