#[cfg(feature = "postgres")]
pub mod pool {
    use async_graphql_value::ConstValue;
    use deadpool_postgres::{Config, Pool, Runtime};
    use indexmap::IndexMap;

    use crate::core::postgres::PostgresIO;

    // ---------------------------------------------------------------------------
    // Helper types
    // ---------------------------------------------------------------------------

    /// Wrapper that accepts any PostgreSQL type and reads the raw binary data.
    struct RawBytes(Vec<u8>);

    impl<'a> postgres_types::FromSql<'a> for RawBytes {
        fn from_sql(
            _ty: &postgres_types::Type,
            raw: &'a [u8],
        ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            Ok(RawBytes(raw.to_vec()))
        }

        fn accepts(_ty: &postgres_types::Type) -> bool {
            true
        }
    }

    /// Wrapper that reads PostgreSQL enum values as UTF-8 strings.
    struct EnumText(String);

    impl<'a> postgres_types::FromSql<'a> for EnumText {
        fn from_sql(
            _ty: &postgres_types::Type,
            raw: &'a [u8],
        ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = std::str::from_utf8(raw)?;
            Ok(EnumText(s.to_string()))
        }

        fn accepts(ty: &postgres_types::Type) -> bool {
            matches!(ty.kind(), postgres_types::Kind::Enum(_))
        }
    }

    /// Wrapper that converts a string parameter to the type PostgreSQL expects.
    #[derive(Debug)]
    struct TypedParam(String);

    impl postgres_types::ToSql for TypedParam {
        fn to_sql(
            &self,
            ty: &postgres_types::Type,
            out: &mut bytes::BytesMut,
        ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
            use postgres_types::Type;

            match *ty {
                Type::BOOL => {
                    let b = match self.0.to_lowercase().as_str() {
                        "true" | "t" | "1" | "yes" => true,
                        "false" | "f" | "0" | "no" => false,
                        other => {
                            return Err(format!("cannot convert '{other}' to bool").into());
                        }
                    };
                    b.to_sql(&Type::BOOL, out)
                }
                Type::INT2 => {
                    let v: i16 = self.0.parse()?;
                    v.to_sql(&Type::INT2, out)
                }
                Type::INT4 => {
                    let v: i32 = self.0.parse()?;
                    v.to_sql(&Type::INT4, out)
                }
                Type::INT8 => {
                    let v: i64 = self.0.parse()?;
                    v.to_sql(&Type::INT8, out)
                }
                Type::FLOAT4 => {
                    let v: f32 = self.0.parse()?;
                    v.to_sql(&Type::FLOAT4, out)
                }
                Type::FLOAT8 => {
                    let v: f64 = self.0.parse()?;
                    v.to_sql(&Type::FLOAT8, out)
                }
                Type::TIMESTAMP => {
                    let dt = parse_naive_datetime(&self.0)?;
                    dt.to_sql(&Type::TIMESTAMP, out)
                }
                Type::TIMESTAMPTZ => {
                    let dt = chrono::DateTime::parse_from_rfc3339(&self.0)
                        .map(|d| d.with_timezone(&chrono::Utc))?;
                    dt.to_sql(&Type::TIMESTAMPTZ, out)
                }
                Type::DATE => {
                    let d = chrono::NaiveDate::parse_from_str(&self.0, "%Y-%m-%d")?;
                    d.to_sql(&Type::DATE, out)
                }
                Type::TIME => {
                    let t = parse_naive_time(&self.0)?;
                    t.to_sql(&Type::TIME, out)
                }
                Type::UUID => {
                    let bytes = parse_uuid_to_bytes(&self.0)?;
                    out.extend_from_slice(&bytes);
                    Ok(postgres_types::IsNull::No)
                }
                _ => {
                    // TEXT, VARCHAR, enum, and everything else: send as raw UTF-8.
                    out.extend_from_slice(self.0.as_bytes());
                    Ok(postgres_types::IsNull::No)
                }
            }
        }

        fn accepts(_ty: &postgres_types::Type) -> bool {
            true
        }

        postgres_types::to_sql_checked!();
    }

    // ---------------------------------------------------------------------------
    // Binary format parsers
    // ---------------------------------------------------------------------------

