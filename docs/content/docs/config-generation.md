+++
title = "Config Generation"
description = "Generate GQLForge configuration from existing API definitions."
+++

# Config Generation

## Overview

The `gqlforge gen` command reads existing API definitions and produces a GQLForge-compatible GraphQL configuration. This automates the process of writing resolver mappings by hand when you already have a formal API specification.

## Usage

```bash
gqlforge gen <file_path>
```

The command accepts a path to a source definition file and writes the generated configuration to stdout. You can redirect the output to a file:

```bash
gqlforge gen ./petstore.json > app.graphql
```

## Supported Source Formats

### OpenAPI / Swagger

Generate a GraphQL schema from a REST API specification:

```bash
gqlforge gen ./openapi.json
```

The generator maps REST endpoints to GraphQL fields with `@http` directives. Path parameters, query parameters, and request/response schemas are translated into GraphQL types and arguments.

For example, an endpoint like `GET /users/{id}` becomes:

```graphql
type Query {
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}
```

### Protocol Buffers

Generate a GraphQL schema from `.proto` files:

```bash
gqlforge gen ./service.proto
```

Protobuf messages are converted to GraphQL types, and RPC methods are mapped to fields with `@grpc` directives. The generated schema preserves the service and method hierarchy.

### GraphQL Schemas

Generate a GQLForge configuration from an existing GraphQL schema or endpoint:

```bash
gqlforge gen ./schema.graphql
```

This is useful for wrapping an existing GraphQL service with GQLForge's optimization layer, adding caching, or composing multiple GraphQL sources.

## Output Structure

The generated configuration follows GQLForge conventions:

- A `schema` block with `@server` and `@upstream` directives
- Type definitions with appropriate resolver directives
- Input types derived from request body schemas

## Customizing Generated Output

The generated configuration serves as a starting point. After generation, you can:

- Adjust field names or types to match your preferred GraphQL conventions
- Add `@cache` directives for frequently accessed fields
- Remove endpoints you do not want to expose
- Add relationships between types using `@http` or `@call` directives
- Apply `@omit` or `@modify` to reshape the schema

## Example Workflow

1. Generate an initial configuration from your API spec:
   ```bash
   gqlforge gen ./api-spec.json > app.graphql
   ```

2. Validate the generated schema:
   ```bash
   gqlforge check ./app.graphql
   ```

3. Customize the output as needed in your editor.

4. Start the server:
   ```bash
   gqlforge start ./app.graphql
   ```
