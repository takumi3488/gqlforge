# postgres-json-jsonb-type

```sql @file:records.sql
CREATE TABLE records (
  id SERIAL PRIMARY KEY,
  metadata JSON,
  settings JSONB
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "records.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type Record {
  id: Int
  metadata: JSON
  settings: JSON
}

type Query {
  records: [Record] @postgres(table: "records")
}
```
