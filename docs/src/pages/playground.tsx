import React from "react"
import Layout from "@theme/Layout"
import PlaygroundPage from "../components/playground"
import {PageDescription, PageTitle} from "../constants/titles"

const Playground = () => {
  return (
    <Layout title={PageTitle.PLAYGROUND} description={PageDescription.PLAYGROUND}>
      <PlaygroundPage />
    </Layout>
  )
}

export default Playground
