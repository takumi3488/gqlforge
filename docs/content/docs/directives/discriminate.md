+++
title = "@discriminate Directive"
description = "Set the discriminator field for union type resolution."
+++

# @discriminate Directive

The `@discriminate` directive tells GQLForge which field in the response data identifies the concrete type of a union member. This is essential for resolving union and interface types from REST APIs that return a type indicator field.

## Fields

| Field   | Type   | Default  | Description                                                     |
| ------- | ------ | -------- | --------------------------------------------------------------- |
| `field` | String | `"type"` | The name of the JSON field used to determine the concrete type. |

## How It Works

When a union type is resolved from an HTTP response, GQLForge inspects the value of the discriminator field to decide which union member the object belongs to. The value must match the name of a type in the union.

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  feed: [FeedItem] @http(url: "https://api.example.com/feed")
}

union FeedItem @discriminate(field: "kind") = Post | Comment | Image

type Post {
  kind: String!
  id: Int!
  title: String!
  body: String!
}

type Comment {
  kind: String!
  id: Int!
  text: String!
}

type Image {
  kind: String!
  id: Int!
  url: String!
}
```

Each object in the upstream response must include a `kind` field whose value is `"Post"`, `"Comment"`, or `"Image"`.
