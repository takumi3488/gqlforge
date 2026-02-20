# s3-put-presigned-url

```graphql @schema
schema @server {
  query: Query
}

type Query {
  putFileUrl(key: String!, contentType: String): String
  @s3(
    bucket: "my-bucket"
    key: "{{.args.key}}"
    contentType: "{{.args.contentType}}"
    operation: PUT_PRESIGNED_URL
  )
}
```
