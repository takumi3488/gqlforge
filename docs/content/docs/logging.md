+++
title = "Logging"
description = "Configure logging output in GQLForge."
+++

# Logging

GQLForge uses a structured logging system built on Rust's tracing framework. You can control log verbosity and output format to suit your environment.

## Log Levels

Set the log level using the `RUST_LOG` environment variable:

```bash
RUST_LOG=info gqlforge start config.graphql
```

Available levels from most to least verbose:

| Level | Description |
|-------|-------------|
| `trace` | Very detailed internal events, useful for deep debugging |
| `debug` | Diagnostic information for development |
| `info` | General operational messages (default) |
| `warn` | Potential issues that do not prevent operation |
| `error` | Failures that affect request handling |

## Filtering by Module

You can set different levels for different modules:

```bash
RUST_LOG="gqlforge=debug,hyper=warn" gqlforge start config.graphql
```

This sets GQLForge's own logs to `debug` while keeping the HTTP library logs at `warn`.

## Production Recommendations

- Use `info` level in production to balance visibility and performance.
- Use `debug` or `trace` only during development or when investigating specific issues.
- Combine with the telemetry system for structured observability in production environments.

## Example

```bash
# Development: verbose output
RUST_LOG=debug gqlforge start config.graphql

# Production: standard output
RUST_LOG=info gqlforge start config.graphql

# Troubleshooting upstream calls
RUST_LOG="gqlforge=trace" gqlforge start config.graphql
```
