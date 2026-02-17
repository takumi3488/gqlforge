use gqlforge_valid::{Valid, Validator};

use crate::core::blueprint::BlueprintError;
use crate::core::config::group_by::GroupBy;
use crate::core::config::{ConfigModule, Postgres};
use crate::core::ir::model::{IO, IR};
use crate::core::mustache::Mustache;
use crate::core::postgres::request_template::RequestTemplate;

pub struct CompilePostgres<'a> {
    pub config_module: &'a ConfigModule,
    pub postgres: &'a Postgres,
}

pub fn compile_postgres(inputs: CompilePostgres) -> Valid<IR, BlueprintError> {
    let pg = inputs.postgres;
    let dedupe = pg.dedupe.unwrap_or_default();

    // Validate that the table exists in the database schema (if available).
    let db_schema = inputs.config_module.extensions().database_schema.as_ref();

    let table_valid = if let Some(schema) = db_schema {
        if schema.find_table(&pg.table).is_some() {
            Valid::succeed(())
        } else {
            Valid::fail(BlueprintError::Cause(format!(
                "Table '{}' not found in database schema",
                pg.table
            )))
        }
    } else {
        // If no database schema is loaded, skip validation (it will be
        // validated at runtime).
        Valid::succeed(())
    };

    table_valid.map(|_| {
        let filter = pg.filter.as_ref().map(|v| Mustache::parse(&v.to_string()));
        let input = pg.input.as_ref().map(|v| Mustache::parse(v));
        let limit = pg.limit.as_ref().map(|v| Mustache::parse(v));
        let offset = pg.offset.as_ref().map(|v| Mustache::parse(v));
        let order_by = pg.order_by.as_ref().map(|v| Mustache::parse(v));

        // Determine columns from database schema if available.
        let columns = db_schema
            .and_then(|s| s.find_table(&pg.table))
            .map(|t| t.columns.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default();

        let req_template = RequestTemplate {
            table: pg.table.clone(),
            operation: pg.operation.clone(),
            filter,
            input,
            limit,
            offset,
            order_by,
            columns,
        };

        let io = if !pg.batch_key.is_empty() {
            IO::Postgres {
                req_template,
                group_by: Some(GroupBy::new(pg.batch_key.clone(), None)),
                dl_id: None,
                dedupe,
            }
        } else {
            IO::Postgres { req_template, group_by: None, dl_id: None, dedupe }
        };

        IR::IO(Box::new(io))
    })
}
