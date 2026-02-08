import React from "react"
import {pageLinks} from "./routes"

export const githubRepoURL = "https://github.com/takumi3488/gqlforge"
export const gqlForgeBenchmarkUrl = "https://github.com/takumi3488/graphql-benchmarks#benchmark-results"

const Highlight = ({text}: {text: string}) => (
  <>
    <span className="text-content-tiny font-bold sm:text-title-tiny lg:text-title-small bg-gqlForge-yellow rounded-[4px] sm:rounded-md px-SPACE_01">
      {text}
    </span>
  </>
)

export const features: Feature[] = [
  {
    logo: require("@site/static/images/home/orchestration.png").default,
    title: "Orchestration",
    content: (
      <>
        GQLForge provides first-class primitives to perform API Orchestration across protocols such as{" "}
        <Highlight text="gRPC, REST, GraphQL," />. This allow developers to enrich existing APIs with more data, perform
        transformations or build a completely new set of aggregation APIs.
      </>
    ),
    alt: "Orchestration",
  },
  {
    logo: require("@site/static/images/home/governance.png").default,
    title: "Governance",
    content: (
      <>
        With GQLForge, your focus shifts to the 'what'—such as entities, their relationships, access control, security,
        authentication, caching, and more—rather than the 'how'. This shift is enabled by the GQLForge DSL, embodying a
        true <Highlight text="declarative approach" /> to managing APIs.
      </>
    ),
    alt: "Governance",
  },
  {
    logo: require("@site/static/images/home/efficiency.png").default,
    title: "Efficiency",
    content: (
      <>
        GQLForge can introspect all orchestration requirements <Highlight text="ahead of time" /> and automatically
        generate a highly efficient data access layer. This results in achieving much lower resource utilization and
        opens up opportunities to use in ultra-low latency workloads.
      </>
    ),
    alt: "Efficiency",
  },
  {
    logo: require("@site/static/images/home/extendability.png").default,
    title: "Extendability",
    content: (
      <>
        At times, the built-in primitives may not fully satisfy specific orchestration needs. In such instances,
        GQLForge offers a lightweight embedded <Highlight text="JavaScript" /> runtime. This feature enables you to
        attach custom hooks for monitoring events within GQLForge, allowing you to directly issue commands for the
        subsequent actions GQLForge should execute.
      </>
    ),
    alt: "Extendability",
  },
]

export const moreFeatures: MoreFeatures[] = [
  {
    id: 1,
    title: "Ahead of Time Optimizations",
    logo: require("@site/static/icons/basic/rocket-icon.svg").default,
  },
  {
    id: 2,
    title: "Composable Orchestration Primitives",
    logo: require("@site/static/icons/basic/extension.svg").default,
  },
  {
    id: 3,
    title: "Macro Resiliency Capabilities",
    logo: require("@site/static/icons/basic/shield.svg").default,
  },
  {
    id: 4,
    title: "Protocol agnostic",
    logo: require("@site/static/icons/basic/check-done.svg").default,
  },
  {
    id: 5,
    title: "High Performance",
    logo: require("@site/static/icons/basic/line-chart-up.svg").default,
  },
  {
    id: 6,
    title: "Security",
    logo: require("@site/static/icons/basic/security.svg").default,
  },
  {
    id: 7,
    title: "Edge Compatible",
    logo: require("@site/static/icons/basic/puzzle.svg").default,
  },
  {
    id: 8,
    title: "Compile time Checks",
    logo: require("@site/static/icons/basic/clock.svg").default,
  },
  {
    id: 9,
    logo: require("@site/static/icons/basic/adaptive.svg").default,
    title: "Adaptive performance improvements",
  },
  {
    id: 11,
    logo: require("@site/static/icons/basic/insight.svg").default,
    title: "Telemetry",
  },
  {
    id: 12,
    logo: require("@site/static/icons/basic/connect.svg").default,
    title: "Scripting Flexibility",
  },
]

export const socials: Social[] = [
  {
    id: 1,
    name: "github",
    image: require("@site/static/icons/companies/github-footer.svg").default,
    href: "https://github.com/takumi3488/gqlforge",
  },
]

export const chooseGQLForge: ChooseGQLForge[] = [
  {
    id: 1,
    title: "Top developer experience",
    description: "Design your APIs, with syntax highlighting and lint checks within your favourite IDE.",
    image: require("@site/static/images/home/dev-experience.png").default,
  },
  {
    id: 2,
    title: "Performance",
    description: "Get performance that's higher than your hand optimized implementation",
    image: require("@site/static/images/home/performance.png").default,
  },
  {
    id: 3,
    title: "Scale Fearlessly",
    description: "Leverage built-in best practices that guarantee robustness at any scale.",
    image: require("@site/static/images/home/scale.png").default,
  },
]

