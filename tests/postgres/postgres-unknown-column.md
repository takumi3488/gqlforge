# postgres-unknown-column

```sql @file:users.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
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
}

type Query {
  users: [User] @postgres(table: "users", orderBy: "nonexistent_column ASC")
}
```
