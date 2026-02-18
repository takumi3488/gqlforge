---
error: true
---

# postgres-db-required-for-multiple

```sql @file:main.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
);
```

```sql @file:analytics.sql
CREATE TABLE events (
  id SERIAL PRIMARY KEY,
  event_type TEXT NOT NULL
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "main.sql"
  - id: "analytics"
    type: Sql
    src: "analytics.sql"
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
  users: [User] @postgres(table: "users")
}
```
