import React from "react"
import Link from "@docusaurus/Link"
import Section from "../shared/Section"
import SectionTitle from "../shared/SectionTitle"

const benefits = [
  {
    title: "Declarative over Imperative",
    description:
      "Focus on what data you need, not how to fetch it. GQLForge's directive-based approach eliminates boilerplate and keeps your API definition readable.",
  },
  {
    title: "Compile-Time Safety",
    description:
      "Schema validation catches misconfigurations before deployment. Type mismatches, missing fields, and N+1 patterns are reported during the build step.",
  },
  {
    title: "Production Ready",
    description:
      "Built-in support for HTTP caching, authentication, telemetry (OpenTelemetry), and health checks. Deploy with confidence on Docker, AWS, or Fly.io.",
  },
]

const Benefits = (): JSX.Element => {
  return (
    <div className="bg-[#1C1D1F] grid-background pb-20 lg:pb-40">
      <Section className="!pb-0 lg:pt-24">
        <div className="flex flex-col gap-8">
          <SectionTitle title="Why GQLForge" />
          <h2 className="text-title-large sm:text-display-tiny font-space-grotesk text-white mb-0">
            A better way to build GraphQL backends
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {benefits.map((benefit, idx) => (
              <div key={idx} className="flex flex-col gap-3 p-6 rounded-2xl border border-gqlForge-border-dark-300">
                <h3 className="text-title-small font-space-grotesk text-white mb-0">{benefit.title}</h3>
                <p className="text-content-small text-gqlForge-light-500 mb-0">{benefit.description}</p>
              </div>
            ))}
          </div>
          <div className="flex justify-center mt-4">
            <Link
              to="/docs/getting-started"
              className="bg-gqlForge-yellow text-gqlForge-dark-500 hover:text-gqlForge-dark-500 hover:no-underline font-bold px-8 py-3 rounded-xl text-title-small"
            >
              Read the Docs
            </Link>
          </div>
        </div>
      </Section>
    </div>
  )
}

export default Benefits
