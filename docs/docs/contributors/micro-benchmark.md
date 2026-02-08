---
title: "Micro Benchmarks"
description: "Running micro benchmarks for GQLForge."
sidebar_label: "Micro Benchmarks"
---

# Micro Benchmarks

GQLForge includes micro benchmarks to measure the performance of critical code paths.

## Running Benchmarks

Execute all benchmarks:

```bash
cargo bench
```

Run a specific benchmark by name:

```bash
cargo bench -- benchmark_name
```

## What Gets Benchmarked

Micro benchmarks cover areas such as:

- **Query parsing**: Time to parse GraphQL query strings into an AST.
- **Schema validation**: Time to validate a configuration and resolve types.
- **Response serialization**: Time to serialize resolved data into JSON.
- **Template rendering**: Time to render Mustache templates for upstream URLs.

## Interpreting Results

Cargo bench outputs timing statistics for each benchmark:

```
test parse_query ... bench:     1,234 ns/iter (+/- 56)
```

The value shows the average time per iteration and the variance. Lower is better.

## Comparing Before and After

To measure the impact of a change:

1. Run benchmarks on the `main` branch and save the output.
2. Switch to your feature branch and run benchmarks again.
3. Compare the numbers to check for regressions or improvements.

For automated comparison, consider using [critcmp](https://github.com/BurntSushi/critcmp) which formats benchmark diffs clearly.

## Writing New Benchmarks

When adding a new performance-sensitive feature:

1. Create a benchmark in the `benches/` directory.
2. Use `criterion` or the built-in `#[bench]` attribute.
3. Focus on isolating the specific operation you want to measure.
4. Avoid I/O or network calls within the benchmark loop.
