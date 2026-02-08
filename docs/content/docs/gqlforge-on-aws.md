+++
title = "Deploy on AWS"
description = "Deploy GQLForge to AWS."
+++

# Deploy on AWS

This guide covers deploying GQLForge to AWS using ECS with Fargate.

## Docker Image

Use the official GQLForge Docker image or build your own:

```dockerfile
FROM ghcr.io/takumi3488/gqlforge/gqlforge:latest

COPY config.graphql /app/config.graphql

EXPOSE 8000

CMD ["gqlforge", "start", "/app/config.graphql"]
```

Push the image to Amazon ECR:

```bash
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin <account-id>.dkr.ecr.us-east-1.amazonaws.com
docker build -t gqlforge-api .
docker tag gqlforge-api:latest <account-id>.dkr.ecr.us-east-1.amazonaws.com/gqlforge-api:latest
docker push <account-id>.dkr.ecr.us-east-1.amazonaws.com/gqlforge-api:latest
```

## ECS Task Definition

Create a task definition (`task-definition.json`):

```json
{
  "family": "gqlforge-api",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "containerDefinitions": [
    {
      "name": "gqlforge",
      "image": "<account-id>.dkr.ecr.us-east-1.amazonaws.com/gqlforge-api:latest",
      "portMappings": [
        { "containerPort": 8000, "protocol": "tcp" }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/gqlforge-api",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

## Deployment Steps

1. Register the task definition:

```bash
aws ecs register-task-definition --cli-input-json file://task-definition.json
```

2. Create an ECS service with an Application Load Balancer:

```bash
aws ecs create-service \
  --cluster my-cluster \
  --service-name gqlforge-api \
  --task-definition gqlforge-api \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-xxx],securityGroups=[sg-xxx],assignPublicIp=ENABLED}"
```

3. Configure the ALB to forward traffic on port 80 to the container on port 8000.

## Environment Variables

Pass secrets through ECS task environment variables or AWS Secrets Manager for sensitive configuration values.
