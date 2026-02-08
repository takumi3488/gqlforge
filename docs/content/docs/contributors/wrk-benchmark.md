+++
title = "Load Testing with wrk"
description = "Running load tests against GQLForge."
+++

# Load Testing with wrk

Use [wrk](https://github.com/wg/wrk) to measure GQLForge throughput and latency under load.

## Prerequisites

Install wrk:

```bash
# macOS
brew install wrk

# Ubuntu/Debian
sudo apt-get install wrk
```

## Basic Usage

Start your GQLForge server:

```bash
gqlforge start config.graphql
```

In another terminal, run wrk against the GraphQL endpoint:

```bash
wrk -t4 -c100 -d30s -s query.lua http://localhost:8000/graphql
```

| Flag | Description |
|------|-------------|
| `-t4` | Use 4 threads |
| `-c100` | Maintain 100 open connections |
| `-d30s` | Run for 30 seconds |
| `-s` | Lua script for custom requests |

## Lua Script for GraphQL

Create a `query.lua` file to send POST requests with a GraphQL query:

```lua
wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"
wrk.body = '{"query":"{ users { id name email } }"}'
```

## Interpreting Results

wrk outputs a summary like:

```
Requests/sec:  12345.67
Transfer/sec:      5.43MB
Latency (avg):     8.12ms
```

Key metrics to watch:

- **Requests/sec**: Overall throughput. Higher is better.
- **Latency**: Average and percentile response times. Lower is better.
- **Errors**: Non-2xx responses indicate issues under load.

## Testing Different Scenarios

Test with varying levels of concurrency to find your server's limits:

```bash
# Light load
wrk -t2 -c10 -d15s -s query.lua http://localhost:8000/graphql

# Heavy load
wrk -t8 -c500 -d30s -s query.lua http://localhost:8000/graphql
```

## Tips

- Run the server in release mode (`cargo build --release`) for accurate production-like numbers.
- Close other resource-intensive applications during the test.
- Run multiple iterations to ensure consistent results.
- Test with realistic query complexity that matches your production workload.
