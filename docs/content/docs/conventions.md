+++
title = "Configuration Conventions"
description = "Conventions for writing GQLForge configuration files."
+++

# Configuration Conventions

## File Format

GQLForge uses **GraphQL SDL (Schema Definition Language)** as its primary configuration format. Configuration files typically use the `.graphql` extension.

```graphql
schema @server(port: 8000) @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}
```

## Schema Structure

A GQLForge configuration file follows a consistent structure:

### 1. Schema Definition

The file begins with a `schema` block that declares the root operation types and applies global directives:

```graphql
schema @server(port: 8000, hostname: "0.0.0.0") @upstream(baseURL: "https://api.example.com") {
  query: Query
  mutation: Mutation
}
```

- `@server` controls the runtime behavior (port, hostname, workers).
- `@upstream` sets defaults for outbound HTTP connections.
- `@link` imports external files or definitions.

### 2. Type Definitions

After the schema block, define your GraphQL types with resolver directives:

```graphql
type Query {
  users: [User] @http(path: "/users")
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
  email: String!
  posts: [Post] @http(path: "/users/{{.value.id}}/posts")
}

type Post {
  id: Int!
  title: String!
  body: String!
}
```

### 3. Input Types and Enums

Define input types for mutations and enums as needed:

```graphql
input CreateUserInput {
  name: String!
  email: String!
}

enum UserRole {
  ADMIN
  MEMBER
  GUEST
}
```

## Naming Conventions

- **Types**: Use PascalCase (`User`, `BlogPost`, `CreateUserInput`).
- **Fields**: Use camelCase (`firstName`, `createdAt`, `userId`).
- **Enums**: Use SCREAMING_SNAKE_CASE for values (`ADMIN`, `IN_PROGRESS`).
- **Files**: Use lowercase with hyphens or simple names (`app.graphql`, `user-schema.graphql`).

## Multi-File Configurations

You can split your configuration across multiple files and pass them all to the CLI:

```bash
gqlforge start ./schema.graphql ./users.graphql ./posts.graphql
```

Alternatively, use the `@link` directive to import files from a primary configuration:

```graphql
schema @server(port: 8000) @link(type: Config, src: "./users.graphql") @link(type: Config, src: "./posts.graphql") {
  query: Query
}
```

## Directive Placement

- **Global directives** (`@server`, `@upstream`, `@link`) are applied to the `schema` definition.
- **Resolver directives** (`@http`, `@grpc`, `@graphQL`, `@call`, `@expr`, `@js`) are applied to individual fields.
- **Schema directives** (`@addField`, `@modify`, `@omit`, `@cache`, `@protected`) are applied to types or fields as appropriate.

## Comments

Standard GraphQL comments (lines starting with `#`) and description strings (triple-quoted blocks) are supported:

```graphql
"""
Represents a registered user in the system.
"""
type User {
  # Unique identifier
  id: Int!
  name: String!
}
```
