+++
title = "CLI Reference"
description = "Complete reference for all GQLForge CLI commands."
+++

# CLI Reference

## Overview

GQLForge provides four primary commands for working with GraphQL configurations.

```
gqlforge <command> [options]
```

## Commands

### `start`

Starts the GraphQL server using the provided configuration files.

```bash
gqlforge start <file_paths...> [options]
```

**Arguments:**

| Argument     | Required | Description                                      |
| ------------ | -------- | ------------------------------------------------ |
| `file_paths` | Yes      | One or more paths to GraphQL configuration files |

**Options:**

| Flag           | Description                                                         |
| -------------- | ------------------------------------------------------------------- |
| `--verify-ssl` | Enable strict SSL certificate verification for upstream connections |

**Examples:**

```bash
# Start with a single config file
gqlforge start ./app.graphql

# Start with multiple config files
gqlforge start ./schema.graphql ./extensions.graphql

# Start with SSL verification enabled
gqlforge start ./app.graphql --verify-ssl
```

---

### `check`

Validates the configuration without starting the server. Reports schema errors, type mismatches, and optionally detects N+1 query patterns.

```bash
gqlforge check <file_paths...> [options]
```

**Arguments:**

| Argument     | Required | Description                                      |
| ------------ | -------- | ------------------------------------------------ |
| `file_paths` | Yes      | One or more paths to GraphQL configuration files |

**Options:**

| Flag                   | Description                                    |
| ---------------------- | ---------------------------------------------- |
| `--n-plus-one-queries` | Detect and report potential N+1 query patterns |
| `--schema`             | Output the composed schema to stdout           |
| `--verify-ssl`         | Enable strict SSL certificate verification     |

**Examples:**

```bash
# Basic validation
gqlforge check ./app.graphql

# Check for N+1 issues
gqlforge check ./app.graphql --n-plus-one-queries

# Output the composed schema
gqlforge check ./app.graphql --schema
```

---

### `init`

Creates a new GQLForge project with a starter configuration file.

```bash
gqlforge init [folder_path]
```

**Arguments:**

| Argument      | Required | Default | Description                                 |
| ------------- | -------- | ------- | ------------------------------------------- |
| `folder_path` | No       | `.`     | Directory where the project will be created |

**Examples:**

```bash
# Initialize in the current directory
gqlforge init

# Initialize in a new directory
gqlforge init ./my-api
```

---

### `gen`

Generates GQLForge configuration from existing API definitions such as REST API specs, Protocol Buffer files, GraphQL schemas, or PostgreSQL databases.

```bash
gqlforge gen <file_path>
```

**Arguments:**

| Argument    | Required | Description                                                     |
| ----------- | -------- | --------------------------------------------------------------- |
| `file_path` | Yes      | Path to a source definition file or a PostgreSQL connection URL |

**Examples:**

```bash
# Generate from a protobuf file
gqlforge gen ./service.proto

# Generate from an OpenAPI spec
gqlforge gen ./openapi.json

# Generate from a PostgreSQL database
gqlforge gen postgres://user:password@localhost:5432/mydb
```

See [Config Generation](@/docs/config-generation.md) for more details on supported source formats.
