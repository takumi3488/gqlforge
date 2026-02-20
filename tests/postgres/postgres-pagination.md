# postgres-pagination

```sql @file:users.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  email VARCHAR(255)
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "users.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type User {
  id: Int
  name: String
  email: String
}

type Query {
  users(limit: Int, offset: Int, orderBy: String): [User]
  @postgres(
    table: "users"
    limit: "{{.args.limit}}"
    offset: "{{.args.offset}}"
    orderBy: "{{.args.orderBy}}"
  )
}
```
