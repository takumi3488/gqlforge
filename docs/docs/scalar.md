---
title: "Scalar Types"
description: "Custom scalar types supported by GQLForge."
sidebar_label: "Scalar Types"
---

# Scalar Types

GQLForge supports several custom scalar types beyond the standard GraphQL scalars. These provide built-in validation and serialization for common data formats.

## Built-in Custom Scalars

| Scalar | Description | Example Value |
|--------|-------------|---------------|
| `Date` | Calendar date in ISO 8601 format | `"2025-01-15"` |
| `DateTime` | Date and time with timezone | `"2025-01-15T09:30:00Z"` |
| `Email` | Validated email address | `"user@example.com"` |
| `JSON` | Arbitrary JSON value | `{"key": "value"}` |
| `PhoneNumber` | Phone number string | `"+1-555-0100"` |
| `Url` | Validated URL string | `"https://example.com"` |
| `Int64` | 64-bit integer | `9223372036854775807` |
| `UInt64` | Unsigned 64-bit integer | `18446744073709551615` |
| `Bytes` | Base64-encoded binary data | `"SGVsbG8="` |
| `Empty` | Represents no value (unit type) | `null` |

## Usage in Schema

Use custom scalars just like standard types in your type definitions:

```graphql
type User {
  id: Int!
  email: Email!
  website: Url
  createdAt: DateTime!
  birthday: Date
  metadata: JSON
}
```

## Validation

GQLForge validates values against their scalar type at runtime. For example, passing `"not-an-email"` to an `Email` field produces a validation error without needing custom logic.

## JSON Scalar

The `JSON` scalar is especially useful for fields with dynamic or unstructured data:

```graphql
type Config {
  settings: JSON
}
```

This allows any valid JSON structure to be stored and returned, giving flexibility when the shape of the data is not known ahead of time.

## Bytes Scalar

The `Bytes` scalar is primarily used with gRPC integration, where protobuf `bytes` fields map to base64-encoded strings in GraphQL:

```graphql
type FileResponse {
  content: Bytes
  filename: String
}
```
