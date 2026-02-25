# postgres-view-select

```sql @file:schema.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  email VARCHAR(255)
);

CREATE VIEW active_users AS
  SELECT id, name, email FROM users WHERE email IS NOT NULL;
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "schema.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type ActiveUser {
  id: Int
  name: String
  email: String
}

type Query {
  activeUsers(limit: Int, offset: Int): [ActiveUser]
  @postgres(table: "active_users", limit: "{{.args.limit}}", offset: "{{.args.offset}}")
}
```
