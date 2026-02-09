+++
title = "gRPC Support"
description = "Connect gRPC services to your GraphQL API."
+++

# gRPC Support

GQLForge can expose gRPC services through your GraphQL API using the `@grpc` directive and Protobuf definitions.

## Linking Protobuf Files

Register your `.proto` files with `@link` so GQLForge understands the service definitions:

```graphql
schema @link(type: Protobuf, src: "./protos/greeter.proto") {
  query: Query
}
```

If your proto files import from other directories, use `proto_paths` to specify search locations:

```graphql
schema
@link(
  type: Protobuf
  src: "./protos/greeter.proto"
  proto_paths: ["./protos", "./third_party"]
) {
  query: Query
}
```

## The @grpc Directive

Use `@grpc` on fields to map them to gRPC method calls:

```graphql
type Query {
  greeting(name: String!): GreetingResponse
  @grpc(
    service: "greeter.GreeterService"
    method: "SayHello"
    body: "{ name: {{.args.name}} }"
    url: "https://grpc.example.com:50051"
  )
}

type GreetingResponse {
  message: String
}
```

### Directive Arguments

| Argument    | Description                                    |
| ----------- | ---------------------------------------------- |
| `service`   | Fully qualified protobuf service name          |
| `method`    | RPC method to invoke                           |
| `url`       | Address of the gRPC server                     |
| `body`      | Request message template using Mustache syntax |
| `batch_key` | Fields to batch on for N+1 prevention          |

## Type Mapping

GQLForge automatically generates GraphQL types from your protobuf messages.

### Field Optionality (proto3)

In proto3, fields without the `optional` keyword are generated as non-null (`!`) GraphQL types, while explicitly `optional` fields are nullable:

| Proto3 Declaration          | GraphQL Type |
| --------------------------- | ------------ |
| `int32 id = 1;`             | `Int!`       |
| `optional int32 id = 1;`    | `Int`        |
| `string name = 2;`          | `String!`    |
| `optional string name = 2;` | `String`     |
| `MyMessage msg = 3;`        | `MyMessage`  |
| `repeated int32 ids = 4;`   | `[Int!]!`    |

Note: Message-type fields are always nullable because they have inherent presence semantics in proto3.

### Repeated Fields

Protobuf `repeated` fields are always present (defaulting to an empty list) and cannot contain null elements. GQLForge maps them as non-null lists of non-null elements:

| Proto Declaration               | GraphQL Type    |
| ------------------------------- | --------------- |
| `repeated int32 ids = 1;`       | `[Int!]!`       |
| `repeated string names = 2;`    | `[String!]!`    |
| `repeated MyMessage items = 3;` | `[MyMessage!]!` |

Example — given this proto definition:

```protobuf
syntax = "proto3";

message Movie {
  string name = 1;
  repeated string cast = 2;
  repeated Review reviews = 3;
}

message Review {
  int32 score = 1;
  string comment = 2;
}
```

GQLForge generates:

```graphql
type Movie {
  name: String!
  cast: [String!]!
  reviews: [Review!]!
}

type Review {
  score: Int!
  comment: String!
}
```

This applies to both proto3 and proto2 `repeated` fields.

### Scalar Types

Protobuf scalar types map to their GraphQL equivalents:

- `string` maps to `String`
- `int32` / `int64` maps to `Int`
- `float` / `double` maps to `Float`
- `bool` maps to `Boolean`
- Nested messages become GraphQL object types

## Example: Full Schema

```graphql
schema @server(port: 8000) @link(type: Protobuf, src: "./user.proto") {
  query: Query
}

type Query {
  user(id: Int!): User
  @grpc(
    service: "users.UserService"
    method: "GetUser"
    url: "https://localhost:50051"
    body: "{ id: {{.args.id}} }"
  )
}
```

GQLForge reads the proto definition, generates the `User` type, and routes incoming GraphQL queries to the gRPC backend.
