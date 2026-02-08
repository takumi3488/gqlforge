[![Gqlforge Logo](./assets/logo_light.svg)](https://gqlforge.pages.dev)

Gqlforge is an open-source solution for building [high-performance] GraphQL backends.

Please support us by giving the repository a star
![image](./assets/star-our-repo.gif)

[high-performance]: https://github.com/takumi3488/graphql-benchmarks

[![Tweet](https://img.shields.io/twitter/url/http/shields.io.svg?style=for-the-badge&logo=x&color=black&labelColor=black)](https://twitter.com/intent/tweet?text=%40takumi3488%20is%20building%20a%20high-performance%20API%20Orchestration%20solution%20over%20%23GraphQL.%0A%0ACheck%20it%20out%20at%3A%0A%F0%9F%94%97%20https%3A%2F%2Fgqlforge.pages.dev%20%0A%F0%9F%94%97%20https%3A%2F%2Fgithub.com%2Ftakumi3488%2Fgqlforge%20%0A%0A&hashtags=api,http,rest,grpc,graphql,nocode,microservice,opensource)
[![Discord](https://img.shields.io/discord/1044859667798568962?style=for-the-badge&cacheSeconds=120&logo=discord)](https://discord.gg/kRZBPpkgwq)

[![Open Bounties](https://img.shields.io/endpoint?url=https%3A%2F%2Fconsole.algora.io%2Fapi%2Fshields%2Ftakumi3488%2Fbounties%3Fstatus%3Dopen&style=for-the-badge)](https://console.algora.io/org/takumi3488/bounties?status=open)
[![Rewarded Bounties](https://img.shields.io/endpoint?url=https%3A%2F%2Fconsole.algora.io%2Fapi%2Fshields%2Ftakumi3488%2Fbounties%3Fstatus%3Dcompleted&style=for-the-badge)](https://console.algora.io/org/takumi3488/bounties?status=completed)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/takumi3488/gqlforge/ci.yml?style=for-the-badge)](https://github.com/takumi3488/gqlforge/actions)
![GitHub release (by tag)](https://img.shields.io/github/downloads/takumi3488/gqlforge/total?style=for-the-badge)
[![Discord](https://img.shields.io/discord/1044859667798568962?style=for-the-badge&cacheSeconds=60)](https://discord.gg/kRZBPpkgwq)
[![Codecov](https://img.shields.io/codecov/c/github/takumi3488/gqlforge?style=for-the-badge)](https://app.codecov.io/gh/takumi3488/gqlforge)

## Installation

### NPM

```bash
npm i -g @gqlforge/gqlforge
```

### Yarn

```bash
yarn global add @gqlforge/gqlforge
```

### Home Brew

```bash
brew tap takumi3488/gqlforge
brew install gqlforge
```

### Curl

```bash
curl -sSL https://raw.githubusercontent.com/takumi3488/gqlforge/master/install.sh | bash
```

### Docker

```bash
docker pull ghcr.io/takumi3488/gqlforge/gqlforge
docker run -p 8080:8080 -p 8081:8081 ghcr.io/takumi3488/gqlforge/gqlforge
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

### Benchmarks

Throughput comparison of various GraphQL solutions for a N + 1 query:

```graphql
query {
  posts {
    title
    body
    user {
      name
    }
  }
}
```

![Throughput Histogram](https://raw.githubusercontent.com/takumi3488/graphql-benchmarks/main/assets/req_sec_histogram1.png)

Check out detailed benchmarks on our benchmarking [repository](https://github.com/takumi3488/graphql-benchmarks).

### Contributing

Your contributions are invaluable! Kindly go through our [contribution guidelines] if you are a first time contributor.

[contribution guidelines]: https://gqlforge.pages.dev/docs/contribution-guidelines

### Support Us

⭐️ Give us a star.

👀 Watch us for updates.

### License

This initiative is protected under the Apache 2.0 License.

<img referrerpolicy="no-referrer-when-downgrade" src="https://static.scarf.sh/a.png?x-pxid=82cc2ee2-ff41-4844-9ae6-c9face103e81" />
