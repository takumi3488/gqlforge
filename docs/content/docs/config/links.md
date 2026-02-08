+++
title = "@link Directive"
description = "Link external resources like proto files, scripts, and certificates."
+++

# @link Directive

The `@link` directive connects external resources to your GQLForge configuration. Use it to import protobuf definitions, JavaScript handlers, TLS certificates, and more.

## Fields

| Field         | Type     | Required | Description                                                     |
| ------------- | -------- | -------- | --------------------------------------------------------------- |
| `id`          | String   | No       | An identifier used to reference this linked resource elsewhere. |
| `src`         | String   | Yes      | URL or local file path to the resource.                         |
| `type_of`     | LinkType | Yes      | The kind of resource being linked. See table below.             |
| `headers`     | [Header] | No       | HTTP headers sent when fetching a remote `src`.                 |
| `meta`        | JSON     | No       | Arbitrary metadata attached to the link.                        |
| `proto_paths` | [String] | No       | Additional search paths for protobuf imports.                   |

## Link Types

| Type        | Description                                                           |
| ----------- | --------------------------------------------------------------------- |
| `Config`    | Another GQLForge configuration file to merge into the current schema. |
| `Protobuf`  | A `.proto` file defining gRPC service interfaces.                     |
| `Script`    | A JavaScript file providing custom resolver functions.                |
| `Cert`      | A TLS certificate file (PEM format).                                  |
| `Key`       | A TLS private key file (PEM format).                                  |
| `Operation` | A file containing persisted GraphQL operations.                       |
| `Htpasswd`  | An htpasswd file for basic authentication.                            |
| `Jwks`      | A JWKS endpoint or file for JWT validation.                           |
| `Grpc`      | A gRPC reflection endpoint for service discovery.                     |

## Examples

### Linking a JavaScript handler

```graphql
schema @link(src: "./handlers.js", type_of: Script) @server(port: 8000) {
  query: Query
}
```

### Linking a protobuf file

```graphql
schema
@link(
  id: "news"
  src: "./news.proto"
  type_of: Protobuf
  proto_paths: ["./proto"]
)
@server(port: 8000) {
  query: Query
}
```

### Linking a JWKS provider for authentication

```graphql
schema @link(id: "jwks", src: "https://example.com/.well-known/jwks.json", type_of: Jwks) @server(port: 8000) {
  query: Query
}
```
