---
error: true
---

# postgres-view-insert-error

```sql @file:schema.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE VIEW active_users AS
  SELECT id, name FROM users;
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
  mutation: Mutation
}

type User {
  id: Int
  name: String
}

type Query {
  users: [User] @postgres(table: "active_users", db: "main")
}

type Mutation {
  createUser(name: String!): User
  @postgres(table: "active_users", operation: INSERT, input: "{\"name\": \"{{.args.name}}\"}", db: "main")
}
```
