---
title: "Deploy on Fly.io"
description: "Deploy GQLForge to Fly.io."
sidebar_label: "Fly.io"
---

# Deploy on Fly.io

This guide walks through deploying a GQLForge server to Fly.io using Docker.

## Dockerfile

Create a `Dockerfile` in your project root:

```dockerfile
FROM ghcr.io/gqlforge/gqlforge:latest

COPY config.graphql /app/config.graphql

EXPOSE 8000

CMD ["gqlforge", "start", "/app/config.graphql"]
```

## Fly Configuration

Create a `fly.toml` file:

```toml
app = "my-gqlforge-api"
primary_region = "nrt"

[http_service]
  internal_port = 8000
  force_https = true

[[http_service.checks]]
  interval = "10s"
  timeout = "2s"
  method = "GET"
  path = "/health"
```

## Deployment Steps

1. Install the Fly CLI and authenticate:

```bash
flyctl auth login
```

2. Create the app:

```bash
flyctl apps create my-gqlforge-api
```

3. Deploy:

```bash
flyctl deploy
```

4. Verify the deployment:

```bash
flyctl status
```

## Setting Environment Variables

If your configuration references environment variables, set them as secrets:

```bash
flyctl secrets set API_TOKEN="your-token-here"
```

## Scaling

Scale your deployment by adjusting the number of machines:

```bash
flyctl scale count 3
```

Fly.io distributes traffic across instances automatically.
