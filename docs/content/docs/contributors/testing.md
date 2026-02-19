+++
title = "Testing"
description = "How to run tests in the GQLForge project."
+++

# Testing

GQLForge uses Rust's built-in test framework along with snapshot testing via the `insta` crate.

## Running All Tests

```bash
cargo test
```

This runs the full test suite including unit tests, integration tests, and snapshot tests.

## Running Specific Tests

Run tests for the main crate only:

```bash
cargo test -p gqlforge
```

Run a specific test by name:

```bash
cargo test test_name_here
```

## Execution Spec Tests

The primary integration test suite is `execution_spec`:

```bash
cargo test --test execution_spec
```

These tests define GraphQL configurations and expected outputs as specification files. Each spec describes a schema, an input query, and the expected response.

## Snapshot Testing with Insta

GQLForge uses [insta](https://insta.rs) for snapshot testing. When a test produces output that differs from the stored snapshot, the test fails.

### Reviewing Snapshot Changes

After making changes that intentionally alter output, review and accept the new snapshots:

```bash
cargo insta review
```

This opens an interactive prompt showing the diff between the old and new snapshot. Accept or reject each change.

### Updating All Snapshots

To accept all pending snapshot changes at once:

```bash
cargo insta accept
```

## PostgreSQL Integration Tests

To run PostgreSQL-specific integration tests:

```bash
cargo test --test postgres_execution_spec
```

These tests require a running PostgreSQL instance. See the test fixtures for connection details.

## Writing New Tests

When adding a new feature:

1. Write unit tests alongside your implementation code using `#[cfg(test)]` modules.
2. Add execution spec tests if the feature affects GraphQL query resolution.
3. Run `cargo test` to verify everything passes.
4. If snapshots change, review them with `cargo insta review` to confirm the changes are expected.
