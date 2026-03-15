mod gen_gql_schema;

use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use gqlforge::cli;
use gqlforge::core::FileIO;
use gqlforge::core::blueprint::Blueprint;
use gqlforge::core::config::RuntimeConfig;
use gqlforge::core::tracing::default_tracing_for_name;
use serde_json::{Value, json};

static JSON_SCHEMA_FILE: &str = "generated/.gqlforgerc.schema.json";
static GRAPHQL_SCHEMA_FILE: &str = "generated/.gqlforgerc.graphql";

#[tokio::main]
#[expect(clippy::unwrap_used, reason = "main function setup")]
async fn main() {
    tracing::subscriber::set_global_default(default_tracing_for_name("gqlforge_typedefs")).unwrap();
    let args: Vec<String> = env::args().collect();
    let arg = args.get(1);

    let Some(arg) = arg else {
        tracing::error!("An argument required, you can pass either `fix` or `check` argument");
        return;
    };
    match arg.as_str() {
        "fix" => {
            let result = mode_fix().await;
            if let Err(e) = result {
                tracing::error!("{e}");
                exit(1);
            }
        }
        "check" => {
            let result = mode_check().await;
            if let Err(e) = result {
                tracing::error!("{e}");
                exit(1);
            }
        }
        _ => {
            tracing::error!("Unknown argument, you can pass either `fix` or `check` argument");
        }
    }
}

async fn mode_check() -> Result<()> {
    let rt = cli::runtime::init(&Blueprint::default())?;
    let file_io = &*rt.file;

    check_json(file_io).await?;
    check_graphql(file_io).await?;

    Ok(())
}

async fn check_json(file_io: &dyn FileIO) -> Result<()> {
    let json_schema = get_json_path();
    let content = file_io
        .read(
            json_schema
                .to_str()
                .ok_or(anyhow!("Unable to determine path"))?,
        )
        .await?;
    let content = serde_json::from_str::<Value>(&content)?;
    let schema = get_updated_json();
    if content.eq(&schema) {
        Ok(())
    } else {
        Err(anyhow!("Schema file '{JSON_SCHEMA_FILE}' mismatch"))
    }
}

async fn check_graphql(file_io: &dyn FileIO) -> Result<()> {
    let graphql_schema = get_graphql_path();
    let content = file_io
        .read(
            graphql_schema
                .to_str()
                .ok_or(anyhow!("Unable to determine path"))?,
        )
        .await?;
    let schema = get_updated_graphql();
    if content.eq(&schema) {
        Ok(())
    } else {
        Err(anyhow!("Schema file '{GRAPHQL_SCHEMA_FILE}' mismatch"))
    }
}

async fn mode_fix() -> Result<()> {
    let rt = cli::runtime::init(&Blueprint::default())?;
    let file_io = rt.file;

    update_json(file_io.clone()).await?;
    update_graphql(file_io.clone()).await?;
    Ok(())
}

async fn update_graphql(file_io: Arc<dyn FileIO>) -> Result<()> {
    let schema = get_updated_graphql();

    let path = get_graphql_path();
    tracing::info!("Updating Graphql Schema: {GRAPHQL_SCHEMA_FILE}");
    file_io
        .write(
            path.to_str().ok_or(anyhow!("Unable to determine path"))?,
            schema.as_bytes(),
        )
        .await?;
    Ok(())
}

async fn update_json(file_io: Arc<dyn FileIO>) -> Result<()> {
    let path = get_json_path();
    let schema = serde_json::to_string_pretty(&get_updated_json())?;
    tracing::info!("Updating JSON Schema: {JSON_SCHEMA_FILE}");
    file_io
        .write(
            path.to_str().ok_or(anyhow!("Unable to determine path"))?,
            schema.as_bytes(),
        )
        .await?;
    Ok(())
}

#[expect(clippy::unwrap_used, reason = "CARGO_MANIFEST_DIR always has a parent")]
fn get_root_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn get_json_path() -> PathBuf {
    get_root_path().join(JSON_SCHEMA_FILE)
}

fn get_graphql_path() -> PathBuf {
    get_root_path().join(GRAPHQL_SCHEMA_FILE)
}

fn get_updated_json() -> Value {
    let schema = schemars::schema_for!(RuntimeConfig);
    json!(schema)
}

fn get_updated_graphql() -> String {
    let doc = gen_gql_schema::build_service_document();
    gqlforge::core::document::print(&doc)
}
