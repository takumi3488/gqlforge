import React from "react"

import Banner from "./Banner"
import Graph from "./Graph"
import Benefits from "./Benefits"
import Configuration from "./Configuration"
import Announcement from "../shared/Announcement"
import IntroductionVideo from "./IntroductionVideo"
const HomePage = (): JSX.Element => {
  return (
    <div className="">
      <Banner />
      <Configuration />
      <IntroductionVideo />
      <Benefits />
      <Graph />
    </div>
  )
}

export default HomePage
