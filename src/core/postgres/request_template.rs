use std::hash::{Hash, Hasher};

use gqlforge_hasher::GqlforgeHasher;

use crate::core::config::PostgresOperation;
use crate::core::has_headers::HasHeaders;
use crate::core::ir::model::{CacheKey, IoId};
use crate::core::mustache::Mustache;
use crate::core::path::PathString;

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Template describing how to build a SQL query for a `@postgres` field.
#[derive(Debug, Clone)]
pub struct RequestTemplate {
    pub table: String,
    pub operation: PostgresOperation,
    pub filter: Option<Mustache>,
    pub input: Option<Mustache>,
    pub limit: Option<Mustache>,
    pub offset: Option<Mustache>,
    pub order_by: Option<Mustache>,
    /// Column names (resolved from `DatabaseSchema` at compile time).
    pub columns: Vec<String>,
}

/// A rendered, ready-to-execute SQL query with parameterised values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedQuery {
    pub sql: String,
    pub params: Vec<String>,
}

impl Hash for RenderedQuery {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sql.hash(state);
        self.params.hash(state);
    }
}

impl RequestTemplate {
    /// Render the template against the given context to produce a SQL string
    /// with positional parameters (`$1`, `$2`, …).
    pub fn render<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<RenderedQuery> {
        match self.operation {
            PostgresOperation::Select => self.render_select(ctx),
            PostgresOperation::SelectOne => self.render_select_one(ctx),
            PostgresOperation::Insert => self.render_insert(ctx),
            PostgresOperation::Update => self.render_update(ctx),
            PostgresOperation::Delete => self.render_delete(ctx),
        }
    }

    fn render_select<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<RenderedQuery> {
        let cols = self.select_columns();
        let table = quote_ident(&self.table);
        let mut sql = format!("SELECT {cols} FROM {table}");
        let mut params = Vec::new();

        if let Some(filter) = &self.filter {
            let (where_clause, where_params) = self.render_filter(filter, ctx, params.len())?;
            sql.push_str(&format!(" WHERE {where_clause}"));
            params.extend(where_params);
        }

        if let Some(order_by) = &self.order_by {
            let rendered = order_by.render(ctx);
            if !rendered.is_empty() {
                sql.push_str(&format!(" ORDER BY {rendered}"));
            }
        }

        if let Some(limit) = &self.limit {
            let rendered = limit.render(ctx);
            if !rendered.is_empty() {
                params.push(rendered);
                sql.push_str(&format!(" LIMIT ${}", params.len()));
            }
        }

        if let Some(offset) = &self.offset {
            let rendered = offset.render(ctx);
            if !rendered.is_empty() {
                params.push(rendered);
                sql.push_str(&format!(" OFFSET ${}", params.len()));
            }
        }

        Ok(RenderedQuery { sql, params })
    }

    fn render_select_one<C: PathString + HasHeaders>(
        &self,
        ctx: &C,
    ) -> anyhow::Result<RenderedQuery> {
        let cols = self.select_columns();
        let table = quote_ident(&self.table);
        let mut sql = format!("SELECT {cols} FROM {table}");
        let mut params = Vec::new();

        if let Some(filter) = &self.filter {
            let (where_clause, where_params) = self.render_filter(filter, ctx, params.len())?;
            sql.push_str(&format!(" WHERE {where_clause}"));
            params.extend(where_params);
        }

        sql.push_str(" LIMIT 1");
        Ok(RenderedQuery { sql, params })
    }

    fn render_insert<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<RenderedQuery> {
        let input_json = self
            .input
            .as_ref()
            .map(|m| m.render(ctx))
            .unwrap_or_default();

        let entries = parse_json_object(&input_json)?;
        let cols: Vec<String> = entries.iter().map(|(k, _)| quote_ident(k)).collect();
        let mut params: Vec<String> = Vec::new();
        let mut placeholders = Vec::new();

        for (_, v) in &entries {
            params.push(v.clone());
            placeholders.push(format!("${}", params.len()));
        }

        let col_list = cols.join(", ");
        let val_list = placeholders.join(", ");
        let ret_cols = self.select_columns();
        let table = quote_ident(&self.table);

        let sql =
            format!("INSERT INTO {table} ({col_list}) VALUES ({val_list}) RETURNING {ret_cols}");
        Ok(RenderedQuery { sql, params })
    }

