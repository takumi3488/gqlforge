use convert_case::{Case, Casing};
use indexmap::IndexMap;
use serde_json::json;

use crate::core::Type;
use crate::core::config::PostgresOperation;
use crate::core::config::{
    Arg, Config, Field, Link, LinkType, Postgres, Resolver, Type as ConfigType,
};
use crate::core::postgres::schema::{Column, DatabaseSchema, PgType};

/// Generate a GraphQL `Config` from a `DatabaseSchema`.
///
/// This follows the PostGraphile-style convention:
/// - Each table → an output type (PascalCase)
/// - Query: `tableNameById`, `tableNameList`
/// - Mutation: `createTableName`, `updateTableName`, `deleteTableName`
/// - FK relationships → nested object fields with `@postgres(batchKey: …)`
pub fn from_database(schema: &DatabaseSchema, connection_url: &str) -> anyhow::Result<Config> {
    let mut config = Config::default();
    config.schema.query = Some("Query".to_string());
    config.schema.mutation = Some("Mutation".to_string());

    let mut query_type = ConfigType::default();
    let mut mutation_type = ConfigType::default();

    for table in schema.tables.values() {
        let type_name = table_to_type_name(&table.name);

        // --- Output type ---
        let mut output_type = ConfigType::default();
        for col in &table.columns {
            let gql_type = column_to_graphql_type(col);
            output_type.fields.insert(
                col.name.to_case(Case::Camel),
                Field::default().type_of(gql_type),
            );
        }

        // --- FK relationship fields ---
        for fk in &table.foreign_keys {
            let ref_type_name = table_to_type_name(&fk.referenced_table);
            let field_name = fk.referenced_table.to_case(Case::Camel);

            let filter_obj: serde_json::Value = fk
                .referenced_columns
                .iter()
                .zip(fk.columns.iter())
                .map(|(ref_col, col)| {
                    (
                        ref_col.clone(),
                        serde_json::Value::String(format!(
                            "{{{{.value.{}}}}}",
                            col.to_case(Case::Camel)
                        )),
                    )
                })
                .collect::<serde_json::Map<String, serde_json::Value>>()
                .into();

            let resolver = Resolver::Postgres(Postgres {
                table: fk.referenced_table.clone(),
                operation: PostgresOperation::SelectOne,
                filter: Some(filter_obj),
                batch_key: fk.referenced_columns.clone(),
                ..Default::default()
            });

            output_type.fields.insert(
                field_name,
                Field::default()
                    .type_of(Type::from(ref_type_name.clone()))
                    .resolvers(resolver.into()),
            );
        }

        // --- Reverse FK (has-many) relationships ---
        for other_table in schema.tables.values() {
            for fk in &other_table.foreign_keys {
                if fk.referenced_table == table.name
                    || fk.referenced_table == table.qualified_name()
                {
                    let child_type_name = table_to_type_name(&other_table.name);
                    let field_name = pluralise(&other_table.name.to_case(Case::Camel));

                    let filter_obj: serde_json::Value = fk
                        .columns
                        .iter()
                        .zip(fk.referenced_columns.iter())
                        .map(|(col, ref_col)| {
                            (
                                col.clone(),
                                serde_json::Value::String(format!(
                                    "{{{{.value.{}}}}}",
                                    ref_col.to_case(Case::Camel)
                                )),
                            )
                        })
                        .collect::<serde_json::Map<String, serde_json::Value>>()
                        .into();

                    let resolver = Resolver::Postgres(Postgres {
                        table: other_table.name.clone(),
                        operation: PostgresOperation::Select,
                        filter: Some(filter_obj),
                        batch_key: fk.columns.clone(),
                        ..Default::default()
                    });

                    output_type.fields.insert(
                        field_name,
                        Field::default()
                            .type_of(
                                Type::from(child_type_name.clone())
                                    .into_list()
                                    .into_required(),
                            )
                            .resolvers(resolver.into()),
                    );
                }
            }
        }

        config.types.insert(type_name.clone(), output_type);

        // --- Query: byId ---
        if let Some(pk) = &table.primary_key {
            let by_id_name = format!("{}ById", table.name.to_case(Case::Camel));
            let mut args = IndexMap::new();
            let mut filter_map = serde_json::Map::new();

            for pk_col in &pk.columns {
                let col = table.find_column(pk_col);
                let gql_type = col
                    .map(|c| scalar_type(&c.pg_type))
                    .unwrap_or("ID".to_string());

                args.insert(
                    pk_col.to_case(Case::Camel),
                    Arg {
                        type_of: Type::from(gql_type.clone()).into_required(),
                        ..Default::default()
                    },
                );
                filter_map.insert(
                    pk_col.clone(),
                    json!(format!("{{{{.args.{}}}}}", pk_col.to_case(Case::Camel))),
                );
            }

            let resolver = Resolver::Postgres(Postgres {
                table: table.name.clone(),
                operation: PostgresOperation::SelectOne,
                filter: Some(serde_json::Value::Object(filter_map)),
                ..Default::default()
            });

            query_type.fields.insert(
                by_id_name,
                Field::default()
                    .type_of(Type::from(type_name.clone()))
                    .args(args)
                    .resolvers(resolver.into()),
            );
        }

        // --- Query: list ---
        {
            let list_name = format!("{}List", table.name.to_case(Case::Camel));

            let resolver = Resolver::Postgres(Postgres {
                table: table.name.clone(),
                operation: PostgresOperation::Select,
                limit: Some("{{.args.limit}}".to_string()),
                offset: Some("{{.args.offset}}".to_string()),
                ..Default::default()
            });

            query_type.fields.insert(
                list_name,
                Field::default()
                    .type_of(Type::from(type_name.clone()).into_list().into_required())
                    .args(IndexMap::from([
                        (
                            "limit".to_string(),
                            Arg { type_of: Type::from("Int".to_string()), ..Default::default() },
                        ),
                        (
                            "offset".to_string(),
                            Arg { type_of: Type::from("Int".to_string()), ..Default::default() },
                        ),
                    ]))
                    .resolvers(resolver.into()),
            );
        }

        // --- Mutation: create ---
        {
            let create_name = format!("create{type_name}");
            let input_type_name = format!("Create{type_name}Input");
            let mut input_type = ConfigType::default();

            for col in &table.columns {
                if col.is_generated {
                    continue;
                }
                if col.has_default
                    && table
                        .primary_key
                        .as_ref()
                        .is_some_and(|pk| pk.columns.contains(&col.name))
                {
                    continue;
                }
                let gql_type = column_to_input_type(col);
                input_type.fields.insert(
                    col.name.to_case(Case::Camel),
                    Field::default().type_of(gql_type),
                );
            }

            config.types.insert(input_type_name.clone(), input_type);

            let resolver = Resolver::Postgres(Postgres {
                table: table.name.clone(),
                operation: PostgresOperation::Insert,
                input: Some("{{.args.input}}".to_string()),
                ..Default::default()
            });

            mutation_type.fields.insert(
                create_name,
                Field::default()
                    .type_of(Type::from(type_name.clone()))
                    .args(IndexMap::from([(
                        "input".to_string(),
                        Arg {
                            type_of: Type::from(input_type_name.clone()).into_required(),
                            ..Default::default()
                        },
                    )]))
                    .resolvers(resolver.into()),
            );
        }

        // --- Mutation: update ---
        if let Some(pk) = &table.primary_key {
            let update_name = format!("update{type_name}");
            let input_type_name = format!("Update{type_name}Input");
            let mut input_type = ConfigType::default();

            for col in &table.columns {
                if col.is_generated {
                    continue;
                }
                if pk.columns.contains(&col.name) {
                    continue;
                }
                let gql_type = column_to_graphql_type(col);
                input_type.fields.insert(
                    col.name.to_case(Case::Camel),
                    Field::default().type_of(gql_type),
                );
            }

            config.types.insert(input_type_name.clone(), input_type);
            let mut args = IndexMap::new();
            let mut filter_map = serde_json::Map::new();
            for pk_col in &pk.columns {
                let col = table.find_column(pk_col);
                let gql_type = col
                    .map(|c| scalar_type(&c.pg_type))
                    .unwrap_or("ID".to_string());

                args.insert(
                    pk_col.to_case(Case::Camel),
                    Arg {
                        type_of: Type::from(gql_type.clone()).into_required(),
                        ..Default::default()
                    },
                );
                filter_map.insert(
                    pk_col.clone(),
                    json!(format!("{{{{.args.{}}}}}", pk_col.to_case(Case::Camel))),
                );
            }
            args.insert(
                "input".to_string(),
                Arg {
                    type_of: Type::from(input_type_name.clone()).into_required(),
                    ..Default::default()
                },
            );

            let resolver = Resolver::Postgres(Postgres {
                table: table.name.clone(),
                operation: PostgresOperation::Update,
                filter: Some(serde_json::Value::Object(filter_map)),
                input: Some("{{.args.input}}".to_string()),
                ..Default::default()
            });

            mutation_type.fields.insert(
                update_name,
                Field::default()
                    .type_of(Type::from(type_name.clone()))
                    .args(args)
                    .resolvers(resolver.into()),
            );
        }

        // --- Mutation: delete ---
        if let Some(pk) = &table.primary_key {
            let delete_name = format!("delete{type_name}");
            let mut args = IndexMap::new();
            let mut filter_map = serde_json::Map::new();

            for pk_col in &pk.columns {
                let col = table.find_column(pk_col);
                let gql_type = col
                    .map(|c| scalar_type(&c.pg_type))
                    .unwrap_or("ID".to_string());

                args.insert(
                    pk_col.to_case(Case::Camel),
                    Arg {
                        type_of: Type::from(gql_type.clone()).into_required(),
                        ..Default::default()
                    },
                );
                filter_map.insert(
                    pk_col.clone(),
                    json!(format!("{{{{.args.{}}}}}", pk_col.to_case(Case::Camel))),
                );
            }

            let resolver = Resolver::Postgres(Postgres {
                table: table.name.clone(),
                operation: PostgresOperation::Delete,
                filter: Some(serde_json::Value::Object(filter_map)),
                ..Default::default()
            });

            mutation_type.fields.insert(
                delete_name,
                Field::default()
                    .type_of(Type::from("Boolean".to_string()))
                    .args(args)
                    .resolvers(resolver.into()),
            );
        }
    }

    config.types.insert("Query".to_string(), query_type);
    config.types.insert("Mutation".to_string(), mutation_type);

    // Add the Postgres link.
    config.links.push(Link {
        id: None,
        src: connection_url.to_string(),
        type_of: LinkType::Postgres,
        headers: None,
        meta: None,
        proto_paths: None,
    });

    Ok(config)
}

