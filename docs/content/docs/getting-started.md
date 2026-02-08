+++
title = "Getting Started"
description = "Install GQLForge and create your first GraphQL API in minutes."
+++

# Getting Started

## Installation

### Via Cargo

Install GQLForge using Cargo:

```bash
cargo install --git https://github.com/takumi3488/gqlforge
```

### Via Docker

You can also run GQLForge using Docker:

```bash
docker pull ghcr.io/takumi3488/gqlforge/gqlforge
docker run -p 8000:8000 -p 8081:8081 ghcr.io/takumi3488/gqlforge/gqlforge
```

After installation, verify it works:

```bash
gqlforge --version
```

## Create Your First Schema

Create a file called `app.graphql` with the following content:

```graphql
schema
  @server(port: 8000)
  @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
  email: String!
}

type Post {
  id: Int!
  title: String!
  body: String!
  userId: Int!
  user: User @http(path: "/users/{{.value.userId}}")
}
```

This schema defines a GraphQL API that proxies requests to the JSONPlaceholder REST API. The `@server` directive sets the port, and `@upstream` sets the base URL for all HTTP resolvers.

## Start the Server

Run GQLForge with your schema file:

```bash
gqlforge start ./app.graphql
```

The server starts on `http://localhost:8000`. A built-in GraphQL Playground is available at the same address in your browser.

## Query Your API

Open the playground or use curl to send a query:

```bash
curl -X POST http://localhost:8000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ posts { title user { name } } }"}'
```

This fetches all posts along with the name of each post's author. GQLForge automatically batches the user lookups to avoid redundant HTTP calls.

## Validate Your Configuration

Before deploying, check your schema for errors and potential N+1 query issues:

```bash
gqlforge check ./app.graphql
```

This command validates the configuration and reports any problems without starting the server.

## Next Steps

- Learn about all available [CLI commands](@/docs/cli.md)
- Explore [resolver directives](@/docs/directives/_index.md) like `@http`, `@grpc`, and `@graphQL`
- Understand the [context object](@/docs/context.md) for dynamic path and header templates
- Configure [runtime settings](@/docs/runtime-config.md) for production deployments
