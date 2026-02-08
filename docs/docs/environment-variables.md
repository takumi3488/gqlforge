---
title: "Environment Variables"
description: "Use environment variables in your GQLForge configuration."
sidebar_label: "Environment Variables"
---

## Overview

GQLForge supports referencing environment variables within your configuration files using Mustache template syntax. This allows you to keep secrets, API keys, and environment-specific values out of your configuration files.

## Syntax

Use `{{.env.VAR_NAME}}` to reference an environment variable:

```graphql
schema
  @server(port: 8000)
  @upstream(baseURL: "{{.env.API_BASE_URL}}") {
  query: Query
}
```

The variable is resolved at server startup from the process environment.

## Common Use Cases

### API Keys and Secrets

Pass authentication credentials to upstream services without hardcoding them:

```graphql
type Query {
  users: [User]
    @http(
      path: "/users"
      headers: [{ key: "Authorization", value: "Bearer {{.env.API_TOKEN}}" }]
    )
}
```

Start the server with the variable set:

```bash
API_TOKEN=your-secret-token gqlforge start ./app.graphql
```

### Base URLs per Environment

Use different upstream URLs for development, staging, and production:

```graphql
schema
  @upstream(baseURL: "{{.env.UPSTREAM_URL}}") {
  query: Query
}
```

```bash
# Development
UPSTREAM_URL=http://localhost:3000 gqlforge start ./app.graphql

# Production
UPSTREAM_URL=https://api.example.com gqlforge start ./app.graphql
```

### Database Connection Strings

Reference connection parameters for data sources:

```graphql
schema
  @server(port: 8000)
  @link(type: Config, src: "{{.env.CONFIG_PATH}}") {
  query: Query
}
```

### Custom Headers

Inject environment-specific headers into upstream requests:

```graphql
type Query {
  data: Data
    @http(
      path: "/data"
      headers: [
        { key: "X-Api-Key", value: "{{.env.DATA_API_KEY}}" }
        { key: "X-Environment", value: "{{.env.DEPLOY_ENV}}" }
      ]
    )
}
```

## Context Variable Access

Environment variables are also available through the `.vars` context during template resolution. Both `.env.VAR_NAME` and `.vars.VAR_NAME` reference the same underlying environment variables. See [Context Object](./context.md) for the full set of template variables.

## Behavior

- If a referenced environment variable is not set, the template renders as an empty string.
- Environment variables are read once at server startup. Changes to the environment after startup require a server restart.
- Variable names are case-sensitive and follow the host operating system's conventions.

## Best Practices

- Use environment variables for any value that differs between environments (URLs, ports, keys).
- Avoid committing secrets to version control. Use `.env` files or secret managers to inject values.
- Document required environment variables in your project so that others can set up the server correctly.
