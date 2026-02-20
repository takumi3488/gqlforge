# s3-list-objects

```graphql @schema
schema @server {
  query: Query
}

type Query {
  listFiles(prefix: String): JSON @s3(bucket: "my-bucket", prefix: "{{.args.prefix}}", operation: LIST)
}
```
