+++
title = "@postgres Directive"
description = "Resolve GraphQL fields by querying a PostgreSQL table."
+++

# @postgres Directive

The `@postgres` directive resolves a field by executing a SQL operation against a PostgreSQL table. It requires a database connection linked via `@link(type: Postgres, src: "postgres://...")`.

## Fields

| Field       | Type              | Default  | Description                                                                             |
| ----------- | ----------------- | -------- | --------------------------------------------------------------------------------------- |
| `db`        | String            | `null`   | The `@link(type: Postgres)` id to use. Optional when only one Postgres link is defined. |
| `table`     | String            | Required | Target table name (optionally schema-qualified, e.g. `"public.users"`).                 |
| `operation` | PostgresOperation | `SELECT` | The CRUD operation to perform. See below.                                               |
| `filter`    | JSON              | `null`   | A JSON object describing the WHERE clause. Supports Mustache templates.                 |
| `input`     | String            | `null`   | Input data source for INSERT/UPDATE. Typically `"{{.args.input}}"`.                     |
| `batchKey`  | [String]          | `[]`     | Columns used for DataLoader batch keys (N+1 prevention).                                |
| `dedupe`    | Boolean           | `false`  | Deduplicate identical in-flight database calls.                                         |
| `limit`     | String            | `null`   | Mustache template for the LIMIT clause, e.g. `"{{.args.limit}}"`.                       |
| `offset`    | String            | `null`   | Mustache template for the OFFSET clause, e.g. `"{{.args.offset}}"`.                     |
| `orderBy`   | String            | `null`   | Mustache template for the ORDER BY clause, e.g. `"{{.args.orderBy}}"`.                  |

## PostgresOperation

| Value        | Description                                              |
| ------------ | -------------------------------------------------------- |
| `SELECT`     | Select multiple rows. Returns a list.                    |
| `SELECT_ONE` | Select a single row by primary key or unique constraint. |
| `INSERT`     | Insert a new row and return the created record.          |
| `UPDATE`     | Update an existing row and return the updated record.    |
| `DELETE`     | Delete a row.                                            |

## Examples

### Fetching a single record by ID

```graphql
type Query {
  userById(id: Int!): User
  @postgres(
    table: "users"
    operation: SELECT_ONE
    filter: { id: "{{.args.id}}" }
  )
}
```

### Inserting a record

```graphql
type Mutation {
  createUser(input: CreateUserInput!): User
  @postgres(
    table: "users"
    operation: INSERT
    input: "{{.args.input}}"
  )
}
```

### Paginated list with ordering

```graphql
type Query {
  usersList(limit: Int, offset: Int, orderBy: String): [User!]!
  @postgres(
    table: "users"
    operation: SELECT
    limit: "{{.args.limit}}"
    offset: "{{.args.offset}}"
    orderBy: "{{.args.orderBy}}"
  )
}
```

### Batched relationship (N+1 prevention)

```graphql
type Post {
  author: User
  @postgres(
    table: "users"
    operation: SELECT_ONE
    filter: { id: "{{.value.userId}}" }
    batchKey: ["id"]
  )
}
```

### Multiple databases

When multiple `@link(type: Postgres)` are defined, use the `db` field to specify which connection to query:

```graphql
schema
@server(port: 8000)
@link(id: "main", type: Postgres, src: "postgres://localhost:5432/main_db")
@link(id: "analytics", type: Postgres, src: "postgres://localhost:5432/analytics_db") {
  query: Query
}

type Query {
  userById(id: Int!): User @postgres(db: "main", table: "users", operation: SELECT_ONE, filter: { id: "{{.args.id}}" })

  pageViews(limit: Int): [PageView!]!
  @postgres(db: "analytics", table: "page_views", operation: SELECT, limit: "{{.args.limit}}")
}
```

When only one `@link(type: Postgres)` is defined, the `db` field can be omitted.

## Security

All dynamic values referenced by Mustache templates in `filter`, `input`, `limit`, `offset`, and `orderBy` are passed as parameterised query arguments — they are never interpolated into SQL text. Table and column names are escaped using `quote_ident` to prevent SQL injection.
