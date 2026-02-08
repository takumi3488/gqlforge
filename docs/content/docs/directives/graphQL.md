+++
title = "@graphQL Directive"
description = "Proxy field resolution to another GraphQL server."
+++

# @graphQL Directive

The `@graphQL` directive resolves a field by forwarding the query to a remote GraphQL endpoint. This enables schema stitching and federation-like composition.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | String | Required | The remote GraphQL endpoint URL. |
| `name` | String | `null` | The remote field name if it differs from the local field name. |
| `args` | [Arg] | `[]` | Arguments to pass to the remote query. |
| `headers` | [Header] | `[]` | HTTP headers sent to the remote server. |
| `batch` | Boolean | `false` | Enable request batching for this remote endpoint. |
| `dedupe` | Boolean | `false` | Deduplicate identical in-flight requests to the remote. |

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  countries: [Country]
    @graphQL(
      url: "https://countries.trevorblades.com/graphql"
      name: "countries"
    )
}

type Country {
  code: String!
  name: String!
  capital: String
}
```

When a client queries `countries`, GQLForge sends the corresponding GraphQL query to the remote server and returns the result.
