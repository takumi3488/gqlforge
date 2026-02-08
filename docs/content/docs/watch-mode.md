+++
title = "Watch Mode"
description = "Automatically reload your GQLForge server when configuration changes."
+++

# Watch Mode

## Overview

During development, GQLForge can monitor your configuration files for changes and automatically restart the server when modifications are detected. This eliminates the need to manually stop and restart the server each time you edit your schema.

## How It Works

When watch mode is active, GQLForge observes the configuration files passed to the `start` command. If any of those files are modified, the server performs the following steps:

1. Detects the file change event.
2. Re-reads and validates the updated configuration.
3. Rebuilds the execution plan from the new schema.
4. Restarts the server with the updated configuration.

If the new configuration contains errors, GQLForge reports them in the terminal and continues running with the previous valid configuration.

## Usage

Watch mode can be enabled through the `@server` directive in your configuration:

```graphql
schema @server(port: 8000) {
  query: Query
}
```

When enabled, any saved change to your `.graphql` files triggers an automatic reload.

## Development Workflow

A typical development cycle with watch mode:

1. Start the server:
   ```bash
   gqlforge start ./app.graphql
   ```

2. Open the GraphQL Playground at `http://localhost:8000`.

3. Edit your `app.graphql` file in your editor. For example, add a new field:
   ```graphql
   type Query {
     posts: [Post] @http(path: "/posts")
     users: [User] @http(path: "/users") # newly added
   }
   ```

4. Save the file. The server reloads automatically.

5. Switch to the Playground and run your new query immediately.

## Linked Files

If your configuration uses the `@link` directive to import other files, changes to those linked files also trigger a reload:

```graphql
schema @server(port: 8000) @link(type: Config, src: "./users.graphql") {
  query: Query
}
```

Editing `users.graphql` causes the server to reload, just as editing the main file would.

## Notes

- Watch mode is intended for local development. For production deployments, use a standard `gqlforge start` without file watching.
- Only files referenced by the configuration (directly or through `@link`) are monitored.
- The reload preserves the server port, so no client-side changes are needed after a restart.
