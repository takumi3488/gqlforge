[![Gqlforge Logo](./assets/logo_light.svg)](https://gqlforge.pages.dev)

Gqlforge is an open-source solution for building high-performance GraphQL backends.

## Installation

### Cargo

```bash
cargo install --git https://github.com/takumi3488/gqlforge
```

### Docker

```bash
docker pull ghcr.io/takumi3488/gqlforge/gqlforge
docker run -p 8000:8000 -p 8081:8081 ghcr.io/takumi3488/gqlforge/gqlforge
```

## Get Started

The below file is a standard `.graphQL` file, with a few additions such as `@server` and `@http` directives. So, basically, we specify the GraphQL schema and how to resolve that GraphQL schema in the same file, without having to write any code!

```graphql
schema @server(port: 8000, hostname: "0.0.0.0") @upstream(httpCache: 42) {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}
```

Now, run the following command to start the server with the full path to the jsonplaceholder.graphql file that you created above.

```bash
gqlforge start ./jsonplaceholder.graphql
```

Head out to [docs] to learn about other powerful gqlforge features.

[docs]: https://gqlforge.pages.dev/docs

### Contributing

Your contributions are invaluable! Kindly go through our [contribution guidelines] if you are a first time contributor.

[contribution guidelines]: https://gqlforge.pages.dev/docs/contribution-guidelines

### Support Us

⭐️ Give us a star.

👀 Watch us for updates.

### License

This initiative is protected under the Apache 2.0 License.

<img referrerpolicy="no-referrer-when-downgrade" src="https://static.scarf.sh/a.png?x-pxid=82cc2ee2-ff41-4844-9ae6-c9face103e81" />
