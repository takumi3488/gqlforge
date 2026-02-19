# postgres-array-type

```sql @file:products.sql
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  tags TEXT[],
  scores INT[]
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "products.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type Product {
  id: Int
  tags: JSON
  scores: JSON
}

type Query {
  products: [Product] @postgres(table: "products")
}
```
