# postgres-insert

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
  mutation: Mutation
}

type User {
  id: Int
  name: String
  email: String
}

type Query {
  users: [User] @postgres(table: "users")
}

type Mutation {
  createUser(name: String!, email: String): User
  @postgres(table: "users", operation: INSERT, input: "{\"name\": \"{{.args.name}}\", \"email\": \"{{.args.email}}\"}")
}
```
