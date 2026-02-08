import React from "react"
import CodeBlock from "@theme/CodeBlock"
import Section from "../shared/Section"
import SectionTitle from "../shared/SectionTitle"

const exampleConfig = `schema
  @server(port: 8000)
  @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  users: [User] @http(path: "/users")
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
  email: String!
  posts: [Post] @http(path: "/users/{{.value.id}}/posts")
}

type Post {
  id: Int!
  title: String!
  body: String!
}`

const Configuration = (): JSX.Element => {
  return (
    <Section className="bg-gqlForge-light-200">
      <div className="flex flex-col gap-8">
        <SectionTitle title="Quick Start" />
        <h2 className="text-title-large sm:text-display-tiny font-space-grotesk mb-0">
          Define your schema, GQLForge does the rest
        </h2>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-start">
          <div className="flex flex-col gap-4">
            <p className="text-content-small sm:text-content-medium text-gqlForge-dark-100">
              Write a GraphQL schema with directives that describe how to fetch data. GQLForge compiles it into an
              optimized runtime that handles batching, caching, and protocol translation automatically.
            </p>
            <div className="flex flex-col gap-3">
              <div className="flex items-start gap-3">
                <span className="font-bold text-title-tiny font-space-grotesk shrink-0">1.</span>
                <div>
                  <p className="font-bold text-content-small mb-1">Install GQLForge</p>
                  <code className="text-content-tiny">npm i -g @gqlforge/gqlforge</code>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <span className="font-bold text-title-tiny font-space-grotesk shrink-0">2.</span>
                <div>
                  <p className="font-bold text-content-small mb-1">Create a config file</p>
                  <code className="text-content-tiny">Save the schema as app.graphql</code>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <span className="font-bold text-title-tiny font-space-grotesk shrink-0">3.</span>
                <div>
                  <p className="font-bold text-content-small mb-1">Start the server</p>
                  <code className="text-content-tiny">gqlforge start app.graphql</code>
                </div>
              </div>
            </div>
          </div>
          <div>
            <CodeBlock language="graphql" title="app.graphql">
              {exampleConfig}
            </CodeBlock>
          </div>
        </div>
      </div>
    </Section>
  )
}

export default Configuration
