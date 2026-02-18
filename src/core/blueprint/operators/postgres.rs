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
    let schemas = &inputs.config_module.extensions().database_schemas;

    // Resolve the connection id.
    let connection_id = match &pg.db {
        Some(id) => id.clone(),
        None => {
            if schemas.len() == 1 {
                schemas[0]
                    .id
                    .clone()
                    .unwrap_or_else(|| "default".to_string())
            } else if schemas.is_empty() {
                "default".to_string()
            } else {
                return Valid::fail(BlueprintError::Cause(
                    "@postgres requires 'db' when multiple Postgres connections are defined"
                        .to_string(),
                ));
            }
        }
    };

    // Validate that the table exists in the database schema (if available).
    let db_schema = inputs
        .config_module
        .extensions()
        .find_database_schema(Some(&connection_id));

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
                connection_id,
            }
        } else {
            IO::Postgres {
                req_template,
                group_by: None,
                dl_id: None,
                dedupe,
                connection_id,
            }
        };

        IR::IO(Box::new(io))
    })
}

#[cfg(test)]
mod tests {
    use gqlforge_valid::Validator;

    use super::*;
    use crate::core::config::{Config, Content, Extensions};
    use crate::core::postgres::schema::{Column, DatabaseSchema, PgType, Table};

    fn make_table(name: &str) -> Table {
        Table {
            schema: "public".to_string(),
            name: name.to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    pg_type: PgType::Integer,
                    is_nullable: false,
                    has_default: true,
                    is_generated: false,
                },
                Column {
                    name: "name".to_string(),
                    pg_type: PgType::Text,
                    is_nullable: false,
                    has_default: false,
                    is_generated: false,
                },
            ],
            primary_key: None,
            foreign_keys: vec![],
            unique_constraints: vec![],
        }
    }

    fn make_schema(table_name: &str) -> DatabaseSchema {
        let mut schema = DatabaseSchema::new();
        schema.add_table(make_table(table_name));
        schema
    }

    fn make_config_module(schemas: Vec<Content<DatabaseSchema>>) -> ConfigModule {
        let mut ext = Extensions::default();
        for s in schemas {
            ext.add_database_schema(s.id, s.content);
        }
        ConfigModule::new(Config::default(), ext)
    }

    #[test]
    fn single_schema_no_db_succeeds() {
        let cm = make_config_module(vec![Content {
            id: Some("main".to_string()),
            content: make_schema("users"),
        }]);
        let pg = Postgres { table: "users".to_string(), ..Default::default() };
        let result = compile_postgres(CompilePostgres { config_module: &cm, postgres: &pg });
        assert!(result.to_result().is_ok());
    }

    #[test]
    fn no_schema_uses_default_id() {
        let cm = make_config_module(vec![]);
        let pg = Postgres { table: "users".to_string(), ..Default::default() };
        let result = compile_postgres(CompilePostgres { config_module: &cm, postgres: &pg });
        // No schema → skips table validation, succeeds with connection_id "default"
        let ir = result.to_result().unwrap();
        match ir {
            IR::IO(io) => match io.as_ref() {
                IO::Postgres { connection_id, .. } => {
                    assert_eq!(connection_id, "default");
                }
                other => panic!("Expected IO::Postgres, got: {:?}", other),
            },
            other => panic!("Expected IR::IO, got: {:?}", other),
        }
    }

    #[test]
    fn multiple_schemas_no_db_fails() {
        let cm = make_config_module(vec![
            Content { id: Some("main".to_string()), content: make_schema("users") },
            Content {
                id: Some("analytics".to_string()),
                content: make_schema("events"),
            },
        ]);
        let pg = Postgres { table: "users".to_string(), ..Default::default() };
        let result = compile_postgres(CompilePostgres { config_module: &cm, postgres: &pg });
        assert!(result.to_result().is_err());
    }

    #[test]
    fn multiple_schemas_with_db_succeeds() {
        let cm = make_config_module(vec![
            Content { id: Some("main".to_string()), content: make_schema("users") },
            Content {
                id: Some("analytics".to_string()),
                content: make_schema("events"),
            },
        ]);
        let pg = Postgres {
            table: "users".to_string(),
            db: Some("main".to_string()),
            ..Default::default()
        };
        let result = compile_postgres(CompilePostgres { config_module: &cm, postgres: &pg });
        assert!(result.to_result().is_ok());
    }

    #[test]
    fn nonexistent_table_fails() {
        let cm = make_config_module(vec![Content {
            id: Some("main".to_string()),
            content: make_schema("users"),
        }]);
        let pg = Postgres {
            table: "nonexistent".to_string(),
            db: Some("main".to_string()),
            ..Default::default()
        };
        let result = compile_postgres(CompilePostgres { config_module: &cm, postgres: &pg });
        assert!(result.to_result().is_err());
    }
}
