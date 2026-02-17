+++
title = "PostgreSQL Support"
description = "Expose a PostgreSQL database as a GraphQL API."
+++

# PostgreSQL Support

GQLForge can expose PostgreSQL databases as a fully-typed GraphQL API using the `@postgres` directive and SQL schema definitions.

## Connecting to PostgreSQL

Register a database connection with `@link` using the `Postgres` link type:

```graphql
schema @server(port: 8000) @link(type: Postgres, src: "postgres://user:password@localhost:5432/mydb") {
  query: Query
  mutation: Mutation
}
```

### Loading SQL Migrations

You can also provide a SQL file containing `CREATE TABLE` statements using the `Sql` link type. GQLForge parses the DDL to understand your table structures without connecting to a live database at configuration time:

```graphql
schema
@server(port: 8000)
@link(type: Postgres, src: "postgres://localhost:5432/mydb")
@link(type: Sql, src: "./migrations/schema.sql") {
  query: Query
  mutation: Mutation
}
```

## Type Mapping

PostgreSQL column types are automatically mapped to GraphQL scalars:

| PostgreSQL Type                               | GraphQL Scalar |
| --------------------------------------------- | -------------- |
| `smallint`, `integer`, `serial`               | `Int`          |
| `bigint`, `bigserial`                         | `String`       |
| `real`, `double precision`, `numeric`         | `Float`        |
| `boolean`                                     | `Boolean`      |
| `uuid`                                        | `ID`           |
| `json`, `jsonb`                               | `JSON`         |
| `date`, `timestamp`, `timestamptz`            | `DateTime`     |
| `text`, `varchar`, `char`                     | `String`       |
| `bytea`                                       | `String`       |
| `time`, `interval`, `inet`, `cidr`, `macaddr` | `String`       |
| Array types (e.g. `integer[]`)                | `JSON`         |

## The @postgres Directive

Use `@postgres` on fields to map them to table operations:

```graphql
type Query {
  userById(id: Int!): User
  @postgres(
    table: "users"
    operation: SELECT_ONE
    filter: { id: "{{.args.id}}" }
  )

  usersList(limit: Int, offset: Int): [User!]!
  @postgres(
    table: "users"
    operation: SELECT
    limit: "{{.args.limit}}"
    offset: "{{.args.offset}}"
  )
}

type Mutation {
  createUser(input: CreateUserInput!): User
  @postgres(
    table: "users"
    operation: INSERT
    input: "{{.args.input}}"
  )
}
```

See [@postgres Directive](@/docs/directives/postgres.md) for the full field reference.

## Schema Auto-Generation

The `gqlforge gen` command can introspect a live PostgreSQL database and generate a complete GraphQL schema automatically:

```bash
gqlforge gen postgres://user:password@localhost:5432/mydb > app.graphql
```

### Generated Operations

For each table, the generator creates:

| Pattern        | Operation    | Description                       |
| -------------- | ------------ | --------------------------------- |
| `{table}ById`  | `SELECT_ONE` | Fetch a single row by primary key |
| `{table}List`  | `SELECT`     | Paginated list with limit/offset  |
| `create{Type}` | `INSERT`     | Create a new record               |
| `update{Type}` | `UPDATE`     | Update a record by primary key    |
| `delete{Type}` | `DELETE`     | Delete a record by primary key    |

Table names are converted to PascalCase for type names and camelCase for field names.

### Foreign Key Relationships

The generator automatically detects foreign key constraints and creates nested fields:

- **Belongs-to**: A `posts.user_id → users.id` foreign key adds a `users` field on the `Posts` type that resolves via `SELECT_ONE` with `batchKey` for N+1 prevention.
- **Has-many**: The inverse relationship adds a pluralised list field (e.g. `postsList`) on the `Users` type.

## Example: Full Schema

Given a database with `users` and `posts` tables:

```graphql
schema
@server(port: 8000)
@link(type: Postgres, src: "postgres://localhost:5432/mydb")
@link(type: Sql, src: "./schema.sql") {
  query: Query
  mutation: Mutation
}

type Query {
  usersById(id: Int!): Users @postgres(table: "users", operation: SELECT_ONE, filter: { id: "{{.args.id}}" })

  usersList(limit: Int, offset: Int): [Users!]!
  @postgres(table: "users", operation: SELECT, limit: "{{.args.limit}}", offset: "{{.args.offset}}")

  postsById(id: Int!): Posts @postgres(table: "posts", operation: SELECT_ONE, filter: { id: "{{.args.id}}" })
}

type Users {
  id: Int!
  name: String!
  email: String
  postsList: [Posts!]!
  @postgres(table: "posts", operation: SELECT, filter: { user_id: "{{.value.id}}" }, batchKey: ["user_id"])
}

type Posts {
  id: Int!
  userId: Int!
  title: String!
  users: Users @postgres(table: "users", operation: SELECT_ONE, filter: { id: "{{.value.userId}}" }, batchKey: ["id"])
}
```
