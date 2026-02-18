use gqlforge_valid::{Valid, Validator};

use crate::core::blueprint::BlueprintError;
use crate::core::config::{ConfigModule, S3, S3Operation};
use crate::core::ir::model::{IO, IR};
use crate::core::mustache::Mustache;
use crate::core::s3::request_template::RequestTemplate;

pub struct CompileS3<'a> {
    pub config_module: &'a ConfigModule,
    pub s3: &'a S3,
}

pub fn compile_s3(inputs: CompileS3) -> Valid<IR, BlueprintError> {
    let s3 = inputs.s3;
    let dedupe = s3.dedupe.unwrap_or_default();

    // Validate that key-requiring operations have a key.
    let key_valid = match s3.operation {
        S3Operation::GetPresignedUrl | S3Operation::PutPresignedUrl | S3Operation::Delete => {
            if s3.key.is_none() {
                Valid::fail(BlueprintError::Cause(format!(
                    "@s3 operation {:?} requires a 'key' field",
                    s3.operation
                )))
            } else {
                Valid::succeed(())
            }
        }
        S3Operation::List => Valid::succeed(()),
    };

    key_valid.map(|_| {
        let bucket = Mustache::parse(&s3.bucket);
        let key = s3.key.as_ref().map(|v| Mustache::parse(v));
        let prefix = s3.prefix.as_ref().map(|v| Mustache::parse(v));
        let content_type = s3.content_type.as_ref().map(|v| Mustache::parse(v));

        let req_template = RequestTemplate {
            bucket,
            operation: s3.operation.clone(),
            key,
            prefix,
            expiration: s3.expiration,
            content_type,
            link_id: s3.link_id.clone(),
        };

        let io = IO::S3 { req_template, dedupe };

        IR::IO(Box::new(io))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::{Config, S3, S3Operation};

    #[test]
    fn compile_get_presigned_url_without_key_fails() {
        let config = ConfigModule::from(Config::default());
        let s3 = S3 {
            bucket: "my-bucket".to_string(),
            operation: S3Operation::GetPresignedUrl,
            key: None,
            ..Default::default()
        };

        let result = compile_s3(CompileS3 { config_module: &config, s3: &s3 });
        assert!(result.is_fail());
    }

    #[test]
    fn compile_delete_without_key_fails() {
        let config = ConfigModule::from(Config::default());
        let s3 = S3 {
            bucket: "my-bucket".to_string(),
            operation: S3Operation::Delete,
            key: None,
            ..Default::default()
        };

        let result = compile_s3(CompileS3 { config_module: &config, s3: &s3 });
        assert!(result.is_fail());
    }

    #[test]
    fn compile_list_without_key_succeeds() {
        let config = ConfigModule::from(Config::default());
        let s3 = S3 {
            bucket: "my-bucket".to_string(),
            operation: S3Operation::List,
            key: None,
            ..Default::default()
        };

        let result = compile_s3(CompileS3 { config_module: &config, s3: &s3 });
        assert!(result.is_succeed());
    }

    #[test]
    fn compile_get_presigned_url_with_key_succeeds() {
        let config = ConfigModule::from(Config::default());
        let s3 = S3 {
            bucket: "my-bucket".to_string(),
            operation: S3Operation::GetPresignedUrl,
            key: Some("{{.args.key}}".to_string()),
            expiration: 7200,
            ..Default::default()
        };

        let result = compile_s3(CompileS3 { config_module: &config, s3: &s3 });
        assert!(result.is_succeed());
    }
}
