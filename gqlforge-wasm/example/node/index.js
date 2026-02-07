const tc = require("@gqlforge/gqlforge-node")

async function run() {
  try {
    let schema = "https://raw.githubusercontent.com/takumi3488/gqlforge/main/examples/jsonplaceholder.graphql"
    let builder = new tc.GqlforgeBuilder()
    builder = await builder.with_config(schema)
    let executor = await builder.build()
    let result = await executor.execute("{posts { id }}")
    console.log("result: " + result)
  } catch (error) {
    console.error("error: " + error)
  }
}

run()
