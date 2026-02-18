---
error: true
---

# postgres-duplicate-link-id

```sql @file:main.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
);
```

```sql @file:other.sql
CREATE TABLE posts (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "main.sql"
  - id: "main"
    type: Sql
    src: "other.sql"
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
  users: [User] @expr(body: "[]")
}
```
