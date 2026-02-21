pub(super) fn format_uuid(raw: &[u8]) -> anyhow::Result<String> {
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

pub(super) fn parse_pg_numeric(raw: &[u8]) -> anyhow::Result<String> {
    if raw.len() < 8 {
        anyhow::bail!("NUMERIC binary too short");
    }
    let ndigits = i16::from_be_bytes([raw[0], raw[1]]) as i32;
    let weight = i16::from_be_bytes([raw[2], raw[3]]) as i32;
    let sign = u16::from_be_bytes([raw[4], raw[5]]);
    let dscale = i16::from_be_bytes([raw[6], raw[7]]) as i32;

    if ndigits < 0 {
        anyhow::bail!("NUMERIC ndigits is negative: {ndigits}");
    }
    if dscale < 0 {
        anyhow::bail!("NUMERIC dscale is negative: {dscale}");
    }

    // NaN
    if sign == 0xC000 {
        return Ok("NaN".to_string());
    }

    // Infinity (PostgreSQL 14+)
    if sign == 0xD000 {
        return Ok("Infinity".to_string());
    }
    if sign == 0xF000 {
        return Ok("-Infinity".to_string());
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
        for d in digits.iter().skip(frac_start) {
            frac.push_str(&format!("{:04}", d));
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

pub(super) fn format_inet(raw: &[u8]) -> anyhow::Result<String> {
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
    } else if family == 3 {
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
    } else {
        anyhow::bail!("Unknown INET address family: {}", family);
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

pub(super) fn format_macaddr(raw: &[u8]) -> anyhow::Result<String> {
    if raw.len() == 8 {
        // MACADDR8
        Ok(format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7]
        ))
    } else if raw.len() >= 6 {
        // MACADDR (6 bytes)
        Ok(format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5]
        ))
    } else {
        anyhow::bail!("macaddr binary too short: {} bytes", raw.len())
    }
}

pub(super) fn format_interval(raw: &[u8]) -> anyhow::Result<String> {
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

pub(super) fn format_timetz(raw: &[u8]) -> anyhow::Result<String> {
    if raw.len() < 12 {
        anyhow::bail!("TIMETZ binary too short");
    }
    let microseconds = i64::from_be_bytes(raw[0..8].try_into()?);
    let offset_secs = i32::from_be_bytes(raw[8..12].try_into()?);

    if microseconds < 0 {
        anyhow::bail!("TIMETZ microseconds value is negative: {microseconds}");
    }

    let total_secs = microseconds / 1_000_000;
    let us_remainder = microseconds % 1_000_000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    // PostgreSQL stores offset as seconds west of UTC (negated from usual
    // convention).
    // offset_secs <= 0 means east of UTC (+ sign), > 0 means west (- sign)
    let tz_sign = if offset_secs <= 0 { '+' } else { '-' };
    let tz_abs = offset_secs.unsigned_abs(); // no overflow even for i32::MIN
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

pub(super) fn format_bytea(raw: &[u8]) -> String {
    let mut s = String::with_capacity(2 + raw.len() * 2);
    s.push_str("\\x");
    for b in raw {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parse_pg_numeric_infinity() {
        let mut raw = Vec::new();
        raw.extend_from_slice(&0i16.to_be_bytes());
        raw.extend_from_slice(&0i16.to_be_bytes());
        raw.extend_from_slice(&0xD000u16.to_be_bytes());
        raw.extend_from_slice(&0i16.to_be_bytes());
        assert_eq!(parse_pg_numeric(&raw).unwrap(), "Infinity");
    }

    #[test]
    fn test_parse_pg_numeric_neg_infinity() {
        let mut raw = Vec::new();
        raw.extend_from_slice(&0i16.to_be_bytes());
        raw.extend_from_slice(&0i16.to_be_bytes());
        raw.extend_from_slice(&0xF000u16.to_be_bytes());
        raw.extend_from_slice(&0i16.to_be_bytes());
        assert_eq!(parse_pg_numeric(&raw).unwrap(), "-Infinity");
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
        assert_eq!(format_macaddr(&raw).unwrap(), "08:00:2b:01:02:03");
    }

    #[test]
    fn test_format_interval_day_and_time() {
        // 1 day 02:30:00 => us for 2h30m = 9_000_000_000, days=1, months=0
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

    #[test]
    fn test_compress_ipv6() {
        assert_eq!(compress_ipv6("0:0:0:0:0:0:0:1"), "::1");
        assert_eq!(compress_ipv6("2001:db8:0:0:0:0:0:1"), "2001:db8::1");
        assert_eq!(compress_ipv6("0:0:0:0:0:0:0:0"), "::");
    }

    #[test]
    fn test_format_macaddr8() {
        let raw = [0x08, 0x00, 0x2b, 0x01, 0x02, 0x03, 0x04, 0x05];
        assert_eq!(format_macaddr(&raw).unwrap(), "08:00:2b:01:02:03:04:05");
    }

    #[test]
    fn test_format_interval_negative_time() {
        // -01:30:00 => us = -5_400_000_000
        let mut raw = Vec::new();
        raw.extend_from_slice(&(-5_400_000_000i64).to_be_bytes());
        raw.extend_from_slice(&0i32.to_be_bytes());
        raw.extend_from_slice(&0i32.to_be_bytes());
        assert_eq!(format_interval(&raw).unwrap(), "-01:30:00");
    }

    #[test]
    fn test_format_timetz_negative_offset() {
        // 15:00:00-05:00
        let mut raw = Vec::new();
        raw.extend_from_slice(&54_000_000_000i64.to_be_bytes());
        raw.extend_from_slice(&18000i32.to_be_bytes());
        assert_eq!(format_timetz(&raw).unwrap(), "15:00:00-05");
    }

    #[test]
    fn test_format_timetz_with_minutes() {
        // 12:00:00+05:30
        let mut raw = Vec::new();
        raw.extend_from_slice(&43_200_000_000i64.to_be_bytes());
        raw.extend_from_slice(&(-19800i32).to_be_bytes());
        assert_eq!(format_timetz(&raw).unwrap(), "12:00:00+05:30");
    }

    #[test]
    fn test_format_timetz_offset_secs_min() {
        let mut raw = Vec::new();
        raw.extend_from_slice(&0i64.to_be_bytes());
        raw.extend_from_slice(&i32::MIN.to_be_bytes());
        assert!(format_timetz(&raw).is_ok());
    }

    #[test]
    fn test_format_uuid_short_input() {
        let raw = [0u8; 15];
        assert!(format_uuid(&raw).is_err());
    }

    #[test]
    fn test_format_inet_short_input() {
        let raw = [2u8, 32, 0]; // only 3 bytes, needs at least 4
        assert!(format_inet(&raw).is_err());
    }

    #[test]
    fn test_format_interval_short_input() {
        let raw = [0u8; 15]; // needs at least 16
        assert!(format_interval(&raw).is_err());
    }

    #[test]
    fn test_parse_pg_numeric_short_input() {
        let raw = [0u8; 7]; // needs at least 8
        assert!(parse_pg_numeric(&raw).is_err());
    }

    #[test]
    fn test_parse_pg_numeric_negative_ndigits() {
        let mut raw = Vec::new();
        raw.extend_from_slice(&(-1i16).to_be_bytes()); // ndigits = -1
        raw.extend_from_slice(&0i16.to_be_bytes()); // weight
        raw.extend_from_slice(&0u16.to_be_bytes()); // sign
        raw.extend_from_slice(&0i16.to_be_bytes()); // dscale
        assert!(parse_pg_numeric(&raw).is_err());
    }
}
