# postgres-view-materialized

```sql @file:schema.sql
CREATE TABLE orders (
  id SERIAL PRIMARY KEY,
  amount NUMERIC NOT NULL,
  status TEXT NOT NULL
);

CREATE MATERIALIZED VIEW completed_orders AS
  SELECT id, amount FROM orders WHERE status = 'completed';
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

type CompletedOrder {
  id: Int
  amount: Float
}

type Query {
  completedOrders(limit: Int, offset: Int): [CompletedOrder]
  @postgres(table: "completed_orders", limit: "{{.args.limit}}", offset: "{{.args.offset}}")
}
```
