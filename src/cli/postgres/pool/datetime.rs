pub(super) fn parse_naive_datetime(
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

pub(super) fn parse_naive_time(
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

pub(super) fn parse_uuid_to_bytes(
    s: &str,
) -> Result<[u8; 16], Box<dyn std::error::Error + Sync + Send>> {
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_parse_uuid_to_bytes_valid() {
        let bytes = parse_uuid_to_bytes("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            bytes,
            [
                0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
                0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_parse_uuid_to_bytes_no_hyphens() {
        let bytes = parse_uuid_to_bytes("550e8400e29b41d4a716446655440000").unwrap();
        assert_eq!(
            bytes,
            [
                0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
                0x00, 0x00
            ]
        );
    }

    #[test]
    fn test_parse_uuid_to_bytes_invalid() {
        assert!(parse_uuid_to_bytes("invalid-uuid").is_err());
    }
}
