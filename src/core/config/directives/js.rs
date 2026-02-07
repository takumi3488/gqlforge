use gqlforge_macros::{DirectiveDefinition, InputDefinition};
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
    InputDefinition,
)]
#[directive_definition(repeatable, locations = "FieldDefinition, Object", lowercase_name)]
pub struct JS {
    pub name: String,
}
