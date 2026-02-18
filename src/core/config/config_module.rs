use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Deref;

use gqlforge_valid::{Valid, Validator};
use jsonwebtoken::jwk::JwkSet;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::core::Transform;
use crate::core::config::Config;
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;
use crate::core::postgres::schema::DatabaseSchema;
use crate::core::proto_reader::ProtoMetadata;
use crate::core::rest::{EndpointSet, Unchecked};

mod merge;

/// A wrapper on top of Config that contains all the resolved extensions and
/// computed values.
#[derive(Clone, Debug, Default)]
pub struct ConfigModule {
    extensions: Extensions,
    cache: Cache,
}

/// A cache that store resolved input, output and interface types so that it's
/// not computed again and again.
#[derive(Clone, Debug, Default)]
struct Cache {
    config: Config,
    input_types: HashSet<String>,
    output_types: HashSet<String>,
    interfaces_types_map: BTreeMap<String, BTreeSet<String>>,
}

impl From<Config> for Cache {
    fn from(value: Config) -> Self {
        let input_types = value.input_types();
        let output_types = value.output_types();
        let interfaces_types_map = value.interfaces_types_map();

        Cache {
            config: value,
            input_types,
            output_types,
            interfaces_types_map,
        }
    }
}

impl ConfigModule {
    pub fn new(config: Config, extensions: Extensions) -> Self {
        ConfigModule { cache: Cache::from(config), extensions }
    }

    pub fn set_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn merge_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = self.extensions.merge_right(extensions);
        self
    }

    pub fn config(&self) -> &Config {
        &self.cache.config
    }

    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub fn input_types(&self) -> &HashSet<String> {
        &self.cache.input_types
    }

    pub fn output_types(&self) -> &HashSet<String> {
        &self.cache.output_types
    }

    pub fn interfaces_types_map(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.cache.interfaces_types_map
    }

    pub fn transform<T: Transform<Value = Config>>(self, transformer: T) -> Valid<Self, T::Error> {
        transformer
            .transform(self.cache.config)
            .map(|config| ConfigModule::new(config, self.extensions))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Content<A> {
    pub id: Option<String>,
    pub content: A,
}

impl<A> Deref for Content<A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

#[derive(Debug)]
pub struct PrivateKey(PrivateKeyDer<'static>);

impl Clone for PrivateKey {
    fn clone(&self) -> Self {
        Self(self.0.clone_key())
    }
}

impl From<PrivateKeyDer<'static>> for PrivateKey {
    fn from(value: PrivateKeyDer<'static>) -> Self {
        Self(value)
    }
}

impl PrivateKey {
    pub fn into_inner(self) -> PrivateKeyDer<'static> {
        self.0
    }
}

/// Extensions are meta-information required before we can generate the
/// blueprint. Typically, this information cannot be inferred without performing
/// an IO operation, i.e., reading a file, making an HTTP call, etc.
#[derive(Clone, Debug, Default, MergeRight)]
pub struct Extensions {
    /// Contains the file descriptor set resolved from the links to proto files
    pub grpc_file_descriptors: HashMap<String, FileDescriptorProto>,

    /// Contains the contents of the JS file
    pub script: Option<String>,

    /// Contains the certificate used on HTTP2 with TLS
    pub cert: Vec<CertificateDer<'static>>,

    /// Contains the key used on HTTP2 with TLS
    pub keys: Vec<PrivateKey>,

    /// Contains the endpoints
    pub endpoint_set: EndpointSet<Unchecked>,

    pub htpasswd: Vec<Content<String>>,

    pub jwks: Vec<Content<JwkSet>>,

    /// Raw SQL migration contents, applied in order to build a DatabaseSchema.
    pub sql_migrations: Vec<String>,

    /// Resolved database schemas for `@postgres` directives.
    /// Each entry corresponds to a `@link(type: Postgres)` and carries an
    /// optional `id` for multi-database setups.
    pub database_schemas: Vec<Content<DatabaseSchema>>,

