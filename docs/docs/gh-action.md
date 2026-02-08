---
title: "GitHub Actions"
description: "Use GQLForge in GitHub Actions CI/CD."
sidebar_label: "GitHub Actions"
---

# GitHub Actions

Validate your GQLForge configuration in CI to catch errors before deployment.

## Workflow Example

Create `.github/workflows/gqlforge-check.yml`:

```yaml
name: GQLForge Check

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install GQLForge
        run: |
          curl -sSL https://github.com/gqlforge/gqlforge/releases/latest/download/gqlforge-linux-amd64 -o gqlforge
          chmod +x gqlforge
          sudo mv gqlforge /usr/local/bin/

      - name: Validate configuration
        run: gqlforge check config.graphql

      - name: Check for N+1 queries
        run: gqlforge check --n-plus-one-queries config.graphql
```

## What Gets Checked

The `gqlforge check` command validates:

- Schema syntax and type correctness
- Directive arguments and usage
- Upstream URL reachability (optional)
- Linked file references (proto files, scripts, htpasswd)

Adding `--n-plus-one-queries` additionally reports any query paths that could trigger N+1 request patterns.

## Caching the Binary

Speed up subsequent runs by caching the GQLForge binary:

```yaml
- name: Cache GQLForge binary
  uses: actions/cache@v4
  with:
    path: /usr/local/bin/gqlforge
    key: gqlforge-${{ runner.os }}-latest
```

## Using with Multiple Config Files

If your project has several configuration files, validate them all:

```yaml
- name: Validate all configs
  run: |
    for file in configs/*.graphql; do
      echo "Checking $file"
      gqlforge check "$file"
    done
```
