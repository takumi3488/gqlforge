use gqlforge_version::VERSION;

const UTM_MEDIUM: &str = "server";
const DEBUG_UTM_SOURCE: &str = "gqlforge-debug";
const RELEASE_UTM_SOURCE: &str = "gqlforge-release";
const BASE_PLAYGROUND_URL: &str = "https://gqlforge.pages.dev/playground/";

#[must_use] 
pub fn build_url(graphiql_url: &str) -> String {
    let utm_source = if VERSION.is_dev() {
        DEBUG_UTM_SOURCE
    } else {
        RELEASE_UTM_SOURCE
    };

    format!(
        "{BASE_PLAYGROUND_URL}?u={graphiql_url}&utm_source={utm_source}&utm_medium={UTM_MEDIUM}"
    )
}
