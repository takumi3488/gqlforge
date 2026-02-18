# postgres-single-connection

```sql @file:main.sql
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
    src: "main.sql"
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
  users: [User] @postgres(table: "users")
}
```