    fn format_uuid(raw: &[u8]) -> anyhow::Result<String> {
        if raw.len() < 16 {
            anyhow::bail!("UUID binary too short");
        }
        Ok(format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            raw[0],
            raw[1],
            raw[2],
            raw[3],
            raw[4],
            raw[5],
            raw[6],
            raw[7],
            raw[8],
            raw[9],
            raw[10],
            raw[11],
            raw[12],
            raw[13],
            raw[14],
            raw[15]
        ))
    }

    fn parse_pg_numeric(raw: &[u8]) -> anyhow::Result<String> {
        if raw.len() < 8 {
            anyhow::bail!("NUMERIC binary too short");
        }
        let ndigits = i16::from_be_bytes([raw[0], raw[1]]) as i32;
        let weight = i16::from_be_bytes([raw[2], raw[3]]) as i32;
        let sign = u16::from_be_bytes([raw[4], raw[5]]);
        let dscale = i16::from_be_bytes([raw[6], raw[7]]) as i32;

        // NaN
        if sign == 0xC000 {
            return Ok("NaN".to_string());
        }

        if ndigits == 0 {
            if dscale > 0 {
                return Ok(format!("0.{}", "0".repeat(dscale as usize)));
            }
            return Ok("0".to_string());
        }

        let expected_len = 8 + ndigits as usize * 2;
        if raw.len() < expected_len {
            anyhow::bail!("NUMERIC binary too short for {ndigits} digits");
        }

        let mut digits = Vec::with_capacity(ndigits as usize);
        for i in 0..ndigits as usize {
            let offset = 8 + i * 2;
            digits.push(i16::from_be_bytes([raw[offset], raw[offset + 1]]));
        }

        // Build the integer and fractional parts from base-10000 digits.
        // `weight` is the exponent of the first digit in base 10000.
        // Digits with index <= weight contribute to the integer part;
        // digits with index > weight contribute to the fractional part.
        let mut int_part = String::new();
        let int_digits_count = weight + 1; // number of base-10000 positions in integer part

        if int_digits_count <= 0 {
            int_part.push('0');
        } else {
            for i in 0..int_digits_count {
                let d = if (i as usize) < digits.len() {
                    digits[i as usize]
                } else {
                    0
                };
                if i == 0 {
                    int_part.push_str(&d.to_string());
                } else {
                    int_part.push_str(&format!("{:04}", d));
                }
            }
        }

        let mut result = if sign == 0x4000 {
            format!("-{}", int_part)
        } else {
            int_part
        };

        if dscale > 0 {
            let mut frac = String::new();
            let frac_start = int_digits_count.max(0) as usize;
            // If weight < 0, we need leading zero groups.
            let leading_zero_groups = if weight < 0 {
                (-weight - 1) as usize
            } else {
                0
            };
            for _ in 0..leading_zero_groups {
                frac.push_str("0000");
            }
            for i in frac_start..digits.len() {
                frac.push_str(&format!("{:04}", digits[i]));
            }
            // Pad with zeros if needed.
            while frac.len() < dscale as usize {
                frac.push('0');
            }
            // Truncate to dscale.
            frac.truncate(dscale as usize);
            result.push('.');
            result.push_str(&frac);
        }

        Ok(result)
    }

    fn format_inet(raw: &[u8]) -> anyhow::Result<String> {
        if raw.len() < 4 {
            anyhow::bail!("INET binary too short");
        }
        let family = raw[0];
        let mask = raw[1];
        let is_cidr = raw[2] != 0;
        let addr_len = raw[3] as usize;

        if raw.len() < 4 + addr_len {
            anyhow::bail!("INET binary too short for address");
        }
        let addr = &raw[4..4 + addr_len];

        let addr_str = if family == 2 {
            // IPv4
            if addr_len != 4 {
                anyhow::bail!("Invalid IPv4 address length");
            }
            format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
        } else {
            // IPv6
            if addr_len != 16 {
                anyhow::bail!("Invalid IPv6 address length");
            }
            let mut parts = Vec::with_capacity(8);
            for i in 0..8 {
                let val = u16::from_be_bytes([addr[i * 2], addr[i * 2 + 1]]);
                parts.push(format!("{:x}", val));
            }
            let full = parts.join(":");
            compress_ipv6(&full)
        };

        let max_mask = if family == 2 { 32 } else { 128 };
        if is_cidr || mask != max_mask {
            Ok(format!("{}/{}", addr_str, mask))
        } else {
            Ok(addr_str)
        }
    }

    fn compress_ipv6(full: &str) -> String {
        let parts: Vec<&str> = full.split(':').collect();
        if parts.len() != 8 {
            return full.to_string();
        }

        // Find longest run of "0" groups.
        let mut best_start = 0usize;
        let mut best_len = 0usize;
        let mut cur_start = 0usize;
        let mut cur_len = 0usize;

        for (i, p) in parts.iter().enumerate() {
            if *p == "0" {
                if cur_len == 0 {
                    cur_start = i;
                }
                cur_len += 1;
            } else {
                if cur_len > best_len {
                    best_start = cur_start;
                    best_len = cur_len;
                }
                cur_len = 0;
            }
        }
        if cur_len > best_len {
            best_start = cur_start;
            best_len = cur_len;
        }

        if best_len < 2 {
            return full.to_string();
        }

        let before = parts[..best_start].join(":");
        let after = parts[best_start + best_len..].join(":");
        format!("{}::{}", before, after)
    }

    fn format_macaddr(raw: &[u8]) -> String {
        if raw.len() == 8 {
            // MACADDR8
            format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7]
            )
        } else {
            // MACADDR (6 bytes)
            format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                raw[0], raw[1], raw[2], raw[3], raw[4], raw[5]
            )
        }
    }

    fn format_interval(raw: &[u8]) -> anyhow::Result<String> {
        if raw.len() < 16 {
            anyhow::bail!("INTERVAL binary too short");
        }
        let microseconds = i64::from_be_bytes(raw[0..8].try_into()?);
        let days = i32::from_be_bytes(raw[8..12].try_into()?);
        let months = i32::from_be_bytes(raw[12..16].try_into()?);

        let mut parts = Vec::new();

        if months != 0 {
            let years = months / 12;
            let mons = months % 12;
            if years != 0 {
                if years == 1 || years == -1 {
                    parts.push(format!("{} year", years));
                } else {
                    parts.push(format!("{} years", years));
                }
            }
            if mons != 0 {
                if mons == 1 || mons == -1 {
                    parts.push(format!("{} mon", mons));
                } else {
                    parts.push(format!("{} mons", mons));
                }
            }
        }

        if days != 0 {
            if days == 1 || days == -1 {
                parts.push(format!("{} day", days));
            } else {
                parts.push(format!("{} days", days));
            }
        }

        if microseconds != 0 || parts.is_empty() {
            let negative = microseconds < 0;
            let abs_us = microseconds.unsigned_abs();
            let total_secs = abs_us / 1_000_000;
            let us_remainder = abs_us % 1_000_000;
            let hours = total_secs / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;

            let time_str = if us_remainder > 0 {
                let frac = format!("{:06}", us_remainder)
                    .trim_end_matches('0')
                    .to_string();
                format!("{:02}:{:02}:{:02}.{}", hours, minutes, seconds, frac)
            } else {
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            };

            if negative {
                parts.push(format!("-{}", time_str));
            } else {
                parts.push(time_str);
            }
        }

        Ok(parts.join(" "))
    }

    fn format_timetz(raw: &[u8]) -> anyhow::Result<String> {
        if raw.len() < 12 {
            anyhow::bail!("TIMETZ binary too short");
        }
        let microseconds = i64::from_be_bytes(raw[0..8].try_into()?);
        let offset_secs = i32::from_be_bytes(raw[8..12].try_into()?);

        let total_secs = microseconds / 1_000_000;
        let us_remainder = microseconds % 1_000_000;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        // PostgreSQL stores offset as seconds west of UTC (negated from usual convention).
        let tz_total = -offset_secs;
        let tz_sign = if tz_total >= 0 { '+' } else { '-' };
        let tz_abs = tz_total.unsigned_abs();
        let tz_hours = tz_abs / 3600;
        let tz_minutes = (tz_abs % 3600) / 60;

        let time_str = if us_remainder > 0 {
            let frac = format!("{:06}", us_remainder)
                .trim_end_matches('0')
                .to_string();
            format!("{:02}:{:02}:{:02}.{}", hours, minutes, seconds, frac)
        } else {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        };

        if tz_minutes > 0 {
            Ok(format!(
                "{}{}{:02}:{:02}",
                time_str, tz_sign, tz_hours, tz_minutes
            ))
        } else {
            Ok(format!("{}{}{:02}", time_str, tz_sign, tz_hours))
        }
    }

    fn format_bytea(raw: &[u8]) -> String {
        let mut s = String::with_capacity(2 + raw.len() * 2);
        s.push_str("\\x");
        for b in raw {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }

    // ---------------------------------------------------------------------------
    // Array parser
    // ---------------------------------------------------------------------------

    fn raw_element_to_const(ty: &postgres_types::Type, raw: &[u8]) -> anyhow::Result<ConstValue> {
        use postgres_types::Type;

        match *ty {
            Type::BOOL => {
                if raw.is_empty() {
                    return Ok(ConstValue::Null);
                }
                Ok(ConstValue::Boolean(raw[0] != 0))
            }
            Type::INT2 => {
                let v = i16::from_be_bytes(raw.try_into()?);
                Ok(ConstValue::Number(v.into()))
            }
            Type::INT4 => {
                let v = i32::from_be_bytes(raw.try_into()?);
                Ok(ConstValue::Number(v.into()))
            }
            Type::INT8 => {
                let v = i64::from_be_bytes(raw.try_into()?);
                Ok(ConstValue::Number(v.into()))
            }
            Type::FLOAT4 => {
                let v = f32::from_be_bytes(raw.try_into()?);
                Ok(serde_json::Number::from_f64(v as f64)
                    .map(ConstValue::Number)
                    .unwrap_or(ConstValue::Null))
            }
            Type::FLOAT8 => {
                let v = f64::from_be_bytes(raw.try_into()?);
                Ok(serde_json::Number::from_f64(v)
                    .map(ConstValue::Number)
                    .unwrap_or(ConstValue::Null))
            }
            Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
                let s = std::str::from_utf8(raw)?;
                Ok(ConstValue::String(s.to_string()))
            }
            Type::UUID => Ok(ConstValue::String(format_uuid(raw)?)),
            Type::BYTEA => Ok(ConstValue::String(format_bytea(raw))),
            Type::NUMERIC => Ok(ConstValue::String(parse_pg_numeric(raw)?)),
            Type::TIMESTAMP => {
                // PostgreSQL binary TIMESTAMP: i64 microseconds since 2000-01-01.
                let us = i64::from_be_bytes(raw.try_into()?);
                let epoch = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap();
                let dt = epoch + chrono::Duration::microseconds(us);
                Ok(ConstValue::String(
                    dt.format("%Y-%m-%dT%H:%M:%S%.f").to_string(),
                ))
            }
            Type::TIMESTAMPTZ => {
                let us = i64::from_be_bytes(raw.try_into()?);
                let epoch = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let dt = epoch + chrono::Duration::microseconds(us);
                Ok(ConstValue::String(dt.to_rfc3339()))
            }
            Type::DATE => {
                // i32 days since 2000-01-01.
                let days = i32::from_be_bytes(raw.try_into()?);
                let epoch = chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
                let d = epoch + chrono::Duration::days(days as i64);
                Ok(ConstValue::String(d.to_string()))
            }
            Type::TIME => {
                // i64 microseconds since midnight.
                let us = i64::from_be_bytes(raw.try_into()?);
                let total_secs = (us / 1_000_000) as u32;
                let micro = (us % 1_000_000) as u32;
                let t =
                    chrono::NaiveTime::from_num_seconds_from_midnight_opt(total_secs, micro * 1000)
                        .unwrap_or_default();
                Ok(ConstValue::String(t.to_string()))
            }
            Type::TIMETZ => Ok(ConstValue::String(format_timetz(raw)?)),
            Type::INTERVAL => Ok(ConstValue::String(format_interval(raw)?)),
            Type::INET | Type::CIDR => Ok(ConstValue::String(format_inet(raw)?)),
            Type::MACADDR | Type::MACADDR8 => Ok(ConstValue::String(format_macaddr(raw))),
            Type::JSON | Type::JSONB => {
                // JSONB binary has a version byte prefix (0x01); JSON does not.
                let data = if *ty == Type::JSONB && !raw.is_empty() {
                    &raw[1..]
                } else {
                    raw
                };
                let v: serde_json::Value = serde_json::from_slice(data)?;
                Ok(ConstValue::from_json(v).unwrap_or(ConstValue::Null))
            }
            _ => {
                // For enum and other types, try UTF-8 string.
                match std::str::from_utf8(raw) {
                    Ok(s) => Ok(ConstValue::String(s.to_string())),
                    Err(_) => Ok(ConstValue::String(format_bytea(raw))),
                }
            }
        }
    }

    fn parse_pg_array(raw: &[u8], elem_type: &postgres_types::Type) -> anyhow::Result<ConstValue> {
        if raw.len() < 12 {
            anyhow::bail!("array binary too short");
        }

        let ndim = i32::from_be_bytes(raw[0..4].try_into()?);
        // has_null flag at raw[4..8], elem_oid at raw[8..12] — we don't need them.

        if ndim == 0 {
            return Ok(ConstValue::List(vec![]));
        }

        // Read dimension sizes and lower bounds.
        let header_end = 12 + ndim as usize * 8;
        if raw.len() < header_end {
            anyhow::bail!("array binary too short for dimensions");
        }

        let mut dims = Vec::with_capacity(ndim as usize);
        for i in 0..ndim as usize {
            let offset = 12 + i * 8;
            let dim_size = i32::from_be_bytes(raw[offset..offset + 4].try_into()?) as usize;
            // lower_bound at offset+4..offset+8 — not needed for value parsing.
            dims.push(dim_size);
        }

        let mut pos = header_end;

        fn read_elements(
            raw: &[u8],
            pos: &mut usize,
            dims: &[usize],
            dim_idx: usize,
            elem_type: &postgres_types::Type,
        ) -> anyhow::Result<ConstValue> {
            if dim_idx == dims.len() - 1 {
                // Leaf dimension: read actual elements.
                let mut items = Vec::with_capacity(dims[dim_idx]);
                for _ in 0..dims[dim_idx] {
                    if *pos + 4 > raw.len() {
                        anyhow::bail!("array binary truncated");
                    }
                    let elem_len = i32::from_be_bytes(raw[*pos..*pos + 4].try_into()?) as i64;
                    *pos += 4;
                    if elem_len < 0 {
                        // NULL element.
                        items.push(ConstValue::Null);
                    } else {
                        let len = elem_len as usize;
                        if *pos + len > raw.len() {
                            anyhow::bail!("array element data truncated");
                        }
                        let elem_data = &raw[*pos..*pos + len];
                        *pos += len;
                        items.push(raw_element_to_const(elem_type, elem_data)?);
                    }
                }
                Ok(ConstValue::List(items))
            } else {
                // Nested dimension.
                let mut items = Vec::with_capacity(dims[dim_idx]);
                for _ in 0..dims[dim_idx] {
                    items.push(read_elements(raw, pos, dims, dim_idx + 1, elem_type)?);
                }
                Ok(ConstValue::List(items))
            }
        }

        read_elements(raw, &mut pos, &dims, 0, elem_type)
    }

    // ---------------------------------------------------------------------------
    // Datetime parse helpers (for TypedParam)
    // ---------------------------------------------------------------------------

    fn parse_naive_datetime(
        s: &str,
    ) -> Result<chrono::NaiveDateTime, Box<dyn std::error::Error + Sync + Send>> {
        let formats = [
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
        ];
        for fmt in &formats {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
                return Ok(dt);
            }
        }
        Err(format!("cannot parse '{s}' as TIMESTAMP").into())
    }

    fn parse_naive_time(
        s: &str,
    ) -> Result<chrono::NaiveTime, Box<dyn std::error::Error + Sync + Send>> {
        let formats = ["%H:%M:%S%.f", "%H:%M:%S", "%H:%M"];
        for fmt in &formats {
            if let Ok(t) = chrono::NaiveTime::parse_from_str(s, fmt) {
                return Ok(t);
            }
        }
        Err(format!("cannot parse '{s}' as TIME").into())
    }

    fn parse_uuid_to_bytes(s: &str) -> Result<[u8; 16], Box<dyn std::error::Error + Sync + Send>> {
        let hex: String = s.chars().filter(|c| *c != '-').collect();
        if hex.len() != 32 {
            return Err(format!("invalid UUID: '{s}'").into());
        }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)?;
        }
        Ok(bytes)
    }

    // ---------------------------------------------------------------------------
    // PostgresPool
    // ---------------------------------------------------------------------------

    /// A connection pool backed by `deadpool-postgres`.
    pub struct PostgresPool {
        pool: Pool,
    }

    impl PostgresPool {
        /// Create a new pool from a PostgreSQL connection string.
        pub fn new(connection_url: &str) -> anyhow::Result<Self> {
            let mut cfg = Config::new();
            cfg.url = Some(connection_url.to_string());

            let tls = crate::core::postgres::make_tls_connect()?;
            let pool = cfg
                .create_pool(Some(Runtime::Tokio1), tls)
                .map_err(|e| anyhow::anyhow!("Failed to create PostgreSQL pool: {e}"))?;

            Ok(Self { pool })
        }
    }

    #[async_trait::async_trait]
    impl PostgresIO for PostgresPool {
        async fn execute(&self, query: &str, params: &[String]) -> anyhow::Result<ConstValue> {
            let client = self.pool.get().await?;

            // Convert String params via TypedParam for correct PostgreSQL type encoding.
            let typed_params: Vec<TypedParam> =
                params.iter().map(|p| TypedParam(p.clone())).collect();
            let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = typed_params
                .iter()
                .map(|p| p as &(dyn tokio_postgres::types::ToSql + Sync))
                .collect();

            let rows = client.query(query, &param_refs).await?;

            // Convert rows to ConstValue (JSON array of objects).
            let mut result = Vec::new();
            for row in &rows {
                let mut obj = IndexMap::new();
                for (i, col) in row.columns().iter().enumerate() {
                    let value = row_value_to_const(row, i, col)?;
                    obj.insert(
                        async_graphql::Name::new(sanitize_graphql_name(col.name())),
                        value,
                    );
                }
                result.push(ConstValue::Object(obj));
            }

            Ok(ConstValue::List(result))
        }
    }

    fn sanitize_graphql_name(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        for c in name.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                result.push(c);
            } else {
                result.push('_');
            }
        }
        if result.starts_with(|c: char| c.is_ascii_digit()) {
            result.insert(0, '_');
        }
        if result.is_empty() {
            result.push_str("_unnamed");
        }
        if result.starts_with("__") {
            result.insert(0, '_');
        }
        result
    }

    fn row_value_to_const(
        row: &tokio_postgres::Row,
        idx: usize,
        col: &tokio_postgres::Column,
    ) -> anyhow::Result<ConstValue> {
        use postgres_types::Kind;
        use tokio_postgres::types::Type;

        let ty = col.type_();

        match *ty {
            Type::BOOL => {
                let v: Option<bool> = row.try_get(idx)?;
                Ok(v.map(ConstValue::Boolean).unwrap_or(ConstValue::Null))
            }
            Type::INT2 => {
                let v: Option<i16> = row.try_get(idx)?;
                Ok(v.map(|n| ConstValue::Number(n.into()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::INT4 => {
                let v: Option<i32> = row.try_get(idx)?;
                Ok(v.map(|n| ConstValue::Number(n.into()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::INT8 => {
                let v: Option<i64> = row.try_get(idx)?;
                Ok(v.map(|n| ConstValue::Number(n.into()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::FLOAT4 => {
                let v: Option<f32> = row.try_get(idx)?;
                Ok(v.and_then(|n| serde_json::Number::from_f64(n as f64))
                    .map(ConstValue::Number)
                    .unwrap_or(ConstValue::Null))
            }
            Type::FLOAT8 => {
                let v: Option<f64> = row.try_get(idx)?;
                Ok(v.and_then(serde_json::Number::from_f64)
                    .map(ConstValue::Number)
                    .unwrap_or(ConstValue::Null))
            }
            Type::JSON | Type::JSONB => {
                let v: Option<serde_json::Value> = row.try_get(idx)?;
                Ok(
                    v.map(|j| ConstValue::from_json(j).unwrap_or(ConstValue::Null))
                        .unwrap_or(ConstValue::Null),
                )
            }
            // --- chrono-based datetime types ---
            Type::TIMESTAMP => {
                let v: Option<chrono::NaiveDateTime> = row.try_get(idx)?;
                Ok(
                    v.map(|dt| ConstValue::String(dt.format("%Y-%m-%dT%H:%M:%S%.f").to_string()))
                        .unwrap_or(ConstValue::Null),
                )
            }
            Type::TIMESTAMPTZ => {
                let v: Option<chrono::DateTime<chrono::Utc>> = row.try_get(idx)?;
                Ok(v.map(|dt| ConstValue::String(dt.to_rfc3339()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::DATE => {
                let v: Option<chrono::NaiveDate> = row.try_get(idx)?;
                Ok(v.map(|d| ConstValue::String(d.to_string()))
                    .unwrap_or(ConstValue::Null))
            }
            Type::TIME => {
                let v: Option<chrono::NaiveTime> = row.try_get(idx)?;
                Ok(v.map(|t| ConstValue::String(t.to_string()))
                    .unwrap_or(ConstValue::Null))
            }
            // --- RawBytes-based types ---
            Type::TIMETZ => {
                let v: Option<RawBytes> = row.try_get(idx)?;
                match v {
                    Some(raw) => Ok(ConstValue::String(format_timetz(&raw.0)?)),
                    None => Ok(ConstValue::Null),
                }
            }
            Type::INTERVAL => {
                let v: Option<RawBytes> = row.try_get(idx)?;
                match v {
                    Some(raw) => Ok(ConstValue::String(format_interval(&raw.0)?)),
                    None => Ok(ConstValue::Null),
                }
            }
            Type::NUMERIC => {
                let v: Option<RawBytes> = row.try_get(idx)?;
                match v {
                    Some(raw) => Ok(ConstValue::String(parse_pg_numeric(&raw.0)?)),
                    None => Ok(ConstValue::Null),
                }
            }
            Type::UUID => {
                let v: Option<RawBytes> = row.try_get(idx)?;
                match v {
                    Some(raw) => Ok(ConstValue::String(format_uuid(&raw.0)?)),
                    None => Ok(ConstValue::Null),
                }
            }
            Type::BYTEA => {
                let v: Option<RawBytes> = row.try_get(idx)?;
                match v {
                    Some(raw) => Ok(ConstValue::String(format_bytea(&raw.0))),
                    None => Ok(ConstValue::Null),
                }
            }
            Type::INET | Type::CIDR => {
                let v: Option<RawBytes> = row.try_get(idx)?;
                match v {
                    Some(raw) => Ok(ConstValue::String(format_inet(&raw.0)?)),
                    None => Ok(ConstValue::Null),
                }
            }
            Type::MACADDR | Type::MACADDR8 => {
                let v: Option<RawBytes> = row.try_get(idx)?;
                match v {
                    Some(raw) => Ok(ConstValue::String(format_macaddr(&raw.0))),
                    None => Ok(ConstValue::Null),
                }
            }
            _ => {
                // Check for enum types.
                if matches!(ty.kind(), Kind::Enum(_)) {
                    let v: Option<EnumText> = row.try_get(idx)?;
                    return Ok(v
                        .map(|e| ConstValue::String(e.0))
                        .unwrap_or(ConstValue::Null));
                }

                // Check for array types.
                if let Kind::Array(elem_ty) = ty.kind() {
                    let v: Option<RawBytes> = row.try_get(idx)?;
                    return match v {
                        Some(raw) => Ok(parse_pg_array(&raw.0, elem_ty)?),
                        None => Ok(ConstValue::Null),
                    };
                }

                // Fallback: try to get as String (works for TEXT, VARCHAR, BPCHAR, NAME, UNKNOWN).
                match row.try_get::<_, Option<String>>(idx) {
                    Ok(v) => Ok(v.map(ConstValue::String).unwrap_or(ConstValue::Null)),
                    Err(_) => {
                        tracing::warn!(
                            column = col.name(),
                            pg_type = %ty,
                            "unsupported PostgreSQL type, returning null"
                        );
                        Ok(ConstValue::Null)
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Unit tests
    // ---------------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        // ===== 7-1: Binary format parser tests =====

        #[test]
        fn test_format_uuid_known() {
            let bytes: [u8; 16] = [
                0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
                0x00, 0x00,
            ];
            assert_eq!(
                format_uuid(&bytes).unwrap(),
                "550e8400-e29b-41d4-a716-446655440000"
            );
        }

        #[test]
        fn test_format_uuid_all_zeros() {
            let bytes = [0u8; 16];
            assert_eq!(
                format_uuid(&bytes).unwrap(),
                "00000000-0000-0000-0000-000000000000"
            );
        }

        #[test]
        fn test_parse_pg_numeric_positive_integer() {
            // 123: ndigits=1, weight=0, sign=0x0000, dscale=0, digits=[123]
            let mut raw = Vec::new();
            raw.extend_from_slice(&1i16.to_be_bytes()); // ndigits
            raw.extend_from_slice(&0i16.to_be_bytes()); // weight
            raw.extend_from_slice(&0u16.to_be_bytes()); // sign (positive)
            raw.extend_from_slice(&0i16.to_be_bytes()); // dscale
            raw.extend_from_slice(&123i16.to_be_bytes()); // digit[0]
            assert_eq!(parse_pg_numeric(&raw).unwrap(), "123");
        }

        #[test]
        fn test_parse_pg_numeric_negative_decimal() {
            // -123.45: ndigits=2, weight=0, sign=0x4000, dscale=2, digits=[123, 4500]
            let mut raw = Vec::new();
            raw.extend_from_slice(&2i16.to_be_bytes());
            raw.extend_from_slice(&0i16.to_be_bytes());
            raw.extend_from_slice(&0x4000u16.to_be_bytes());
            raw.extend_from_slice(&2i16.to_be_bytes());
            raw.extend_from_slice(&123i16.to_be_bytes());
            raw.extend_from_slice(&4500i16.to_be_bytes());
            assert_eq!(parse_pg_numeric(&raw).unwrap(), "-123.45");
        }

        #[test]
        fn test_parse_pg_numeric_zero() {
            let mut raw = Vec::new();
            raw.extend_from_slice(&0i16.to_be_bytes());
            raw.extend_from_slice(&0i16.to_be_bytes());
            raw.extend_from_slice(&0u16.to_be_bytes());
            raw.extend_from_slice(&0i16.to_be_bytes());
            assert_eq!(parse_pg_numeric(&raw).unwrap(), "0");
        }

        #[test]
        fn test_parse_pg_numeric_nan() {
            let mut raw = Vec::new();
            raw.extend_from_slice(&0i16.to_be_bytes());
            raw.extend_from_slice(&0i16.to_be_bytes());
            raw.extend_from_slice(&0xC000u16.to_be_bytes());
            raw.extend_from_slice(&0i16.to_be_bytes());
            assert_eq!(parse_pg_numeric(&raw).unwrap(), "NaN");
        }

        #[test]
        fn test_parse_pg_numeric_large() {
            // 12345678.9012
            // In base-10000: 1234, 5678, 9012 => ndigits=3, weight=1, dscale=4
            let mut raw = Vec::new();
            raw.extend_from_slice(&3i16.to_be_bytes()); // ndigits
            raw.extend_from_slice(&1i16.to_be_bytes()); // weight
            raw.extend_from_slice(&0u16.to_be_bytes()); // sign
            raw.extend_from_slice(&4i16.to_be_bytes()); // dscale
            raw.extend_from_slice(&1234i16.to_be_bytes());
            raw.extend_from_slice(&5678i16.to_be_bytes());
            raw.extend_from_slice(&9012i16.to_be_bytes());
            assert_eq!(parse_pg_numeric(&raw).unwrap(), "12345678.9012");
        }

        #[test]
        fn test_format_inet_ipv4() {
            let raw = [2u8, 32, 0, 4, 192, 168, 1, 1];
            assert_eq!(format_inet(&raw).unwrap(), "192.168.1.1");
        }

        #[test]
        fn test_format_inet_ipv4_cidr() {
            let raw = [2u8, 8, 1, 4, 10, 0, 0, 0];
            assert_eq!(format_inet(&raw).unwrap(), "10.0.0.0/8");
        }

        #[test]
        fn test_format_inet_ipv6_loopback() {
            let mut raw = vec![3u8, 128, 0, 16];
            let mut addr = [0u8; 16];
            addr[15] = 1;
            raw.extend_from_slice(&addr);
            assert_eq!(format_inet(&raw).unwrap(), "::1");
        }

        #[test]
        fn test_format_macaddr() {
            let raw = [0x08, 0x00, 0x2b, 0x01, 0x02, 0x03];
            assert_eq!(format_macaddr(&raw), "08:00:2b:01:02:03");
        }

        #[test]
        fn test_format_interval_day_and_time() {
            // 1 day 02:30:00 => µs for 2h30m = 9_000_000_000, days=1, months=0
            let mut raw = Vec::new();
            raw.extend_from_slice(&9_000_000_000i64.to_be_bytes());
            raw.extend_from_slice(&1i32.to_be_bytes());
            raw.extend_from_slice(&0i32.to_be_bytes());
            assert_eq!(format_interval(&raw).unwrap(), "1 day 02:30:00");
        }

        #[test]
        fn test_format_interval_year_months() {
            // 1 year 2 mons => months=14
            let mut raw = Vec::new();
            raw.extend_from_slice(&0i64.to_be_bytes());
            raw.extend_from_slice(&0i32.to_be_bytes());
            raw.extend_from_slice(&14i32.to_be_bytes());
            assert_eq!(format_interval(&raw).unwrap(), "1 year 2 mons");
        }

        #[test]
        fn test_format_interval_zero() {
            let mut raw = Vec::new();
            raw.extend_from_slice(&0i64.to_be_bytes());
            raw.extend_from_slice(&0i32.to_be_bytes());
            raw.extend_from_slice(&0i32.to_be_bytes());
            assert_eq!(format_interval(&raw).unwrap(), "00:00:00");
        }

        #[test]
        fn test_format_timetz() {
            // 10:30:00+09:00
            // µs = (10*3600 + 30*60) * 1_000_000 = 37_800_000_000
            // PostgreSQL offset = seconds west of UTC = -32400 (since +09 is 9*3600 east)
            let mut raw = Vec::new();
            raw.extend_from_slice(&37_800_000_000i64.to_be_bytes());
            raw.extend_from_slice(&(-32400i32).to_be_bytes());
            assert_eq!(format_timetz(&raw).unwrap(), "10:30:00+09");
        }

        #[test]
        fn test_format_bytea_data() {
            let raw = [0xDE, 0xAD, 0xBE, 0xEF];
            assert_eq!(format_bytea(&raw), "\\xdeadbeef");
        }

        #[test]
        fn test_format_bytea_empty() {
            assert_eq!(format_bytea(&[]), "\\x");
        }

        // ===== 7-2: TypedParam to_sql tests =====

        fn typed_param_to_bytes(
            value: &str,
            ty: &postgres_types::Type,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
            use bytes::BytesMut;
            use postgres_types::ToSql;

            let param = TypedParam(value.to_string());
            let mut buf = BytesMut::new();
            param.to_sql(ty, &mut buf)?;
            Ok(buf.to_vec())
        }

        #[test]
        fn test_typed_param_bool_true() {
            let bytes = typed_param_to_bytes("true", &postgres_types::Type::BOOL).unwrap();
            assert_eq!(bytes, vec![1]);
        }

        #[test]
        fn test_typed_param_bool_false() {
            let bytes = typed_param_to_bytes("false", &postgres_types::Type::BOOL).unwrap();
            assert_eq!(bytes, vec![0]);
        }

        #[test]
        fn test_typed_param_bool_t() {
            let bytes = typed_param_to_bytes("t", &postgres_types::Type::BOOL).unwrap();
            assert_eq!(bytes, vec![1]);
        }

        #[test]
        fn test_typed_param_bool_f() {
            let bytes = typed_param_to_bytes("f", &postgres_types::Type::BOOL).unwrap();
            assert_eq!(bytes, vec![0]);
        }

        #[test]
        fn test_typed_param_bool_1() {
            let bytes = typed_param_to_bytes("1", &postgres_types::Type::BOOL).unwrap();
            assert_eq!(bytes, vec![1]);
        }

        #[test]
        fn test_typed_param_bool_0() {
            let bytes = typed_param_to_bytes("0", &postgres_types::Type::BOOL).unwrap();
            assert_eq!(bytes, vec![0]);
        }

        #[test]
        fn test_typed_param_bool_invalid() {
            assert!(typed_param_to_bytes("maybe", &postgres_types::Type::BOOL).is_err());
        }

        #[test]
        fn test_typed_param_int4() {
            let bytes = typed_param_to_bytes("42", &postgres_types::Type::INT4).unwrap();
            assert_eq!(bytes, 42i32.to_be_bytes().to_vec());
        }

        #[test]
        fn test_typed_param_int4_neg() {
            let bytes = typed_param_to_bytes("-1", &postgres_types::Type::INT4).unwrap();
            assert_eq!(bytes, (-1i32).to_be_bytes().to_vec());
        }

        #[test]
        fn test_typed_param_int8() {
            let bytes = typed_param_to_bytes("9999999999", &postgres_types::Type::INT8).unwrap();
            assert_eq!(bytes, 9999999999i64.to_be_bytes().to_vec());
        }

        #[test]
        fn test_typed_param_float8() {
            let bytes = typed_param_to_bytes("3.14", &postgres_types::Type::FLOAT8).unwrap();
            assert_eq!(bytes, 3.14f64.to_be_bytes().to_vec());
        }

        #[test]
        fn test_typed_param_uuid() {
            let bytes = typed_param_to_bytes(
                "550e8400-e29b-41d4-a716-446655440000",
                &postgres_types::Type::UUID,
            )
            .unwrap();
            assert_eq!(bytes.len(), 16);
            assert_eq!(
                bytes,
                vec![
                    0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55,
                    0x44, 0x00, 0x00
                ]
            );
        }

        #[test]
        fn test_typed_param_text() {
            let bytes = typed_param_to_bytes("hello", &postgres_types::Type::TEXT).unwrap();
            assert_eq!(bytes, b"hello".to_vec());
        }

        #[test]
        fn test_typed_param_text_default() {
            // Unknown type falls back to text encoding.
            let bytes = typed_param_to_bytes("hello", &postgres_types::Type::NAME).unwrap();
            assert_eq!(bytes, b"hello".to_vec());
        }

        // ===== 7-3: Array parser tests =====

        fn build_pg_array(elem_oid: i32, elements: &[Option<&[u8]>]) -> Vec<u8> {
            let mut buf = Vec::new();
            // ndim=1
            buf.extend_from_slice(&1i32.to_be_bytes());
            // has_null
            let has_null = if elements.iter().any(|e| e.is_none()) {
                1i32
            } else {
                0i32
            };
            buf.extend_from_slice(&has_null.to_be_bytes());
            // elem_oid
            buf.extend_from_slice(&elem_oid.to_be_bytes());
            // dim_size
            buf.extend_from_slice(&(elements.len() as i32).to_be_bytes());
            // lower_bound
            buf.extend_from_slice(&1i32.to_be_bytes());
            // elements
            for elem in elements {
                match elem {
                    None => {
                        buf.extend_from_slice(&(-1i32).to_be_bytes());
                    }
                    Some(data) => {
                        buf.extend_from_slice(&(data.len() as i32).to_be_bytes());
                        buf.extend_from_slice(data);
                    }
                }
            }
            buf
        }

        fn build_empty_pg_array() -> Vec<u8> {
            let mut buf = Vec::new();
            buf.extend_from_slice(&0i32.to_be_bytes()); // ndim=0
            buf.extend_from_slice(&0i32.to_be_bytes()); // has_null
            buf.extend_from_slice(&0i32.to_be_bytes()); // elem_oid
            buf
        }

        #[test]
        fn test_parse_pg_array_empty() {
            let raw = build_empty_pg_array();
            let result = parse_pg_array(&raw, &postgres_types::Type::INT4).unwrap();
            assert_eq!(result, ConstValue::List(vec![]));
        }

        #[test]
        fn test_parse_pg_array_int4() {
            let b1 = 1i32.to_be_bytes();
            let b2 = 2i32.to_be_bytes();
            let b3 = 3i32.to_be_bytes();
            let elems: Vec<Option<&[u8]>> = vec![Some(&b1), Some(&b2), Some(&b3)];
            // INT4 OID = 23
            let raw = build_pg_array(23, &elems);
            let result = parse_pg_array(&raw, &postgres_types::Type::INT4).unwrap();
            assert_eq!(
                result,
                ConstValue::List(vec![
                    ConstValue::Number(1.into()),
                    ConstValue::Number(2.into()),
                    ConstValue::Number(3.into()),
                ])
            );
        }

        #[test]
        fn test_parse_pg_array_text() {
            let elems: Vec<Option<&[u8]>> = vec![Some(b"a"), Some(b"b")];
            // TEXT OID = 25
            let raw = build_pg_array(25, &elems);
            let result = parse_pg_array(&raw, &postgres_types::Type::TEXT).unwrap();
            assert_eq!(
                result,
                ConstValue::List(vec![
                    ConstValue::String("a".to_string()),
                    ConstValue::String("b".to_string()),
                ])
            );
        }

        #[test]
        fn test_parse_pg_array_with_null() {
            let b1 = 1i32.to_be_bytes();
            let b3 = 3i32.to_be_bytes();
            let elems: Vec<Option<&[u8]>> = vec![Some(&b1), None, Some(&b3)];
            let raw = build_pg_array(23, &elems);
            let result = parse_pg_array(&raw, &postgres_types::Type::INT4).unwrap();
            assert_eq!(
                result,
                ConstValue::List(vec![
                    ConstValue::Number(1.into()),
                    ConstValue::Null,
                    ConstValue::Number(3.into()),
                ])
            );
        }

        #[test]
        fn test_parse_pg_array_bool() {
            let elems: Vec<Option<&[u8]>> = vec![Some(&[1u8]), Some(&[0u8])];
            // BOOL OID = 16
            let raw = build_pg_array(16, &elems);
            let result = parse_pg_array(&raw, &postgres_types::Type::BOOL).unwrap();
            assert_eq!(
                result,
                ConstValue::List(vec![ConstValue::Boolean(true), ConstValue::Boolean(false),])
            );
        }

        // ===== 7-4: raw_element_to_const tests =====

        #[test]
        fn test_raw_element_bool() {
            let result = raw_element_to_const(&postgres_types::Type::BOOL, &[1]).unwrap();
            assert_eq!(result, ConstValue::Boolean(true));

            let result = raw_element_to_const(&postgres_types::Type::BOOL, &[0]).unwrap();
            assert_eq!(result, ConstValue::Boolean(false));
        }

        #[test]
        fn test_raw_element_int4() {
            let result =
                raw_element_to_const(&postgres_types::Type::INT4, &42i32.to_be_bytes()).unwrap();
            assert_eq!(result, ConstValue::Number(42.into()));
        }

        #[test]
        fn test_raw_element_float8() {
            let result =
                raw_element_to_const(&postgres_types::Type::FLOAT8, &3.14f64.to_be_bytes())
                    .unwrap();
            match result {
                ConstValue::Number(n) => {
                    let v: f64 = n.as_f64().unwrap();
                    assert!((v - 3.14).abs() < 1e-10);
                }
                other => panic!("expected Number, got {:?}", other),
            }
        }

        #[test]
        fn test_raw_element_text() {
            let result = raw_element_to_const(&postgres_types::Type::TEXT, b"hello").unwrap();
            assert_eq!(result, ConstValue::String("hello".to_string()));
        }

        #[test]
        fn test_raw_element_uuid() {
            let bytes: [u8; 16] = [
                0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
                0x00, 0x00,
            ];
            let result = raw_element_to_const(&postgres_types::Type::UUID, &bytes).unwrap();
            assert_eq!(
                result,
                ConstValue::String("550e8400-e29b-41d4-a716-446655440000".to_string())
            );
        }

        // ===== IPv6 compression tests =====

        #[test]
        fn test_compress_ipv6() {
            assert_eq!(compress_ipv6("0:0:0:0:0:0:0:1"), "::1");
            assert_eq!(compress_ipv6("2001:db8:0:0:0:0:0:1"), "2001:db8::1");
            assert_eq!(compress_ipv6("0:0:0:0:0:0:0:0"), "::");
        }

        // ===== TypedParam error cases & additional types =====

        #[test]
        fn test_typed_param_int4_invalid() {
            assert!(typed_param_to_bytes("abc", &postgres_types::Type::INT4).is_err());
        }

        #[test]
        fn test_typed_param_int2() {
            let bytes = typed_param_to_bytes("123", &postgres_types::Type::INT2).unwrap();
            assert_eq!(bytes, 123i16.to_be_bytes().to_vec());
        }

        #[test]
        fn test_typed_param_float4() {
            let bytes = typed_param_to_bytes("1.5", &postgres_types::Type::FLOAT4).unwrap();
            assert_eq!(bytes, 1.5f32.to_be_bytes().to_vec());
        }

        #[test]
        fn test_typed_param_float8_invalid() {
            assert!(typed_param_to_bytes("xyz", &postgres_types::Type::FLOAT8).is_err());
        }

        #[test]
        fn test_typed_param_timestamp() {
            let result =
                typed_param_to_bytes("2024-01-15T10:30:00", &postgres_types::Type::TIMESTAMP);
            assert!(result.is_ok());
        }

        #[test]
        fn test_typed_param_date() {
            let bytes = typed_param_to_bytes("2024-01-15", &postgres_types::Type::DATE).unwrap();
            assert!(!bytes.is_empty());
        }

        #[test]
        fn test_typed_param_date_invalid() {
            assert!(typed_param_to_bytes("2024-13-01", &postgres_types::Type::DATE).is_err());
        }

        #[test]
        fn test_typed_param_time() {
            let bytes = typed_param_to_bytes("10:30:00", &postgres_types::Type::TIME).unwrap();
            assert!(!bytes.is_empty());
        }

        #[test]
        fn test_typed_param_uuid_invalid() {
            assert!(typed_param_to_bytes("invalid-uuid", &postgres_types::Type::UUID).is_err());
        }

        // ===== Helper function tests: parse_naive_datetime =====

        #[test]
        fn test_parse_naive_datetime_iso_t() {
            let dt = parse_naive_datetime("2024-01-15T10:30:00").unwrap();
            assert_eq!(dt.to_string(), "2024-01-15 10:30:00");
        }

        #[test]
        fn test_parse_naive_datetime_space() {
            let dt = parse_naive_datetime("2024-01-15 10:30:00").unwrap();
            assert_eq!(dt.to_string(), "2024-01-15 10:30:00");
        }

        #[test]
        fn test_parse_naive_datetime_with_frac() {
            let dt = parse_naive_datetime("2024-01-15T10:30:00.123456").unwrap();
            assert_eq!(dt.to_string(), "2024-01-15 10:30:00.123456");
        }

        #[test]
        fn test_parse_naive_datetime_invalid() {
            assert!(parse_naive_datetime("not-a-date").is_err());
        }

        // ===== Helper function tests: parse_naive_time =====

        #[test]
        fn test_parse_naive_time_hms() {
            let t = parse_naive_time("10:30:00").unwrap();
            assert_eq!(t.to_string(), "10:30:00");
        }

        #[test]
        fn test_parse_naive_time_hm() {
            let t = parse_naive_time("10:30").unwrap();
            assert_eq!(t.to_string(), "10:30:00");
        }

        #[test]
        fn test_parse_naive_time_invalid() {
            assert!(parse_naive_time("25:00:00").is_err());
        }

        // ===== Helper function tests: parse_uuid_to_bytes =====

        #[test]
        fn test_parse_uuid_to_bytes_valid() {
            let bytes = parse_uuid_to_bytes("550e8400-e29b-41d4-a716-446655440000").unwrap();
            assert_eq!(
                bytes,
                [
                    0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55,
                    0x44, 0x00, 0x00
                ]
            );
        }

        #[test]
        fn test_parse_uuid_to_bytes_no_hyphens() {
            let bytes = parse_uuid_to_bytes("550e8400e29b41d4a716446655440000").unwrap();
            assert_eq!(
                bytes,
                [
                    0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55,
                    0x44, 0x00, 0x00
                ]
            );
        }

        #[test]
        fn test_parse_uuid_to_bytes_invalid() {
            assert!(parse_uuid_to_bytes("invalid-uuid").is_err());
        }

        // ===== raw_element_to_const additional type tests =====

        #[test]
        fn test_raw_element_int2() {
            let result =
                raw_element_to_const(&postgres_types::Type::INT2, &42i16.to_be_bytes()).unwrap();
            assert_eq!(result, ConstValue::Number(42.into()));
        }

        #[test]
        fn test_raw_element_int8() {
            let result =
                raw_element_to_const(&postgres_types::Type::INT8, &9999999999i64.to_be_bytes())
                    .unwrap();
            assert_eq!(result, ConstValue::Number(9999999999i64.into()));
        }

        #[test]
        fn test_raw_element_float4() {
            let result =
                raw_element_to_const(&postgres_types::Type::FLOAT4, &1.5f32.to_be_bytes()).unwrap();
            match result {
                ConstValue::Number(n) => {
                    let v = n.as_f64().unwrap();
                    assert!((v - 1.5).abs() < 1e-6);
                }
                other => panic!("expected Number, got {:?}", other),
            }
        }

        #[test]
        fn test_raw_element_float4_nan() {
            let result =
                raw_element_to_const(&postgres_types::Type::FLOAT4, &f32::NAN.to_be_bytes())
                    .unwrap();
            assert_eq!(result, ConstValue::Null);
        }

        #[test]
        fn test_raw_element_varchar() {
            let result =
                raw_element_to_const(&postgres_types::Type::VARCHAR, b"test string").unwrap();
            assert_eq!(result, ConstValue::String("test string".to_string()));
        }

        #[test]
        fn test_raw_element_timestamp() {
            // 2024-01-15T10:30:00 is 8780 days + 37800 seconds after 2000-01-01T00:00:00
            // = (8780 * 86400 + 37800) * 1_000_000 µs = 758_629_800_000_000
            let us: i64 = 758_629_800_000_000;
            let result =
                raw_element_to_const(&postgres_types::Type::TIMESTAMP, &us.to_be_bytes()).unwrap();
            match result {
                ConstValue::String(s) => {
                    assert!(s.starts_with("2024-01-15T10:30:00"));
                }
                other => panic!("expected String, got {:?}", other),
            }
        }

        #[test]
        fn test_raw_element_timestamptz() {
            // Same µs value; TIMESTAMPTZ returns RFC3339 with +00:00
            let us: i64 = 758_629_800_000_000;
            let result =
                raw_element_to_const(&postgres_types::Type::TIMESTAMPTZ, &us.to_be_bytes())
                    .unwrap();
            match result {
                ConstValue::String(s) => {
                    assert!(s.contains("2024-01-15"));
                    assert!(s.contains("+00:00"));
                }
                other => panic!("expected String, got {:?}", other),
            }
        }

        #[test]
        fn test_raw_element_date() {
            // 2024-01-15 is 8780 days after 2000-01-01
            let days: i32 = 8780;
            let result =
                raw_element_to_const(&postgres_types::Type::DATE, &days.to_be_bytes()).unwrap();
            assert_eq!(result, ConstValue::String("2024-01-15".to_string()));
        }

        #[test]
        fn test_raw_element_time() {
            // 10:30:00 = (10*3600 + 30*60) * 1_000_000 µs = 37_800_000_000
            let us: i64 = 37_800_000_000;
            let result =
                raw_element_to_const(&postgres_types::Type::TIME, &us.to_be_bytes()).unwrap();
            assert_eq!(result, ConstValue::String("10:30:00".to_string()));
        }

        #[test]
        fn test_raw_element_json() {
            let json_bytes = br#"{"key":"value"}"#;
            let result = raw_element_to_const(&postgres_types::Type::JSON, json_bytes).unwrap();
            match result {
                ConstValue::Object(obj) => {
                    assert_eq!(
                        obj.get("key"),
                        Some(&ConstValue::String("value".to_string()))
                    );
                }
                other => panic!("expected Object, got {:?}", other),
            }
        }

        #[test]
        fn test_raw_element_jsonb() {
            // JSONB has a 0x01 version byte prefix
            let mut jsonb_bytes = vec![0x01u8];
            jsonb_bytes.extend_from_slice(br#"{"key":"value"}"#);
            let result = raw_element_to_const(&postgres_types::Type::JSONB, &jsonb_bytes).unwrap();
            match result {
                ConstValue::Object(obj) => {
                    assert_eq!(
                        obj.get("key"),
                        Some(&ConstValue::String("value".to_string()))
                    );
                }
                other => panic!("expected Object, got {:?}", other),
            }
        }

        #[test]
        fn test_raw_element_unknown_utf8_fallback() {
            // Use an OID that doesn't match any known type — NAME is handled as TEXT,
            // so use a truly unknown type. The fallback branch tries UTF-8 first.
            let result =
                raw_element_to_const(&postgres_types::Type::REGCLASS, b"some_table").unwrap();
            assert_eq!(result, ConstValue::String("some_table".to_string()));
        }

        // ===== sanitize_graphql_name tests =====

        #[test]
        fn test_sanitize_graphql_name_normal() {
            assert_eq!(sanitize_graphql_name("user_name"), "user_name");
        }

        #[test]
        fn test_sanitize_graphql_name_special_chars() {
            assert_eq!(sanitize_graphql_name("user-name"), "user_name");
        }

        #[test]
        fn test_sanitize_graphql_name_starts_with_digit() {
            assert_eq!(sanitize_graphql_name("1col"), "_1col");
        }

        #[test]
        fn test_sanitize_graphql_name_empty() {
            assert_eq!(sanitize_graphql_name(""), "_unnamed");
        }

        #[test]
        fn test_sanitize_graphql_name_double_underscore() {
            assert_eq!(sanitize_graphql_name("__type"), "___type");
        }

        // ===== Additional binary formatter tests =====

        #[test]
        fn test_format_macaddr8() {
            let raw = [0x08, 0x00, 0x2b, 0x01, 0x02, 0x03, 0x04, 0x05];
            assert_eq!(format_macaddr(&raw), "08:00:2b:01:02:03:04:05");
        }

        #[test]
        fn test_format_interval_negative_time() {
            // -01:30:00 => µs = -5_400_000_000
            let mut raw = Vec::new();
            raw.extend_from_slice(&(-5_400_000_000i64).to_be_bytes());
            raw.extend_from_slice(&0i32.to_be_bytes());
            raw.extend_from_slice(&0i32.to_be_bytes());
            assert_eq!(format_interval(&raw).unwrap(), "-01:30:00");
        }

        #[test]
        fn test_format_timetz_negative_offset() {
            // 15:00:00-05:00
            // µs = 15*3600*1_000_000 = 54_000_000_000
            // PostgreSQL offset = seconds west of UTC = 18000 (since -05 is 5*3600 west)
            let mut raw = Vec::new();
            raw.extend_from_slice(&54_000_000_000i64.to_be_bytes());
            raw.extend_from_slice(&18000i32.to_be_bytes());
            assert_eq!(format_timetz(&raw).unwrap(), "15:00:00-05");
        }

        #[test]
        fn test_format_timetz_with_minutes() {
            // 12:00:00+05:30
            // µs = 12*3600*1_000_000 = 43_200_000_000
            // PostgreSQL offset = -(5*3600 + 30*60) = -19800
            let mut raw = Vec::new();
            raw.extend_from_slice(&43_200_000_000i64.to_be_bytes());
            raw.extend_from_slice(&(-19800i32).to_be_bytes());
            assert_eq!(format_timetz(&raw).unwrap(), "12:00:00+05:30");
        }
    }
}
