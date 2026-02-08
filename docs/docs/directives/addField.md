---
title: "@addField Directive"
description: "Add derived fields to types in the GQLForge schema."
sidebar_label: "@addField"
---

# @addField Directive

The `@addField` directive adds a new derived field to a type. The field's value is resolved from an existing path within the same type, enabling data reshaping without custom resolvers.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | The name of the new field to add. |
| `path` | [String] | A dot-path through the existing type structure to derive the value from. |

## Usage

Apply `@addField` on a type definition. The `path` traverses nested fields to produce the new field's value.

## Example

```graphql
schema @server(port: 8000) {
  query: Query
}

type Query {
  user(id: Int!): User
    @http(url: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User
  @addField(name: "city", path: ["address", "city"])
  @addField(name: "companyName", path: ["company", "name"]) {
  id: Int!
  name: String!
  email: String!
  address: Address
  company: Company
}

type Address {
  city: String
  street: String
}

type Company {
  name: String
}
```

With this configuration, `User.city` resolves to `user.address.city` and `User.companyName` resolves to `user.company.name`, flattening the nested structure.
