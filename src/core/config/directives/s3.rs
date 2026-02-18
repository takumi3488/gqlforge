use gqlforge_macros::{DirectiveDefinition, InputDefinition};
use serde::{Deserialize, Serialize};

use crate::core::is_default;

/// The operation type for an `@s3` directive.
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    strum_macros::Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum S3Operation {
    /// Generate a presigned URL for downloading an object.
    #[default]
    GetPresignedUrl,
    /// Generate a presigned URL for uploading an object.
    PutPresignedUrl,
    /// List objects in a bucket.
    List,
    /// Delete an object from a bucket.
    Delete,
}

/// The `@s3` directive maps a GraphQL field to an Amazon S3 or
/// S3-compatible storage operation.
///
/// Supports presigned URL generation, object listing, and deletion.
/// Mustache templates can be used in `bucket`, `key`, `prefix`, and
/// `contentType` fields for dynamic values.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    InputDefinition,
    DirectiveDefinition,
)]
#[directive_definition(repeatable, locations = "FieldDefinition")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct S3 {
    /// The target S3 bucket name (Mustache template supported).
    pub bucket: String,

    /// The S3 operation to perform.
    pub operation: S3Operation,

    /// The object key (Mustache template supported).
    /// Required for GET_PRESIGNED_URL, PUT_PRESIGNED_URL, and DELETE.
    #[serde(default, skip_serializing_if = "is_default")]
    pub key: Option<String>,

    /// Prefix for LIST operations (Mustache template supported).
    #[serde(default, skip_serializing_if = "is_default")]
    pub prefix: Option<String>,

    /// Presigned URL expiration time in seconds (default: 3600).
    #[serde(
        default = "default_expiration",
        skip_serializing_if = "is_default_expiration"
    )]
    pub expiration: u64,

    /// Content-Type header for PUT presigned URLs (Mustache template supported).
    #[serde(default, skip_serializing_if = "is_default")]
    pub content_type: Option<String>,

    /// The @link id of the S3 connection to use.
    /// When omitted, the default (first) S3 connection is used.
    #[serde(default, skip_serializing_if = "is_default")]
    pub link_id: Option<String>,

    /// Enables deduplication of identical IO operations.
    #[serde(default, skip_serializing_if = "is_default")]
    pub dedupe: Option<bool>,
}

fn default_expiration() -> u64 {
    3600
}

fn is_default_expiration(val: &u64) -> bool {
    *val == 3600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let s3 = S3 {
            bucket: "my-bucket".to_string(),
            operation: S3Operation::GetPresignedUrl,
            key: Some("{{.args.key}}".to_string()),
            prefix: None,
            expiration: 3600,
            content_type: None,
            link_id: None,
            dedupe: None,
        };

        let json = serde_json::to_string(&s3).unwrap();
        let deserialized: S3 = serde_json::from_str(&json).unwrap();
        assert_eq!(s3, deserialized);
    }

    #[test]
    fn operation_screaming_snake_case() {
        let json = r#""GET_PRESIGNED_URL""#;
        let op: S3Operation = serde_json::from_str(json).unwrap();
        assert_eq!(op, S3Operation::GetPresignedUrl);

        let json = r#""PUT_PRESIGNED_URL""#;
        let op: S3Operation = serde_json::from_str(json).unwrap();
        assert_eq!(op, S3Operation::PutPresignedUrl);

        let json = r#""LIST""#;
        let op: S3Operation = serde_json::from_str(json).unwrap();
        assert_eq!(op, S3Operation::List);

        let json = r#""DELETE""#;
        let op: S3Operation = serde_json::from_str(json).unwrap();
        assert_eq!(op, S3Operation::Delete);
    }

    #[test]
    fn default_expiration_omitted() {
        let s3 = S3 {
            bucket: "b".to_string(),
            operation: S3Operation::List,
            expiration: 3600,
            ..Default::default()
        };
        let json = serde_json::to_value(&s3).unwrap();
        // Default expiration (3600) should be omitted
        assert!(json.get("expiration").is_none());
    }
}
