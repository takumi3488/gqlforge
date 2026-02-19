# s3-delete-object

```graphql @schema
schema @server {
  query: Query
}

type Query {
  deleteFile(key: String!): JSON @s3(bucket: "my-bucket", key: "{{.args.key}}", operation: DELETE)
}
```
