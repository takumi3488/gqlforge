# s3-get-presigned-url

```graphql @schema
schema @server {
  query: Query
}

type Query {
  getFileUrl(key: String!): String @s3(bucket: "my-bucket", key: "{{.args.key}}", operation: GET_PRESIGNED_URL)
}
```
