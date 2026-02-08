import React from "react"
import Layout from "@theme/Layout"

import HomePage from "../components/home"
import {PageDescription, PageTitle} from "../constants/titles"

const Home = (): JSX.Element => {
  return (
    <Layout title={PageTitle.HOME} description={PageDescription.HOME}>
      <HomePage />
    </Layout>
  )
}

export default Home
