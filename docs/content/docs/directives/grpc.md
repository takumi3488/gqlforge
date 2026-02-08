+++
title = "@grpc Directive"
description = "Resolve GraphQL fields by calling gRPC service methods."
+++

# @grpc Directive

The `@grpc` directive resolves a field by invoking a gRPC service method. It requires a linked protobuf definition via the `@link` directive.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `url` | String | Required | The gRPC server address (e.g. `https://grpc-server:50051`). |
| `method` | String | Required | Fully qualified gRPC method name (e.g. `news.NewsService.GetNews`). |
| `body` | String | `null` | Template for the gRPC request message body. |
| `headers` | [Header] | `[]` | Additional metadata headers sent with the gRPC call. |
| `batch_key` | [String] | `[]` | Field path used for request batching. |
| `dedupe` | Boolean | `false` | Deduplicate identical in-flight gRPC calls. |
| `select` | String | `null` | Path selector to extract a subset of the response message. |
| `on_response_body` | String | `null` | JS function name to transform the response. |

## Example

```graphql
schema
  @link(id: "news", src: "./news.proto", type_of: Protobuf)
  @server(port: 8000) {
  query: Query
}

type Query {
  news: [NewsEntry]
    @grpc(
      url: "https://grpc-server:50051"
      method: "news.NewsService.ListNews"
    )
  newsById(id: Int!): NewsEntry
    @grpc(
      url: "https://grpc-server:50051"
      method: "news.NewsService.GetNews"
      body: {id: "{{.args.id}}"}
    )
}

type NewsEntry {
  id: Int!
  title: String!
  content: String!
}
```

The `method` field must match the package, service, and RPC name defined in the `.proto` file.
