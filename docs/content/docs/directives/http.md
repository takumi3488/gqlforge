+++
title = "@http Directive"
description = "Resolve GraphQL fields by calling REST or HTTP endpoints."
+++

# @http Directive

The `@http` directive resolves a field by making an HTTP request to an external endpoint. It is the primary way to connect REST APIs to your GraphQL schema.

## Fields

| Field              | Type       | Default           | Description                                                          |
| ------------------ | ---------- | ----------------- | -------------------------------------------------------------------- |
| `url`              | String     | Required          | The endpoint URL. Supports mustache templates like `{{.args.id}}`.   |
| `method`           | Method     | `GET`             | HTTP method: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`.                |
| `body`             | String     | `null`            | Request body template, typically used with POST/PUT.                 |
| `encoding`         | Encoding   | `ApplicationJson` | Body encoding: `ApplicationJson` or `ApplicationXWwwFormUrlencoded`. |
| `headers`          | [Header]   | `[]`              | Additional request headers.                                          |
| `query`            | [URLParam] | `[]`              | URL query parameters appended to the request.                        |
| `batch_key`        | [String]   | `[]`              | Field path used to group and batch multiple requests.                |
| `dedupe`           | Boolean    | `false`           | Deduplicate identical in-flight requests.                            |
| `select`           | String     | `null`            | JSONPath-like selector to extract a subset of the response.          |
| `on_response_body` | String     | `null`            | Name of a JS function to transform the response body.                |
| `on_request`       | String     | `null`            | Name of a JS function to transform the outgoing request.             |

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  users: [User] @http(url: "https://jsonplaceholder.typicode.com/users")
  user(id: Int!): User @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
  email: String!
  posts: [Post] @http(url: "https://jsonplaceholder.typicode.com/users/{{.value.id}}/posts")
}

type Post {
  id: Int!
  title: String!
  body: String!
}
```

The `{{.args.id}}` template substitutes the field argument, and `{{.value.id}}` references the parent object's `id` field.

## Subscriptions (SSE Streaming)

When `@http` is placed on a field under the `Subscription` root type, GQLForge automatically connects to the endpoint as an SSE (Server-Sent Events) stream and delivers each event to the client as a GraphQL subscription update.

This is useful for consuming REST SSE backends that emit raw JSON events (not wrapped in a GraphQL response envelope).

### How It Works

1. GQLForge sends a request to the URL specified in the directive with an `Accept: text/event-stream` header.
2. The upstream server responds with an SSE stream where each event contains raw JSON in `data:` lines.
3. GQLForge parses each SSE event as JSON and delivers it directly to the subscribing client.

### Example

```graphql
schema @server(port: 8000) {
  query: Query
  subscription: Subscription
}

type Query {
  latestSensor: SensorData @http(url: "https://api.example.com/sensors/latest")
}

type Subscription {
  sensorData: SensorData @http(url: "https://api.example.com/sse/sensors")
}

type SensorData {
  temperature: Float!
  humidity: Float!
  timestamp: String!
}
```

Subscribe via SSE:

```bash
curl -N -X POST http://localhost:8000/graphql/stream \
  -H "Content-Type: application/json" \
  -d '{"query": "subscription { sensorData { temperature humidity timestamp } }"}'
```

### Upstream SSE Format

The upstream server must deliver events in the standard SSE format. Each event should contain a raw JSON object in `data:` lines (no GraphQL response wrapper):

```
data: {"temperature": 25.3, "humidity": 60, "timestamp": "2026-01-15T10:30:00Z"}

data: {"temperature": 25.5, "humidity": 59, "timestamp": "2026-01-15T10:31:00Z"}
```

> **Note**: `batch_key` and `select` are not supported for streaming subscriptions.

> **See also**: For proxying subscriptions from an upstream GraphQL server via SSE, see [@graphQL Directive — Subscriptions](/docs/directives/graphql/#subscriptions-sse-streaming). For gRPC server-streaming subscriptions, see [gRPC Support — Streaming Subscriptions](/docs/grpc/#streaming-subscriptions).
