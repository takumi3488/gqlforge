use anyhow::{Context, Result};
use sqlparser::ast::{
    AlterTableOperation, ColumnDef, ColumnOption, ColumnOptionDef, DataType, ObjectName, Statement,
    TableConstraint,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use super::schema::{
    Column, DatabaseSchema, ForeignKey, PgType, PrimaryKey, Table, UniqueConstraint,
};

/// Parse one or more SQL migration strings (in order) into a `DatabaseSchema`.
pub fn parse_migrations(sql_contents: &[String]) -> Result<DatabaseSchema> {
    let mut schema = DatabaseSchema::new();
    let dialect = PostgreSqlDialect {};

    for (idx, sql) in sql_contents.iter().enumerate() {
        let statements = Parser::parse_sql(&dialect, sql)
            .with_context(|| format!("Failed to parse migration #{idx}"))?;

        for stmt in statements {
            apply_statement(&mut schema, &stmt)?;
        }
    }

    Ok(schema)
}

fn apply_statement(schema: &mut DatabaseSchema, stmt: &Statement) -> Result<()> {
    match stmt {
        Statement::CreateTable(create) => {
            let (table_schema, table_name) = extract_schema_and_name(&create.name);
            let mut columns = Vec::new();
            let mut primary_key = None;
            let mut foreign_keys = Vec::new();
            let mut unique_constraints = Vec::new();

            for col_def in &create.columns {
                columns.push(column_from_def(col_def));
            }

            for constraint in &create.constraints {
                match constraint {
                    TableConstraint::PrimaryKey { columns: pk_cols, .. } => {
                        primary_key = Some(PrimaryKey {
                            columns: pk_cols.iter().map(|c| c.value.clone()).collect(),
                        });
                    }
                    TableConstraint::ForeignKey {
                        columns: fk_cols,
                        foreign_table,
                        referred_columns,
                        ..
                    } => {
                        let (ref_schema, ref_table) = extract_schema_and_name(foreign_table);
                        foreign_keys.push(ForeignKey {
                            columns: fk_cols.iter().map(|c| c.value.clone()).collect(),
                            referenced_schema: ref_schema,
                            referenced_table: ref_table,
                            referenced_columns: referred_columns
                                .iter()
                                .map(|c| c.value.clone())
                                .collect(),
                        });
                    }
                    TableConstraint::Unique { columns: u_cols, .. } => {
                        unique_constraints.push(UniqueConstraint {
                            columns: u_cols.iter().map(|c| c.value.clone()).collect(),
                        });
                    }
                    _ => {}
                }
            }

            // Extract inline PK from column options if no table-level PK.
            if primary_key.is_none() {
                let inline_pk_cols: Vec<String> = columns
                    .iter()
                    .zip(create.columns.iter())
                    .filter_map(|(col, def)| {
                        if def.options.iter().any(|opt| {
                            matches!(
                                opt,
                                ColumnOptionDef {
                                    option: ColumnOption::Unique { is_primary: true, .. },
                                    ..
                                }
                            )
                        }) {
                            Some(col.name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                if !inline_pk_cols.is_empty() {
                    primary_key = Some(PrimaryKey { columns: inline_pk_cols });
                }
            }

            // Table-level PK columns must be non-nullable
            if let Some(ref pk) = primary_key {
                for col in &mut columns {
                    if pk.columns.contains(&col.name) {
                        col.is_nullable = false;
                    }
                }
            }

            let table = Table {
                schema: table_schema,
                name: table_name,
                columns,
                primary_key,
                foreign_keys,
                unique_constraints,
            };
            schema.add_table(table);
        }
        Statement::AlterTable { name, operations, .. } => {
            let qualified = format_object_name(name);
            let key = if schema.tables.contains_key(&qualified) {
                Some(qualified)
            } else {
                let (_, tname) = extract_schema_and_name(name);
                let alt = format!("public.{tname}");
                if schema.tables.contains_key(&alt) {
                    Some(alt)
                } else {
                    None
                }
            };
            if let Some(table) = key.and_then(|k| schema.tables.get_mut(&k)) {
                for op in operations {
                    apply_alter_op(table, op)?;
                }
            }
        }
        _ => {
            // DROP, INSERT, etc. are ignored.
        }
    }
    Ok(())
}

fn apply_alter_op(table: &mut Table, op: &AlterTableOperation) -> Result<()> {
    match op {
        AlterTableOperation::AddColumn { column_def, .. } => {
            table.columns.push(column_from_def(column_def));
        }
        AlterTableOperation::DropColumn { column_name, .. } => {
            table.columns.retain(|c| c.name != column_name.value);
        }
        _ => {}
    }
    Ok(())
}

fn column_from_def(col: &ColumnDef) -> Column {
    let is_serial = format!("{}", col.data_type)
        .to_lowercase()
        .contains("serial");
    let is_nullable = !is_serial
        && !col.options.iter().any(|opt| {
            matches!(
                opt,
                ColumnOptionDef { option: ColumnOption::NotNull, .. }
                    | ColumnOptionDef { option: ColumnOption::Unique { is_primary: true, .. }, .. }
            )
        });
    let has_default = col.options.iter().any(|opt| {
        matches!(
            opt,
            ColumnOptionDef { option: ColumnOption::Default(_), .. }
        )
    });
    let is_generated = col.options.iter().any(|opt| {
        matches!(
            opt,
            ColumnOptionDef { option: ColumnOption::Generated { .. }, .. }
        )
    });

    let pg_type = data_type_to_pg_type(&col.data_type);

    Column {
        name: col.name.value.clone(),
        pg_type,
        is_nullable,
        has_default,
        is_generated,
    }
}

fn data_type_to_pg_type(dt: &DataType) -> PgType {
    match dt {
        DataType::SmallInt(_) => PgType::SmallInt,
        DataType::Int(_) | DataType::Integer(_) => PgType::Integer,
        DataType::BigInt(_) => PgType::BigInt,
        DataType::Real => PgType::Real,
        DataType::Double | DataType::DoublePrecision => PgType::DoublePrecision,
        DataType::Numeric(_) | DataType::Decimal(_) | DataType::Dec(_) => PgType::Numeric,
        DataType::Boolean => PgType::Boolean,
        DataType::Text => PgType::Text,
        DataType::Varchar(_) | DataType::CharacterVarying(_) => PgType::Varchar,
        DataType::Char(_) | DataType::Character(_) => PgType::Char,
        DataType::Uuid => PgType::Uuid,
        DataType::Date => PgType::Date,
        DataType::Timestamp(_, tz) => {
            if matches!(tz, sqlparser::ast::TimezoneInfo::WithTimeZone) {
                PgType::TimestampTz
            } else {
                PgType::Timestamp
            }
        }
        DataType::Time(_, tz) => {
            if matches!(tz, sqlparser::ast::TimezoneInfo::WithTimeZone) {
                PgType::TimeTz
            } else {
                PgType::Time
            }
        }
        DataType::Interval => PgType::Interval,
        DataType::JSON => PgType::Json,
        DataType::JSONB => PgType::Jsonb,
        DataType::Bytea => PgType::Bytea,
        DataType::Array(arr_inner) => {
            let inner = match arr_inner {
                sqlparser::ast::ArrayElemTypeDef::AngleBracket(inner_dt) => {
                    data_type_to_pg_type(inner_dt)
                }
                sqlparser::ast::ArrayElemTypeDef::SquareBracket(inner_dt, _) => {
                    data_type_to_pg_type(inner_dt)
                }
                sqlparser::ast::ArrayElemTypeDef::Parenthesis(inner_dt) => {
                    data_type_to_pg_type(inner_dt)
                }
                sqlparser::ast::ArrayElemTypeDef::None => PgType::Text,
            };
            PgType::Array(Box::new(inner))
        }
        _ => {
            // Handle types that sqlparser doesn't have dedicated variants for
            // (e.g. SERIAL, BIGSERIAL) by matching the Display output.
            let name = format!("{dt}");
            PgType::from_sql_name(&name)
        }
    }
}

fn extract_schema_and_name(name: &ObjectName) -> (String, String) {
    let parts: Vec<&str> = name.0.iter().map(|p| p.value.as_str()).collect();
    match parts.as_slice() {
        [] => ("public".to_string(), String::new()),
        [schema, table] => (schema.to_string(), table.to_string()),
        [table] => ("public".to_string(), table.to_string()),
        other => {
            let len = other.len();
            (other[len - 2].to_string(), other[len - 1].to_string())
        }
    }
}

fn format_object_name(name: &ObjectName) -> String {
    let (schema, table) = extract_schema_and_name(name);
    format!("{schema}.{table}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_create_table() {
        let sql = r#"
            CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email TEXT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
        "#
        .to_string();

        let schema = parse_migrations(&[sql]).unwrap();
        let table = schema.find_table("users").unwrap();

        assert_eq!(table.name, "users");
        assert_eq!(table.schema, "public");
        assert_eq!(table.columns.len(), 4);

        let id_col = table.find_column("id").unwrap();
        assert_eq!(id_col.pg_type, PgType::Integer);
        assert!(!id_col.is_nullable);

        let name_col = table.find_column("name").unwrap();
        assert_eq!(name_col.pg_type, PgType::Varchar);
        assert!(!name_col.is_nullable);

        let email_col = table.find_column("email").unwrap();
        assert!(email_col.is_nullable);

        let created_col = table.find_column("created_at").unwrap();
        assert_eq!(created_col.pg_type, PgType::TimestampTz);
        assert!(created_col.has_default);

        assert!(table.primary_key.is_some());
        assert_eq!(table.primary_key.as_ref().unwrap().columns, vec!["id"]);
    }

    #[test]
    fn parse_foreign_key() {
        let sql = r#"
            CREATE TABLE posts (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );
        "#
        .to_string();

        let schema = parse_migrations(&[sql]).unwrap();
        let table = schema.find_table("posts").unwrap();

        assert_eq!(table.foreign_keys.len(), 1);
        let fk = &table.foreign_keys[0];
        assert_eq!(fk.columns, vec!["user_id"]);
        assert_eq!(fk.referenced_table, "users");
        assert_eq!(fk.referenced_columns, vec!["id"]);
    }

    #[test]
    fn parse_multiple_migrations() {
        let m1 = "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT NOT NULL);".to_string();
        let m2 = "ALTER TABLE users ADD COLUMN email TEXT;".to_string();

        let schema = parse_migrations(&[m1, m2]).unwrap();
        let table = schema.find_table("users").unwrap();
        assert_eq!(table.columns.len(), 3);
        assert!(table.find_column("email").is_some());
    }
}
