use std::path::Path;

use super::helpers::{GQLFORGE_RC, GQLFORGE_RC_SCHEMA};
use crate::core::runtime::TargetRuntime;

pub async fn validate_rc_config_files(runtime: TargetRuntime, file_paths: &[String]) {
    // base config files.
    let gqlforgerc = include_str!("../../../generated/.gqlforgerc.graphql");
    let gqlforgerc_json = include_str!("../../../generated/.gqlforgerc.schema.json");

    // Define the config files to check with their base contents
    let rc_config_files = vec![
        (GQLFORGE_RC, gqlforgerc),
        (GQLFORGE_RC_SCHEMA, gqlforgerc_json),
    ];

    for path in file_paths {
        let parent_dir = match Path::new(path).parent() {
            Some(dir) => dir,
            None => continue,
        };

        let mut outdated_files = Vec::with_capacity(rc_config_files.len());

        for (file_name, base_content) in &rc_config_files {
            let config_path = parent_dir.join(file_name);
            if config_path.exists() {
                match runtime.file.read(&config_path.to_string_lossy()).await {
                    Ok(content) => {
                        if &content != base_content {
                            // file content not same.
                            outdated_files.push(file_name.to_owned());
                        }
                    }
                    _ => {
                        // unable to read file.
                        outdated_files.push(file_name.to_owned());
                    }
                }
            }
        }

        if !outdated_files.is_empty() {
            let outdated_files = outdated_files.join(", ");
            tracing::warn!(
                "[{}] {} outdated, reinitialize using gqlforge init.",
                outdated_files,
                pluralizer::pluralize("is", outdated_files.len() as isize, false)
            );
        }
    }
}