    /// S3 connection configurations from `@link(type: S3)` directives.
    pub s3_configs: Vec<S3LinkConfig>,
}

/// Configuration for an S3-compatible connection derived from a `@link`.
#[derive(Clone, Debug)]
pub struct S3LinkConfig {
    /// The @link id (used to match `linkId` in @s3 directives).
    pub id: String,
    /// The S3 endpoint URL (e.g. `https://s3.ap-northeast-1.amazonaws.com`).
    pub endpoint: String,
    /// The AWS region.
    pub region: String,
    /// Whether to use path-style addressing (for MinIO, R2, etc.).
    pub force_path_style: bool,
}

impl Extensions {
    pub fn add_proto(&mut self, metadata: ProtoMetadata) {
        for file in metadata.descriptor_set.file {
            self.grpc_file_descriptors
                .insert(file.name().to_string(), file);
        }
    }

    pub fn get_file_descriptor_set(&self) -> FileDescriptorSet {
        FileDescriptorSet { file: self.grpc_file_descriptors.values().cloned().collect() }
    }

    pub fn has_auth(&self) -> bool {
        !self.htpasswd.is_empty() || !self.jwks.is_empty()
    }

    pub fn add_sql_migration(&mut self, content: String) {
        self.sql_migrations.push(content);
    }

    pub fn add_database_schema(&mut self, id: Option<String>, schema: DatabaseSchema) {
        self.database_schemas.push(Content { id, content: schema });
    }

    /// Find a database schema by connection id.
    ///
    /// - If `id` is `Some`, returns the matching schema.
    /// - If `id` is `None` and there is exactly one schema, returns it.
    /// - Otherwise returns `None`.
    pub fn find_database_schema(&self, id: Option<&str>) -> Option<&DatabaseSchema> {
        match id {
            Some(id) => self
                .database_schemas
                .iter()
                .find(|c| c.id.as_deref() == Some(id))
                .map(|c| &c.content),
            None => {
                if self.database_schemas.len() == 1 {
                    Some(&self.database_schemas[0].content)
                } else {
                    None
                }
            }
        }
    }

    pub fn add_s3_config(&mut self, config: S3LinkConfig) {
        self.s3_configs.push(config);
    }
}

impl MergeRight for FileDescriptorSet {
    fn merge_right(mut self, other: Self) -> Self {
        self.file.extend(other.file);

        self
    }
}

impl Deref for ConfigModule {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        self.config()
    }
}

impl From<Config> for ConfigModule {
    fn from(config: Config) -> Self {
        ConfigModule { cache: Cache::from(config), ..Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::postgres::schema::DatabaseSchema;

    #[test]
    fn find_by_id_returns_matching() {
        let mut ext = Extensions::default();
        let schema = DatabaseSchema::new();
        ext.add_database_schema(Some("main".to_string()), schema.clone());
        ext.add_database_schema(Some("analytics".to_string()), DatabaseSchema::new());

        let found = ext.find_database_schema(Some("main"));
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &schema);
    }

    #[test]
    fn find_by_id_returns_none_for_missing() {
        let mut ext = Extensions::default();
        ext.add_database_schema(Some("main".to_string()), DatabaseSchema::new());

        assert!(ext.find_database_schema(Some("nonexistent")).is_none());
    }

    #[test]
    fn find_without_id_returns_single() {
        let mut ext = Extensions::default();
        let schema = DatabaseSchema::new();
        ext.add_database_schema(Some("main".to_string()), schema.clone());

        let found = ext.find_database_schema(None);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &schema);
    }

    #[test]
    fn find_without_id_returns_none_for_multiple() {
        let mut ext = Extensions::default();
        ext.add_database_schema(Some("main".to_string()), DatabaseSchema::new());
        ext.add_database_schema(Some("analytics".to_string()), DatabaseSchema::new());

        assert!(ext.find_database_schema(None).is_none());
    }
}
