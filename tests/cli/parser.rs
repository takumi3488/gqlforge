#![expect(clippy::expect_used, reason = "test code")]
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use anyhow::anyhow;
use gqlforge::core::FileIO;
use gqlforge::core::config::Source;
use markdown::ParseOptions;
use markdown::mdast::Node;

use crate::file::NativeFileTest;

#[derive(Clone)]
pub struct ExecutionSpec {
    pub env: Option<HashMap<String, String>>,
    pub configs: ConfigHolder,

    // if this is set to true,
    // then we will assert Config<Resolved>
    // instead of asserting the generated config
    pub debug_assert_config: bool,
}

pub struct IO {
    pub fs: NativeFileTest,
    pub paths: Vec<String>,
}

#[derive(Clone)]
pub struct ConfigHolder {
    configs: Vec<(Source, String)>,
}

impl ConfigHolder {
    pub async fn into_io(self) -> IO {
        let fs = NativeFileTest::default();
        let mut paths = vec![];
        for (i, (source, content)) in self.configs.iter().enumerate() {
            let path = format!("config{}.{}", i, source.ext());
            fs.write(&path, content.as_bytes())
                .await
                .expect("writing test config should succeed");
            paths.push(path);
        }
        IO { fs, paths }
    }
}

impl ExecutionSpec {
    pub fn from_source(path: &Path, contents: &str) -> anyhow::Result<Self> {
        let ast = markdown::to_mdast(contents, &ParseOptions::default())
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {e}", path.display()))?;
        let children = ast
            .children()
            .unwrap_or_else(|| panic!("Failed to parse {}: empty file unexpected", path.display()))
            .iter()
            .peekable();

        let mut env = None;
        let mut debug_assert_config = false;
        let mut configs = vec![];

        for node in children {
            match node {
                Node::Heading(heading) => {
                    if heading.depth == 2
                        && let Some(Node::Text(expect)) = heading.children.first()
                    {
                        let split = expect.value.splitn(2, ':').collect::<Vec<&str>>();
                        match split[..] {
                            [a, b] => {
                                debug_assert_config =
                                    a.contains("debug_assert") && b.ends_with("true");
                            }
                            _ => {
                                return Err(anyhow!(
                                    "Unexpected header annotation {:?} in {}",
                                    expect.value,
                                    path.display(),
                                ));
                            }
                        }
                    }
                }
                Node::Code(code) => {
                    let (content, lang, meta) = {
                        (
                            code.value.clone(),
                            code.lang.clone(),
                            code.meta.clone(),
                        )
                    };
                    if let Some(meta_str) = meta.as_ref().filter(|s| s.contains('@')) {
                        let temp_cleaned_meta = meta_str.replace('@', "");
                        let name: &str = &temp_cleaned_meta;

                        let lang = match lang {
                            Some(x) => Ok(x),
                            None => Err(anyhow!(
                                "Unexpected code block with no specific language in {}",
                                path.display()
                            )),
                        }?;
                        let source = Source::from_str(&lang)?;
                        match name {
                            "config" => {
                                configs.push((source, content));
                            }
                            "env" => {
                                let vars: HashMap<String, String> = match source {
                                    Source::Json => Ok(serde_json::from_str(&content)?),
                                    Source::Yml => Ok(serde_yaml_ng::from_str(&content)?),
                                    Source::GraphQL => Err(anyhow!(
                                        "Unexpected language in env block in {} (only JSON and YAML are supported)",
                                        path.display()
                                    )),
                                }?;

                                env = Some(vars);
                            }
                            _ => {
                                return Err(anyhow!(
                                    "Unexpected component {name:?} in {}: {meta:#?}",
                                    path.display()
                                ));
                            }
                        }
                    }
                }
                _ => return Err(anyhow!("Unexpected node in {}: {node:#?}", path.display())),
            }
        }

        Ok(Self { env, configs: ConfigHolder { configs }, debug_assert_config })
    }
}
