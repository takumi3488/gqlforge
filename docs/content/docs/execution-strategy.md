+++
title = "Execution Strategy"
description = "How GQLForge optimizes query execution with its JIT engine."
+++

# Execution Strategy

## Overview

GQLForge uses an ahead-of-time optimization approach to minimize runtime overhead when executing GraphQL queries. Rather than interpreting the schema and resolvers on every request, the engine analyzes the entire configuration at startup and produces an optimized execution plan.

## Ahead-of-Time Analysis

When the server starts, GQLForge performs a full analysis of your schema and resolver configurations. This phase includes:

- **Dependency graph construction**: Determines the relationships between types, fields, and their resolvers.
- **Resolver ordering**: Identifies which resolvers depend on parent values and which can run independently.
- **N+1 detection**: Flags potential N+1 query patterns where a field resolver inside a list would trigger one request per item.

This upfront analysis means the server avoids repeated work during request handling.

## JIT Execution Engine

GQLForge compiles resolver chains into an internal execution plan at startup rather than interpreting them dynamically. This JIT-style approach provides several benefits:

- **Reduced per-request overhead**: No schema introspection or resolver lookup happens at query time.
- **Predictable performance**: Execution paths are determined before the first request arrives.
- **Early error detection**: Configuration issues are caught at startup, not at runtime.

## Parallel Execution

Independent resolvers within a query are executed concurrently. When the execution plan identifies fields that have no data dependencies on each other, their resolvers run in parallel.

For example, given this query:

```graphql
{
  users {
    name
  }
  posts {
    title
  }
}
```

The resolvers for `users` and `posts` execute simultaneously since neither depends on the other's result.

## Data Loader Batching

When a resolver appears inside a list type, GQLForge automatically batches the requests. Instead of sending one HTTP call per list item, the engine collects all required IDs and issues a single batched request where possible.

Consider this schema:

```graphql
type Post {
  id: Int!
  userId: Int!
  user: User @http(path: "/users/{{.value.userId}}")
}
```

When resolving `user` for a list of 10 posts, GQLForge groups the lookups and reduces the number of outbound HTTP calls. This eliminates the classic N+1 problem without requiring manual batching logic in your configuration.

## Request Deduplication

If multiple fields within the same query resolve to the same upstream URL with identical parameters, GQLForge deduplicates those calls. Only one HTTP request is sent, and the result is shared across all fields that need it.

## Summary

| Optimization           | Description                                                 |
| ---------------------- | ----------------------------------------------------------- |
| Ahead-of-time analysis | Schema and resolvers are analyzed once at startup           |
| JIT execution plan     | Resolver chains are compiled into optimized execution paths |
| Parallel execution     | Independent resolvers run concurrently                      |
| Data loader batching   | List-nested resolvers are batched into fewer upstream calls |
| Request deduplication  | Identical upstream requests are merged into one             |
