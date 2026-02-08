import React from "react"

import Banner from "./Banner"
import Configuration from "./Configuration"
import Features from "./Features"
import Benefits from "./Benefits"

const HomePage = (): JSX.Element => {
  return (
    <div>
      <Banner />
      <Configuration />
      <Features />
      <Benefits />
    </div>
  )
}

export default HomePage
