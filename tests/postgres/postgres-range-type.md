# postgres-range-type

```sql @file:reservations.sql
CREATE TABLE reservations (
  id SERIAL PRIMARY KEY,
  booking_period INT4RANGE,
  price_range NUMRANGE
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "reservations.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type Reservation {
  id: Int
  bookingPeriod: String
  priceRange: String
}

type Query {
  reservations: [Reservation] @postgres(table: "reservations")
}
```
