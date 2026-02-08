+++
title = "@call Directive"
description = "Compose multiple resolver steps into a single field resolution."
+++

# @call Directive

The `@call` directive composes one or more resolver steps into a pipeline. Each step invokes another query or mutation field, passing data forward through the chain.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `steps` | [Step] | Required | An ordered list of resolver steps to execute. |
| `dedupe` | Boolean | `false` | Deduplicate identical composed calls. |

### Step Fields

| Field | Type | Description |
|-------|------|-------------|
| `query` | String | Name of a Query field to invoke. |
| `mutation` | String | Name of a Mutation field to invoke (use `query` or `mutation`, not both). |
| `args` | JSON | Arguments passed to the target field. Supports mustache templates. |

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  posts: [Post] @http(url: "https://jsonplaceholder.typicode.com/posts")
  user(id: Int!): User
    @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")

  firstPostAuthor: User
    @call(
      steps: [
        {query: "posts"}
        {query: "user", args: {id: "{{.0.userId}}"}}
      ]
    )
}

type Post {
  id: Int!
  userId: Int!
  title: String!
}

type User {
  id: Int!
  name: String!
  email: String!
}
```

The `firstPostAuthor` field fetches all posts, then uses the first post's `userId` to resolve the corresponding user.
