# postgres-multiple-connections

```sql @file:main.sql
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
);
```

```sql @file:analytics.sql
CREATE TABLE events (
  id SERIAL PRIMARY KEY,
  event_type TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);
```

```yaml @config
links:
  - id: "main"
    type: Sql
    src: "main.sql"
  - id: "analytics"
    type: Sql
    src: "analytics.sql"
```

```graphql @schema
schema @server {
  query: Query
}

type User {
  id: Int
  name: String
}

type Event {
  id: Int
  eventType: String
  createdAt: DateTime
}

type Query {
  users: [User] @postgres(table: "users", db: "main")
  events: [Event] @postgres(table: "events", db: "analytics")
}
```
