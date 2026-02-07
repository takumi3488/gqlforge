use std::num::NonZeroU64;

use gqlforge_macros::{DirectiveDefinition, InputDefinition, MergeRight};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Deserialize,
    Serialize,
    Eq,
    schemars::JsonSchema,
    MergeRight,
    DirectiveDefinition,
    InputDefinition,
)]
#[directive_definition(locations = "Object,FieldDefinition")]
/// The @cache operator enables caching for the query, field or type it is
/// applied to.
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Cache {
    /// Specifies the duration, in milliseconds, of how long the value has to be
    /// stored in the cache.
    pub max_age: NonZeroU64,
}
