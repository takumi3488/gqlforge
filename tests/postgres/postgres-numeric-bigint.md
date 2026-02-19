# postgres-numeric-bigint

```sql @file:metrics.sql
CREATE TABLE metrics (
  id SERIAL PRIMARY KEY,
  counter BIGINT,
  ratio NUMERIC(10, 4),
  amount DECIMAL(15, 2)
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "metrics.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type Metric {
  id: Int
  counter: String
  ratio: Float
  amount: Float
}

type Query {
  metrics: [Metric] @postgres(table: "metrics")
}
```
