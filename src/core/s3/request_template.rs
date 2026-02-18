use std::hash::{Hash, Hasher};

use gqlforge_hasher::GqlforgeHasher;

use crate::core::config::S3Operation;
use crate::core::has_headers::HasHeaders;
use crate::core::ir::model::{CacheKey, IoId};
use crate::core::mustache::Mustache;
use crate::core::path::PathString;

/// Template describing how to build an S3 request for a `@s3` field.
#[derive(Debug, Clone)]
pub struct RequestTemplate {
    pub bucket: Mustache,
    pub operation: S3Operation,
    pub key: Option<Mustache>,
    pub prefix: Option<Mustache>,
    pub expiration: u64,
    pub content_type: Option<Mustache>,
    pub link_id: Option<String>,
}

/// A rendered, ready-to-execute S3 request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedRequest {
    pub bucket: String,
    pub operation: S3Operation,
    pub key: Option<String>,
    pub prefix: Option<String>,
    pub expiration: u64,
    pub content_type: Option<String>,
    pub link_id: Option<String>,
}

impl Hash for RenderedRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bucket.hash(state);
        self.operation.hash(state);
        self.key.hash(state);
        self.prefix.hash(state);
        self.expiration.hash(state);
        self.content_type.hash(state);
        self.link_id.hash(state);
    }
}

impl RequestTemplate {
    /// Render the template against the given context to produce a concrete
    /// S3 request.
    pub fn render<C: PathString + HasHeaders>(&self, ctx: &C) -> RenderedRequest {
        let bucket = self.bucket.render(ctx);
        let key = self
            .key
            .as_ref()
            .map(|m| m.render(ctx))
            .filter(|s| !s.is_empty());
        let prefix = self
            .prefix
            .as_ref()
            .map(|m| m.render(ctx))
            .filter(|s| !s.is_empty());
        let content_type = self
            .content_type
            .as_ref()
            .map(|m| m.render(ctx))
            .filter(|s| !s.is_empty());

        RenderedRequest {
            bucket,
            operation: self.operation.clone(),
            key,
            prefix,
            expiration: self.expiration,
            content_type,
            link_id: self.link_id.clone(),
        }
    }
}

impl<Ctx: PathString + HasHeaders> CacheKey<Ctx> for RequestTemplate {
    fn cache_key(&self, ctx: &Ctx) -> Option<IoId> {
        let rendered = self.render(ctx);
        let mut hasher = GqlforgeHasher::default();
        rendered.hash(&mut hasher);
        Some(IoId::new(hasher.finish()))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use http::HeaderMap;

    use super::*;

    struct Ctx {
        value: serde_json::Value,
    }

    impl PathString for Ctx {
        fn path_string<'a, T: AsRef<str>>(&'a self, parts: &'a [T]) -> Option<Cow<'a, str>> {
            self.value.path_string(parts)
        }
    }

    impl HasHeaders for Ctx {
        fn headers(&self) -> &HeaderMap {
            static EMPTY: std::sync::LazyLock<HeaderMap> = std::sync::LazyLock::new(HeaderMap::new);
            &EMPTY
        }
    }

    #[test]
    fn render_get_presigned_url() {
        let tmpl = RequestTemplate {
            bucket: Mustache::parse("my-bucket"),
            operation: S3Operation::GetPresignedUrl,
            key: Some(Mustache::parse("photos/image.jpg")),
            prefix: None,
            expiration: 3600,
            content_type: None,
            link_id: None,
        };

        let ctx = Ctx { value: serde_json::Value::Null };
        let rendered = tmpl.render(&ctx);

        assert_eq!(rendered.bucket, "my-bucket");
        assert_eq!(rendered.key, Some("photos/image.jpg".to_string()));
        assert_eq!(rendered.expiration, 3600);
    }

    #[test]
    fn render_list_with_prefix() {
        let tmpl = RequestTemplate {
            bucket: Mustache::parse("my-bucket"),
            operation: S3Operation::List,
            key: None,
            prefix: Some(Mustache::parse("uploads/")),
            expiration: 3600,
            content_type: None,
            link_id: Some("minio".to_string()),
        };

        let ctx = Ctx { value: serde_json::Value::Null };
        let rendered = tmpl.render(&ctx);

        assert_eq!(rendered.bucket, "my-bucket");
        assert_eq!(rendered.prefix, Some("uploads/".to_string()));
        assert_eq!(rendered.link_id, Some("minio".to_string()));
    }

    #[test]
    fn cache_key_consistency() {
        let tmpl = RequestTemplate {
            bucket: Mustache::parse("my-bucket"),
            operation: S3Operation::GetPresignedUrl,
            key: Some(Mustache::parse("file.txt")),
            prefix: None,
            expiration: 3600,
            content_type: None,
            link_id: None,
        };

        let ctx = Ctx { value: serde_json::Value::Null };
        let key1 = tmpl.cache_key(&ctx);
        let key2 = tmpl.cache_key(&ctx);

        assert_eq!(key1, key2);
        assert!(key1.is_some());
    }
}
