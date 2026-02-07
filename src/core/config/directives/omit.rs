use gqlforge_macros::{DirectiveDefinition, MergeRight};
use serde::{Deserialize, Serialize};

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    DirectiveDefinition,
    MergeRight,
)]
#[directive_definition(locations = "FieldDefinition")]
#[serde(deny_unknown_fields)]
/// Used to omit a field from public consumption.
pub struct Omit {}
