---
error: true
---

# s3-error-missing-key

```graphql @schema
schema @server {
  query: Query
}

type Query {
  getFileUrl: String @s3(bucket: "my-bucket", operation: GET_PRESIGNED_URL)
}
```