export const gqlforgeFeatures: GQLForgeFeatures[] = [
  {
    id: 1,
    title: "Powerful Batching Primitive",
    image: require("@site/static/images/choose-gqlforge/rocket.png").default,
    redirection_url: "/docs/graphql-n-plus-one-problem-solved-gqlforge/#using-batch-apis",
  },
  {
    id: 2,
    title: "Extensions with plugins and JS support",
    image: require("@site/static/images/choose-gqlforge/grid.png").default,
    redirection_url: "/docs/graphql-javascript-customization/",
  },
  {
    id: 3,
    title: "Field based Authentication & Authorisation",
    image: require("@site/static/images/choose-gqlforge/shield-tick.png").default,
    redirection_url: "/docs/field-level-access-control-graphql-authentication/",
  },
  {
    id: 4,
    title: "Protocol agnostic",
    image: require("@site/static/images/choose-gqlforge/check-done.png").default,
    redirection_url: "/docs/graphql-grpc-gqlforge/",
  },
  {
    id: 5,
    title: "Performance",
    image: require("@site/static/images/choose-gqlforge/line-chart-up.png").default,
    redirection_url: "https://github.com/takumi3488/graphql-benchmarks",
  },
  {
    id: 6,
    title: "Security",
    image: require("@site/static/images/choose-gqlforge/lock.png").default,
    redirection_url: "/docs/field-level-access-control-graphql-authentication/",
  },
  {
    id: 7,
    title: "Edge Compatible",
    image: require("@site/static/images/choose-gqlforge/puzzle-piece.png").default,
    redirection_url: "/docs/deploy-graphql-github-actions/",
  },
  {
    id: 8,
    title: "Compile time checks",
    image: require("@site/static/images/choose-gqlforge/clock-stopwatch.png").default,
    redirection_url: "/docs/gqlforge-graphql-cli/#check",
  },
]

export const benefits: Benefits[] = [
  {
    id: 1,
    title: "Secure",
    description:
      "GQLForge has been validated against a comprehensive database of GraphQL vulnerabilities. Rest easy knowing your GraphQL backends are secure.",
    image: require("@site/static/images/home/secure-icon.png").default,
    redirection_url: "/docs/field-level-access-control-graphql-authentication/",
  },
  {
    id: 2,
    title: "High-Performance",
    description:
      "GQLForge performs ahead-of-time optimizations based on analysis of the schema and data dependencies. Deploy GraphQL without compromises.",
    image: require("@site/static/images/home/performance.png").default,
    redirection_url: "https://github.com/takumi3488/graphql-benchmarks",
  },
  {
    id: 3,
    title: "Statically Verified",
    description:
      "GQLForge statically verifies that GraphQL schemas match resolvers and warns about N + 1 issues. Deploy new APIs with confidence.",
    image: require("@site/static/images/home/statically-verified-icon.png").default,
    redirection_url: "/docs/graphql-n-plus-one-problem-solved-gqlforge/",
  },
  {
    id: 4,
    title: "Simple",
    description:
      "GQLForge configuration generator can integrate thousands of APIs in a matter of minutes. Configure with ease and deploy with confidence.",
    image: require("@site/static/images/home/simple-icon.png").default,
    redirection_url: "/docs/gqlforge-dsl-graphql-custom-directives/",
  },
  {
    id: 5,
    title: "Customizable",
    description:
      "Write custom Javascript to customize any aspect of your GraphQL backend. Leverage this escape hatch to satisfy any requirement.",
    image: require("@site/static/images/home/customizable-icon.png").default,
    redirection_url: "/docs/graphql-javascript-customization/",
  },
  {
    id: 6,
    title: "Plug & Play",
    description:
      "Engineered to stay out of your way, shipping as a single executable with no dependencies or requirements. Get started quickly and easily.",
    image: require("@site/static/images/home/plug-play-icon.png").default,
    redirection_url: "/docs/",
  },
  {
    id: 7,
    title: "Open Source",
    description:
      "GQLForge is developed and released under the Apache 2 open source license, the gold standard for OSS. Embrace a vendor-neutral solution.",
    image: require("@site/static/images/home/open-source-icon.png").default,
    redirection_url: "https://github.com/takumi3488/gqlforge",
  },
]

// Define an enum for theme options
export enum Theme {
  Light = "light",
  Dark = "dark",
  Gray = "gray",
  GQLForge = "gqlforge",
}

export const footerItems: FooterItem[] = [
  {
    title: "Developers",
    items: [
      {
        name: "Docs",
        link: pageLinks.docs,
      },
      {
        name: "Learn",
        link: pageLinks.introduction,
      },
    ],
  },
]
