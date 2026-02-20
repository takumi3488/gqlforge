# postgres-uuid-type

```sql @file:items.sql
CREATE TABLE items (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "items.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type Item {
  id: ID
  name: String
}

type Query {
  items: [Item] @postgres(table: "items")
}
```
