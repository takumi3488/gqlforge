+++
title = "Integration Testing"
description = "Writing integration tests for GQLForge."
+++

# Integration Testing

GQLForge's integration test suite is built around the `execution_spec` framework. These tests verify end-to-end behavior by defining a complete GraphQL configuration, executing queries, and validating responses.

## How Execution Specs Work

Each spec file defines:

- **A GraphQL schema** with directives (`@http`, `@grpc`, `@js`, etc.)
- **Mock upstream responses** for HTTP or gRPC services
- **Input queries** to execute against the schema
- **Expected output** that the server should return

The test runner reads the spec, starts a test server with the defined configuration, executes the queries, and compares results against the expected output.

## Running Execution Specs

```bash
cargo test --test execution_spec
```

To run a specific spec by name:

```bash
cargo test --test execution_spec spec_name
```

## Writing a New Spec

1. Create a new spec file in the appropriate test directory.
2. Define your schema with the relevant directives.
3. Provide mock responses that the test server should return for upstream calls.
4. Specify the GraphQL query and expected response.
5. Run the test to verify it passes.

## Snapshot Integration

Execution specs use insta snapshots for output validation. When you add or modify a spec:

1. Run the test: `cargo test --test execution_spec`
2. If a new snapshot is created, review it: `cargo insta review`
3. Accept if the output matches your expectations.

## Tips

- Keep specs focused on a single feature or behavior.
- Use descriptive spec names that explain what is being tested.
- When debugging a failing spec, run it in isolation with `cargo test --test execution_spec -- spec_name` to see detailed output.
