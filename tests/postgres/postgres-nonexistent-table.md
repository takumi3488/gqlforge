---
error: true
---

# postgres-nonexistent-table

```sql @file:main.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
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

type Post {
  id: Int
  title: String
}

type Query {
  posts: [Post] @postgres(table: "posts", db: "main")
}
```
