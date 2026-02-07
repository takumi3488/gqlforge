---
error: true
---

# test-experimental-headers-error

```yaml @config
server:
  headers:
    experimental: ["non-experimental", "foo", "bar", "gqlforge"]
```

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "World!")
}
```
