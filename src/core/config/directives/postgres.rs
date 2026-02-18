use gqlforge_macros::{DirectiveDefinition, InputDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::is_default;

/// The operation type for a `@postgres` directive.
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    strum_macros::Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PostgresOperation {
    /// SELECT multiple rows (returns a list).
    #[default]
    Select,
    /// SELECT a single row by primary key or unique constraint.
    SelectOne,
    /// INSERT a new row.
    Insert,
    /// UPDATE an existing row.
    Update,
    /// DELETE a row.
    Delete,
}

/// The `@postgres` directive maps a GraphQL field to a PostgreSQL table
/// operation.
///
/// Supports CRUD operations with Mustache-templated filter expressions,
/// pagination, and batched data loading.
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
pub struct Postgres {
    /// The `@link(type: Postgres)` id to use. Optional when only one Postgres
    /// link is defined.
    #[serde(default, skip_serializing_if = "is_default")]
    pub db: Option<String>,

    /// The target table name (optionally schema-qualified, e.g.
    /// "public.users").
    pub table: String,

    /// The CRUD operation to perform.
    #[serde(default, skip_serializing_if = "is_default")]
    pub operation: PostgresOperation,

    /// A JSON object describing the WHERE clause. Supports Mustache templates
    /// for dynamic values, e.g. `{"id": "{{.args.id}}"}`.
    #[serde(default, skip_serializing_if = "is_default")]
    pub filter: Option<Value>,

    /// For INSERT/UPDATE: the input data source. Typically a Mustache template
    /// referencing the `input` argument, e.g. `"{{.args.input}}"`.
    #[serde(default, skip_serializing_if = "is_default")]
    pub input: Option<String>,

    /// Columns used for DataLoader batch keys (N+1 prevention).
    #[serde(rename = "batchKey", default, skip_serializing_if = "is_default")]
    pub batch_key: Vec<String>,

    /// Enables deduplication of identical IO operations.
    #[serde(default, skip_serializing_if = "is_default")]
    pub dedupe: Option<bool>,

    /// Mustache template for the LIMIT clause, e.g. `"{{.args.limit}}"`.
    #[serde(default, skip_serializing_if = "is_default")]
    pub limit: Option<String>,

    /// Mustache template for the OFFSET clause, e.g. `"{{.args.offset}}"`.
    #[serde(default, skip_serializing_if = "is_default")]
    pub offset: Option<String>,

    /// Mustache template for the ORDER BY clause, e.g. `"{{.args.orderBy}}"`.
    #[serde(default, skip_serializing_if = "is_default")]
    pub order_by: Option<String>,
}
