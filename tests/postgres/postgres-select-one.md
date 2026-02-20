# postgres-select-one

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
  user(id: Int!): User @postgres(table: "users", operation: SELECT_ONE, filter: { id: "{{.args.id}}" })
}
```
