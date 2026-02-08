import React from "react"
import {features} from "@site/src/constants"
import Section from "../shared/Section"
import SectionTitle from "../shared/SectionTitle"

const Features = (): JSX.Element => {
  return (
    <Section>
      <div className="flex flex-col gap-8">
        <SectionTitle title="Features" />
        <h2 className="text-title-large sm:text-display-tiny font-space-grotesk mb-0">
          Everything you need for production GraphQL
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, idx) => (
            <div
              key={idx}
              className="flex flex-col gap-3 p-6 rounded-2xl border border-gqlForge-border-light-300 hover:shadow-md transition-shadow"
            >
              <span className="text-3xl">{feature.icon}</span>
              <h3 className="text-title-small font-space-grotesk mb-0">{feature.title}</h3>
              <p className="text-content-small text-gqlForge-dark-100 mb-0">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>
    </Section>
  )
}

export default Features
