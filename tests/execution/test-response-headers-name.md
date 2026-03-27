---
error: true
---

# test-response-headers-name

```yaml @config
server:
  headers:
    # sakoku-ignore-next-line
    custom: [{ key: "🤣", value: "a" }]
```

```graphql @schema
schema {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @expr(body: { name: "John" })
}
```
