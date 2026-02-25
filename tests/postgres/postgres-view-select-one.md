# postgres-view-select-one

```sql @file:schema.sql
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  price NUMERIC NOT NULL,
  active BOOLEAN NOT NULL DEFAULT true
);

CREATE VIEW active_products AS
  SELECT id, name, price FROM products WHERE active = true;
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

type ActiveProduct {
  id: Int
  name: String
  price: Float
}

type Query {
  activeProduct(id: Int!): ActiveProduct
  @postgres(table: "active_products", operation: SELECT_ONE, filter: { id: "{{.args.id}}" })
}
```
