# postgres-date-time-types

```sql @file:events.sql
CREATE TABLE events (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMPTZ,
  scheduled_date DATE,
  start_time TIME
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "events.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type Event {
  id: Int
  createdAt: DateTime
  updatedAt: DateTime
  scheduledDate: Date
  startTime: String
}

type Query {
  events: [Event] @postgres(table: "events")
}
```
