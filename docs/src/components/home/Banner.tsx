import React from "react"
import Link from "@docusaurus/Link"
import Section from "../shared/Section"

const Banner = (): JSX.Element => {
  return (
    <main className="grid justify-center">
      <Section className="flex flex-col sm:items-center sm:text-center w-full !pb-0 pt-16 lg:pt-24">
        <div className="h-full 2xl:min-h-0">
          <h1 className="text-title-large max-w-xs sm:text-display-small lg:text-display-large sm:max-w-5xl font-space-grotesk">
            Build GraphQL APIs
            <br />
            <span className="bg-gqlForge-yellow rounded-md sm:rounded-2xl px-SPACE_02">without writing resolvers</span>
          </h1>
          <p className="sm:max-w-2xl sm:m-auto text-content-small sm:text-content-medium lg:text-content-large font-normal max-w-md sm:mt-SPACE_04 mb-0">
            GQLForge is a high-performance GraphQL runtime built in Rust. Compose REST, gRPC, and GraphQL services into a
            unified API layer using a declarative configuration.
          </p>
          <div className="flex flex-col sm:flex-row justify-center mt-SPACE_06 sm:mt-SPACE_10 gap-SPACE_04 sm:gap-SPACE_06">
            <Link
              to="/docs/getting-started"
              className="flex items-center justify-center bg-gqlForge-dark-500 text-white hover:text-white hover:no-underline font-bold px-8 py-3 rounded-xl text-title-small h-12 sm:h-16"
            >
              Get Started
            </Link>
            <Link
              to="https://github.com/takumi3488/gqlforge"
              className="flex items-center justify-center border-2 border-gqlForge-dark-500 text-gqlForge-dark-500 hover:text-gqlForge-dark-500 hover:no-underline font-bold px-8 py-3 rounded-xl text-title-small h-12 sm:h-16"
            >
              View on GitHub
            </Link>
          </div>
        </div>
      </Section>
    </main>
  )
}

export default Banner