fn table_to_type_name(table_name: &str) -> String {
    table_name.to_case(Case::Pascal)
}

fn column_to_graphql_type(col: &Column) -> Type {
    let scalar = scalar_type(&col.pg_type);
    let ty = Type::from(scalar);
    if col.is_nullable {
        ty
    } else {
        ty.into_required()
    }
}

fn column_to_input_type(col: &Column) -> Type {
    let scalar = scalar_type(&col.pg_type);
    let ty = Type::from(scalar);
    if col.is_nullable || col.has_default {
        ty
    } else {
        ty.into_required()
    }
}

fn scalar_type(pg: &PgType) -> String {
    pg.graphql_scalar().to_string()
}

fn pluralise(name: &str) -> String {
    let p = pluralizer::pluralize(name, 2, false);
    if p == *name { format!("{name}List") } else { p }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::postgres::schema::{Column, ForeignKey, PgType, PrimaryKey, Table};

    fn sample_schema() -> DatabaseSchema {
        let mut schema = DatabaseSchema::new();
        schema.add_table(Table {
            schema: "public".into(),
            name: "users".into(),
            columns: vec![
                Column {
                    name: "id".into(),
                    pg_type: PgType::Integer,
                    is_nullable: false,
                    has_default: true,
                    is_generated: false,
                },
                Column {
                    name: "name".into(),
                    pg_type: PgType::Text,
                    is_nullable: false,
                    has_default: false,
                    is_generated: false,
                },
                Column {
                    name: "email".into(),
                    pg_type: PgType::Text,
                    is_nullable: true,
                    has_default: false,
                    is_generated: false,
                },
            ],
            primary_key: Some(PrimaryKey { columns: vec!["id".into()] }),
            foreign_keys: vec![],
            unique_constraints: vec![],
        });
        schema.add_table(Table {
            schema: "public".into(),
            name: "posts".into(),
            columns: vec![
                Column {
                    name: "id".into(),
                    pg_type: PgType::Integer,
                    is_nullable: false,
                    has_default: true,
                    is_generated: false,
                },
                Column {
                    name: "user_id".into(),
                    pg_type: PgType::Integer,
                    is_nullable: false,
                    has_default: false,
                    is_generated: false,
                },
                Column {
                    name: "title".into(),
                    pg_type: PgType::Text,
                    is_nullable: false,
                    has_default: false,
                    is_generated: false,
                },
            ],
            primary_key: Some(PrimaryKey { columns: vec!["id".into()] }),
            foreign_keys: vec![ForeignKey {
                columns: vec!["user_id".into()],
                referenced_schema: "public".into(),
                referenced_table: "users".into(),
                referenced_columns: vec!["id".into()],
            }],
            unique_constraints: vec![],
        });
        schema
    }

    #[test]
    fn generates_query_types() {
        let schema = sample_schema();
        let config = from_database(&schema, "postgres://localhost/test").unwrap();

        // Check output types exist
        assert!(config.types.contains_key("Users"));
        assert!(config.types.contains_key("Posts"));

        // Check query fields
        let query = config.types.get("Query").unwrap();
        assert!(query.fields.contains_key("usersById"));
        assert!(query.fields.contains_key("usersList"));
        assert!(query.fields.contains_key("postsById"));
        assert!(query.fields.contains_key("postsList"));

        // Check mutation fields
        let mutation = config.types.get("Mutation").unwrap();
        assert!(mutation.fields.contains_key("createUsers"));
        assert!(mutation.fields.contains_key("updateUsers"));
        assert!(mutation.fields.contains_key("deleteUsers"));
    }

    #[test]
    fn generates_fk_relationships() {
        let schema = sample_schema();
        let config = from_database(&schema, "postgres://localhost/test").unwrap();

        // Posts should have a `users` field (FK to users)
        let posts_type = config.types.get("Posts").unwrap();
        assert!(posts_type.fields.contains_key("users"));

        // Users should have a `posts` has-many field (already plural → postsList)
        let users_type = config.types.get("Users").unwrap();
        assert!(users_type.fields.contains_key("postsList"));
    }
}
