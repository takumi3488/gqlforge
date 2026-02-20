# s3-multiple-connections

```graphql @schema
schema @server {
  query: Query
}

type Query {
  getPrimaryUrl(key: String!): String
  @s3(bucket: "primary-bucket", key: "{{.args.key}}", operation: GET_PRESIGNED_URL, linkId: "primary")
  getBackupUrl(key: String!): String
  @s3(bucket: "backup-bucket", key: "{{.args.key}}", operation: GET_PRESIGNED_URL, linkId: "backup")
}
```
