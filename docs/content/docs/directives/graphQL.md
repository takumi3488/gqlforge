+++
title = "@graphQL Directive"
description = "Proxy field resolution to another GraphQL server."
+++

# @graphQL Directive

The `@graphQL` directive resolves a field by forwarding the query to a remote GraphQL endpoint. This enables schema stitching and federation-like composition.

## Fields

| Field        | Type     | Default          | Description                                                              |
| ------------ | -------- | ---------------- | ------------------------------------------------------------------------ |
| `url`        | String   | Required         | The remote GraphQL endpoint URL.                                         |
| `name`       | String   | `null`           | The remote field name if it differs from the local field name.           |
| `args`       | [Arg]    | `[]`             | Arguments to pass to the remote query.                                   |
| `headers`    | [Header] | `[]`             | HTTP headers sent to the remote server.                                  |
| `batch`      | Boolean  | `false`          | Enable request batching for this remote endpoint.                        |
| `dedupe`     | Boolean  | `false`          | Deduplicate identical in-flight requests to the remote.                  |
| `stream_url` | String   | `{url}/stream`   | SSE endpoint URL for upstream subscriptions. Only used for Subscription fields. |

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

## Subscriptions (SSE Streaming)

When a `@graphQL` directive is placed on a field under the `Subscription` root type, GQLForge connects to the upstream server's SSE endpoint and relays events to the client.

### How It Works

1. GQLForge sends the subscription query to the upstream SSE endpoint via POST with `Accept: text/event-stream`.
2. The upstream server responds with an SSE stream where each event contains a JSON GraphQL response in `data:` lines.
3. GQLForge parses each SSE event, extracts the specified field from the response, and delivers it to the client.

### Example

```graphql
schema @server(port: 8000) {
  query: Query
  subscription: Subscription
}

type Query {
  messages: [Message]
  @graphQL(
    url: "https://chat.example.com/graphql"
    name: "messages"
  )
}

type Subscription {
  newMessage: Message
  @graphQL(
    url: "https://chat.example.com/graphql"
    name: "newMessage"
  )
}

type Message {
  id: ID!
  text: String!
  sender: String!
}
```

Subscribe via SSE:

```bash
curl -N -X POST http://localhost:8000/graphql/stream \
  -H "Content-Type: application/json" \
  -d '{"query": "subscription { newMessage { id text sender } }"}'
```

### Custom SSE Endpoint

By default, GQLForge sends subscription requests to `{url}/stream`. If the upstream server uses a different SSE endpoint, specify it with `stream_url`:

```graphql
type Subscription {
  newMessage: Message
  @graphQL(
    url: "https://chat.example.com/graphql"
    name: "newMessage"
    stream_url: "https://chat.example.com/subscriptions/sse"
  )
}
```

### Upstream SSE Format

The upstream server must deliver events in the standard SSE format. Each event should contain a JSON GraphQL response:

```
data: {"data":{"newMessage":{"id":"1","text":"hello","sender":"alice"}}}

data: {"data":{"newMessage":{"id":"2","text":"world","sender":"bob"}}}
```

> **See also**: For gRPC server-streaming subscriptions, see [gRPC Support — Streaming Subscriptions](/docs/grpc/#streaming-subscriptions).
