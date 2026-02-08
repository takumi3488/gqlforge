---
title: "GraphQL Playground"
description: "Use the built-in GraphQL Playground to explore and test your API."
sidebar_label: "Playground"
---

## Overview

GQLForge includes a built-in GraphQL Playground that launches automatically when you start the server. It provides an interactive environment for writing queries, exploring your schema, and inspecting responses.

## Accessing the Playground

After starting the server with `gqlforge start`, open your browser and navigate to the server address:

```
http://localhost:8000
```

The Playground loads at the root URL by default. The port depends on your `@server` configuration.

## Features

### Query Editor

Write and execute GraphQL queries, mutations, and subscriptions directly in the browser. The editor includes syntax highlighting and auto-completion based on your schema.

```graphql
{
  posts {
    id
    title
    user {
      name
      email
    }
  }
}
```

### Schema Explorer

Browse the full schema documentation generated from your configuration. View all types, fields, arguments, and their descriptions without leaving the Playground.

### Request Headers

Add custom HTTP headers to your requests using the headers panel at the bottom of the editor. This is useful for testing authenticated endpoints:

```json
{
  "Authorization": "Bearer your-token-here"
}
```

### Query Variables

Pass variables to your queries through the variables panel:

```json
{
  "userId": 1
}
```

With a corresponding query:

```graphql
query GetUser($userId: Int!) {
  user(id: $userId) {
    name
    email
  }
}
```

## Configuration

The Playground is enabled by default in the `@server` directive. You can disable it for production deployments if needed by adjusting the server configuration.

## Usage in Development

The Playground is particularly useful during development for:

- Verifying that resolvers return the expected data
- Testing field arguments and input types
- Inspecting the composed schema after merging multiple config files
- Debugging query execution by examining response payloads
