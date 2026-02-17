use anyhow::{Context, Result};

use super::schema::{
    Column, DatabaseSchema, ForeignKey, PgType, PrimaryKey, Table, UniqueConstraint,
};

fn redact_url(url: &str) -> String {
    if let Some(at) = url.find('@')
        && let Some(scheme_end) = url.find("://")
    {
        return format!("{}://***{}", &url[..scheme_end], &url[at..]);
    }
    "<redacted>".to_string()
}

/// Connect to a live PostgreSQL instance and introspect its schema.
pub async fn introspect(connection_url: &str) -> Result<DatabaseSchema> {
    let tls = super::make_tls_connect()?;
    let (client, connection) = tokio_postgres::connect(connection_url, tls)
        .await
        .with_context(|| {
            format!(
                "Failed to connect to PostgreSQL: {}",
                redact_url(connection_url)
            )
        })?;

    // Spawn the connection handler.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("PostgreSQL connection error: {e}");
        }
    });

    let mut schema = DatabaseSchema::new();

    // --- 1. Fetch tables ---
    let tables_query = r#"
        SELECT table_schema, table_name
        FROM information_schema.tables
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
          AND table_type = 'BASE TABLE'
        ORDER BY table_schema, table_name
    "#;
    let table_rows = client.query(tables_query, &[]).await?;

    for row in &table_rows {
        let table_schema: String = row.get("table_schema");
        let table_name: String = row.get("table_name");

        let columns = fetch_columns(&client, &table_schema, &table_name).await?;
        let primary_key = fetch_primary_key(&client, &table_schema, &table_name).await?;
        let foreign_keys = fetch_foreign_keys(&client, &table_schema, &table_name).await?;
        let unique_constraints =
            fetch_unique_constraints(&client, &table_schema, &table_name).await?;

        schema.add_table(Table {
            schema: table_schema,
            name: table_name,
            columns,
            primary_key,
            foreign_keys,
            unique_constraints,
        });
    }

    Ok(schema)
}

async fn fetch_columns(
    client: &tokio_postgres::Client,
    schema: &str,
    table: &str,
) -> Result<Vec<Column>> {
    let query = r#"
        SELECT
            column_name,
            data_type,
            is_nullable,
            column_default,
            is_generated,
            is_identity
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        ORDER BY ordinal_position
    "#;
    let rows = client.query(query, &[&schema, &table]).await?;

    let mut columns = Vec::new();
    for row in rows {
        let name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let is_nullable: String = row.get("is_nullable");
        let column_default: Option<String> = row.get("column_default");
        let is_generated: String = row.get("is_generated");
        let is_identity: String = row.get("is_identity");

        columns.push(Column {
            name,
            pg_type: PgType::from_sql_name(&data_type),
            is_nullable: is_nullable == "YES",
            has_default: column_default.is_some(),
            is_generated: is_generated != "NEVER" || is_identity == "YES",
        });
    }

    Ok(columns)
}

async fn fetch_primary_key(
    client: &tokio_postgres::Client,
    schema: &str,
    table: &str,
) -> Result<Option<PrimaryKey>> {
    let query = r#"
        SELECT kcu.column_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name
         AND tc.table_schema = kcu.table_schema
        WHERE tc.constraint_type = 'PRIMARY KEY'
          AND tc.table_schema = $1
          AND tc.table_name = $2
        ORDER BY kcu.ordinal_position
    "#;
    let rows = client.query(query, &[&schema, &table]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let columns: Vec<String> = rows.iter().map(|r| r.get("column_name")).collect();
    Ok(Some(PrimaryKey { columns }))
}

async fn fetch_foreign_keys(
    client: &tokio_postgres::Client,
    schema: &str,
    table: &str,
) -> Result<Vec<ForeignKey>> {
    let query = r#"
        SELECT
            a1.attname AS column_name,
            ns2.nspname AS foreign_table_schema,
            cl2.relname AS foreign_table_name,
            a2.attname AS foreign_column_name,
            con.conname AS constraint_name
        FROM pg_catalog.pg_constraint con
        JOIN pg_catalog.pg_class cl ON cl.oid = con.conrelid
        JOIN pg_catalog.pg_namespace ns ON ns.oid = cl.relnamespace
        JOIN pg_catalog.pg_class cl2 ON cl2.oid = con.confrelid
        JOIN pg_catalog.pg_namespace ns2 ON ns2.oid = cl2.relnamespace
        CROSS JOIN LATERAL unnest(con.conkey, con.confkey) WITH ORDINALITY AS u(conkey, confkey, ord)
        JOIN pg_catalog.pg_attribute a1 ON a1.attrelid = con.conrelid AND a1.attnum = u.conkey
        JOIN pg_catalog.pg_attribute a2 ON a2.attrelid = con.confrelid AND a2.attnum = u.confkey
        WHERE con.contype = 'f'
          AND ns.nspname = $1
          AND cl.relname = $2
        ORDER BY con.conname, u.ord
    "#;
    let rows = client.query(query, &[&schema, &table]).await?;

    // Group by constraint name.
    let mut fk_map: std::collections::BTreeMap<String, ForeignKey> =
        std::collections::BTreeMap::new();
    for row in rows {
        let constraint: String = row.get("constraint_name");
        let col: String = row.get("column_name");
        let ref_schema: String = row.get("foreign_table_schema");
        let ref_table: String = row.get("foreign_table_name");
        let ref_col: String = row.get("foreign_column_name");

        let entry = fk_map.entry(constraint).or_insert_with(|| ForeignKey {
            columns: Vec::new(),
            referenced_schema: ref_schema,
            referenced_table: ref_table,
            referenced_columns: Vec::new(),
        });
        entry.columns.push(col);
        entry.referenced_columns.push(ref_col);
    }

    Ok(fk_map.into_values().collect())
}

async fn fetch_unique_constraints(
    client: &tokio_postgres::Client,
    schema: &str,
    table: &str,
) -> Result<Vec<UniqueConstraint>> {
    let query = r#"
        SELECT kcu.column_name, tc.constraint_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name
         AND tc.table_schema = kcu.table_schema
        WHERE tc.constraint_type = 'UNIQUE'
          AND tc.table_schema = $1
          AND tc.table_name = $2
        ORDER BY tc.constraint_name, kcu.ordinal_position
    "#;
    let rows = client.query(query, &[&schema, &table]).await?;

    let mut uc_map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for row in rows {
        let constraint: String = row.get("constraint_name");
        let col: String = row.get("column_name");
        uc_map.entry(constraint).or_default().push(col);
    }

    Ok(uc_map
        .into_values()
        .map(|columns| UniqueConstraint { columns })
        .collect())
}
