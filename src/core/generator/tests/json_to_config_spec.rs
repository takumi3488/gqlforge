use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use gqlforge::core::generator::{Generator, Input};
use gqlforge::core::http::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "body")]
    pub body: Option<Value>,
}

mod default {
    pub fn status() -> u16 {
        200
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    #[serde(default = "default::status")]
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, rename = "body")]
    pub body: Option<Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonFixture {
    request: APIRequest,
    response: APIResponse,
    #[serde(default)]
    is_mutation: Option<bool>,
    #[serde(default)]
    is_subscription: Option<bool>,
    field_name: String,
}

datatest_stable::harness! {
    { test = run_json_to_config_spec, root = "src/core/generator/tests/fixtures/json", pattern = r"^.*\.json" },
}

pub fn run_json_to_config_spec(path: &Path) -> datatest_stable::Result<()> {
    let json_data = load_json(path)?;
    test_spec(path, json_data)?;
    Ok(())
}

fn load_json(path: &Path) -> anyhow::Result<JsonFixture> {
    let contents = fs::read_to_string(path)?;
    let json_data: JsonFixture = serde_json::from_str(&contents).unwrap();
    Ok(json_data)
}

fn test_spec(path: &Path, json_data: JsonFixture) -> anyhow::Result<()> {
    let JsonFixture { request, response, is_mutation, is_subscription, field_name } = json_data;

    let req_body = request.body.unwrap_or_default();
    let resp_body = response.body.unwrap_or_default();

    let generator = Generator::default().inputs(vec![Input::Json {
        url: request.url,
        method: request.method,
        req_body,
        res_body: resp_body,
        field_name,
        is_mutation: is_mutation.unwrap_or_default(),
        is_subscription: is_subscription.unwrap_or_default(),
        headers: request.headers,
    }]);

    let mut config_gen = generator;
    if is_mutation.unwrap_or_default() {
        config_gen = config_gen.mutation(Some("Mutation".into()));
    }
    if is_subscription.unwrap_or_default() {
        config_gen = config_gen.subscription(Some("Subscription".into()));
    }
    let cfg = config_gen.generate(true)?;

    let snapshot_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid snapshot name"))?;

    insta::assert_snapshot!(snapshot_name, cfg.to_sdl());
    Ok(())
}
