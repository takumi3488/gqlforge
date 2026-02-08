import React from "react"
import {pageLinks} from "./routes"

export const githubRepoURL = "https://github.com/takumi3488/gqlforge"

export const features: Feature[] = [
  {
    title: "Declarative API Orchestration",
    description:
      "Define your GraphQL schema using directives to connect REST, gRPC, and GraphQL backends. No boilerplate resolver code required.",
    icon: "🔗",
  },
  {
    title: "High Performance",
    description:
      "Built in Rust with ahead-of-time query optimization. The JIT engine analyzes data dependencies and generates efficient execution plans.",
    icon: "⚡",
  },
  {
    title: "N+1 Detection",
    description:
      "Automatically detects N+1 query patterns at compile time and provides batching primitives to eliminate redundant API calls.",
    icon: "🔍",
  },
  {
    title: "Protocol Agnostic",
    description:
      "Seamlessly integrate REST APIs via @http, gRPC services via @grpc, and other GraphQL endpoints via @graphQL in a single unified schema.",
    icon: "🌐",
  },
  {
    title: "Extensible with JavaScript",
    description:
      "When built-in directives are not enough, use the embedded JavaScript runtime via @js to implement custom resolver logic.",
    icon: "📜",
  },
  {
    title: "Single Binary, Zero Dependencies",
    description:
      "Ships as one executable with no external dependencies. Install and start serving GraphQL in seconds.",
    icon: "📦",
  },
]

export const socials: Social[] = [
  {
    id: 1,
    name: "github",
    image: require("@site/static/icons/companies/github-footer.svg").default,
    href: githubRepoURL,
  },
]

export const footerItems: FooterItem[] = [
  {
    title: "Docs",
    items: [
      {name: "Getting Started", link: "/docs/getting-started"},
      {name: "CLI Reference", link: "/docs/cli"},
      {name: "Directives", link: "/docs/directives"},
    ],
  },
  {
    title: "Community",
    items: [
      {name: "GitHub", link: githubRepoURL},
      {name: "Contributing", link: "/docs/contributors/guidelines"},
    ],
  },
]

export enum Theme {
  Light = "light",
  Dark = "dark",
  Gray = "gray",
  GQLForge = "gqlforge",
}
