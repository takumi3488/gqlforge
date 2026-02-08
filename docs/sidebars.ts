import {SidebarsConfig} from "@docusaurus/plugin-content-docs"

const sidebars: SidebarsConfig = {
  docs: [
    {
      type: "category",
      label: "Getting Started",
      collapsed: false,
      items: ["getting-started", "cli", "conventions", "playground"],
    },
    {
      type: "category",
      label: "Configuration",
      collapsed: false,
      items: ["runtime-config", "config/server", "config/upstream", "config/links", "config/telemetry"],
    },
    {
      type: "category",
      label: "Resolver Directives",
      collapsed: false,
      items: [
        "directives",
        "directives/http",
        "directives/grpc",
        "directives/graphQL",
        "directives/call",
        "directives/expr",
        "directives/js",
      ],
    },
    {
      type: "category",
      label: "Schema Directives",
      collapsed: false,
      items: [
        "directives/addField",
        "directives/modify",
        "directives/omit",
        "directives/discriminate",
        "directives/rest",
        "directives/cache",
        "directives/protected",
      ],
    },
    {
      type: "category",
      label: "Features",
      collapsed: false,
      items: [
        "N+1",
        "auth",
        "http-cache",
        "http2",
        "logging",
        "telemetry",
        "scalar",
        "config-generation",
        "watch-mode",
        "execution-strategy",
        "context",
        "environment-variables",
        "grpc",
        "scripting",
        "rest",
        "apollo-federation-subgraph",
      ],
    },
    {
      type: "category",
      label: "Integrations",
      collapsed: false,
      items: ["apollo-studio", "data-dog", "new-relic", "honey-comb"],
    },
    {
      type: "category",
      label: "Deployment",
      collapsed: false,
      items: ["gqlforge-on-fly", "gqlforge-on-aws", "gh-action", "client-tuning"],
    },
    {
      type: "category",
      label: "Contributing",
      collapsed: false,
      items: [
        "contributors/guidelines",
        "contributors/testing",
        "contributors/integration-testing",
        "contributors/telemetry",
        "contributors/micro-benchmark",
        "contributors/wrk-benchmark",
      ],
    },
  ],
}

module.exports = sidebars