    fn render_update<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<RenderedQuery> {
        let input_json = self
            .input
            .as_ref()
            .map(|m| m.render(ctx))
            .unwrap_or_default();

        let entries = parse_json_object(&input_json)?;
        let mut params: Vec<String> = Vec::new();
        let mut set_clauses = Vec::new();

        for (k, v) in &entries {
            params.push(v.clone());
            set_clauses.push(format!("{} = ${}", quote_ident(k), params.len()));
        }

        let set_str = set_clauses.join(", ");
        let ret_cols = self.select_columns();
        let table = quote_ident(&self.table);
        let mut sql = format!("UPDATE {table} SET {set_str}");

        if let Some(filter) = &self.filter {
            let (where_clause, where_params) = self.render_filter(filter, ctx, params.len())?;
            sql.push_str(&format!(" WHERE {where_clause}"));
            params.extend(where_params);
        }

        sql.push_str(&format!(" RETURNING {ret_cols}"));
        Ok(RenderedQuery { sql, params })
    }

    fn render_delete<C: PathString + HasHeaders>(&self, ctx: &C) -> anyhow::Result<RenderedQuery> {
        let table = quote_ident(&self.table);
        let mut sql = format!("DELETE FROM {table}");
        let mut params = Vec::new();

        if let Some(filter) = &self.filter {
            let (where_clause, where_params) = self.render_filter(filter, ctx, params.len())?;
            sql.push_str(&format!(" WHERE {where_clause}"));
            params.extend(where_params);
        }

        Ok(RenderedQuery { sql, params })
    }

    /// Parse a JSON filter object into `col = $N` clauses, returning the clause
    /// string and the parameter values.
    fn render_filter<C: PathString + HasHeaders>(
        &self,
        filter: &Mustache,
        ctx: &C,
        offset: usize,
    ) -> anyhow::Result<(String, Vec<String>)> {
        let rendered = filter.render(ctx);
        let entries = parse_json_object(&rendered)?;
        let mut params = Vec::new();
        let mut clauses = Vec::new();

        for (k, v) in entries {
            params.push(v);
            clauses.push(format!("{} = ${}", quote_ident(&k), offset + params.len()));
        }

        Ok((clauses.join(" AND "), params))
    }

    fn select_columns(&self) -> String {
        if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

impl<Ctx: PathString + HasHeaders> CacheKey<Ctx> for RequestTemplate {
    fn cache_key(&self, ctx: &Ctx) -> Option<IoId> {
        let rendered = self.render(ctx).ok()?;
        let mut hasher = GqlforgeHasher::default();
        rendered.hash(&mut hasher);
        Some(IoId::new(hasher.finish()))
    }
}

/// Parse a simple JSON object `{"k":"v", ...}` into key-value pairs.
/// Values are stringified (quotes stripped for simple strings).
fn parse_json_object(json_str: &str) -> anyhow::Result<Vec<(String, String)>> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in input/filter: {e}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Expected JSON object in input/filter, got: {value}"))?;

    Ok(obj
        .iter()
        .map(|(k, v)| {
            let val = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            (k.clone(), val)
        })
        .collect())
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
    fn render_select() {
        let tmpl = RequestTemplate {
            table: "users".into(),
            operation: PostgresOperation::Select,
            filter: Some(Mustache::parse(r#"{"active": "true"}"#)),
            input: None,
            limit: Some(Mustache::parse("10")),
            offset: Some(Mustache::parse("0")),
            order_by: Some(Mustache::parse("name ASC")),
            columns: vec!["id".into(), "name".into(), "email".into()],
        };

        let ctx = Ctx { value: serde_json::Value::Null };
        let rendered = tmpl.render(&ctx).unwrap();

        assert_eq!(
            rendered.sql,
            r#"SELECT "id", "name", "email" FROM "users" WHERE "active" = $1 ORDER BY name ASC LIMIT $2 OFFSET $3"#
        );
        assert_eq!(rendered.params, vec!["true", "10", "0"]);
    }

    #[test]
    fn render_insert() {
        let tmpl = RequestTemplate {
            table: "users".into(),
            operation: PostgresOperation::Insert,
            filter: None,
            input: Some(Mustache::parse(
                r#"{"name": "Alice", "email": "alice@example.com"}"#,
            )),
            limit: None,
            offset: None,
            order_by: None,
            columns: vec!["id".into(), "name".into(), "email".into()],
        };

        let ctx = Ctx { value: serde_json::Value::Null };
        let rendered = tmpl.render(&ctx).unwrap();

        assert!(rendered.sql.starts_with(r#"INSERT INTO "users""#));
        assert!(rendered.sql.contains("RETURNING"));
        assert_eq!(rendered.params.len(), 2);
    }

    #[test]
    fn render_delete() {
        let tmpl = RequestTemplate {
            table: "users".into(),
            operation: PostgresOperation::Delete,
            filter: Some(Mustache::parse(r#"{"id": "42"}"#)),
            input: None,
            limit: None,
            offset: None,
            order_by: None,
            columns: vec![],
        };

        let ctx = Ctx { value: serde_json::Value::Null };
        let rendered = tmpl.render(&ctx).unwrap();

        assert_eq!(rendered.sql, r#"DELETE FROM "users" WHERE "id" = $1"#);
        assert_eq!(rendered.params, vec!["42"]);
    }
}
