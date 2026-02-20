# s3-delete-object

```graphql @schema
schema @server {
  query: Query
  mutation: Mutation
}

type Query {
  getFileUrl(key: String!): String @s3(bucket: "my-bucket", key: "{{.args.key}}", operation: GET_PRESIGNED_URL)
}

type Mutation {
  deleteFile(key: String!): JSON @s3(bucket: "my-bucket", key: "{{.args.key}}", operation: DELETE)
}
```
