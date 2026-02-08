---
title: "Directives Overview"
description: "Overview of all available GQLForge directives."
sidebar_label: "Overview"
---

# Directives Overview

GQLForge uses GraphQL schema directives to declaratively configure data fetching, transformation, and access control. Below is a summary of every available directive.

## Data Source Directives

| Directive | Description |
|-----------|-------------|
| [@http](./directives/http.md) | Resolve a field by calling a REST/HTTP endpoint. |
| [@grpc](./directives/grpc.md) | Resolve a field by calling a gRPC service method. |
| [@graphQL](./directives/graphQL.md) | Resolve a field by proxying to another GraphQL server. |
| [@call](./directives/call.md) | Compose multiple resolver steps into a single field resolution. |
| [@expr](./directives/expr.md) | Return a static value or a computed expression. |
| [@js](./directives/js.md) | Resolve a field using a custom JavaScript function. |

## Schema Transformation Directives

| Directive | Description |
|-----------|-------------|
| [@addField](./directives/addField.md) | Add a derived field to a type. |
| [@modify](./directives/modify.md) | Rename a field or mark it as omitted in the public schema. |
| [@omit](./directives/omit.md) | Exclude a field from the public-facing schema entirely. |
| [@discriminate](./directives/discriminate.md) | Set the discriminator field for union type resolution. |

## Performance and Security Directives

| Directive | Description |
|-----------|-------------|
| [@cache](./directives/cache.md) | Cache a field's resolved value for a specified duration. |
| [@protected](./directives/protected.md) | Restrict field access to authenticated users. |

## Endpoint Directives

| Directive | Description |
|-----------|-------------|
| [@rest](./directives/rest.md) | Expose a GraphQL query as a REST endpoint. |
