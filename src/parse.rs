// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{bail, Result};

/// Parse a human-friendly duration string into seconds.
/// Supports: "2h", "30m", "1h30m", "3600s", "1h30m45s"
pub fn parse_duration(s: &str) -> Result<i32> {
    let s = s.trim().to_ascii_lowercase();
    if s.is_empty() {
        bail!("empty duration string");
    }

    let mut total_secs: i64 = 0;
    let mut num_buf = String::new();
    let mut found_unit = false;

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            num_buf.push(ch);
        } else {
            if num_buf.is_empty() {
                bail!("unexpected '{ch}' in duration: \"{s}\"");
            }
            let n: i64 = num_buf.parse()?;
            num_buf.clear();
            found_unit = true;
            match ch {
                'h' => total_secs += n * 3600,
                'm' => total_secs += n * 60,
                's' => total_secs += n,
                _ => bail!("unknown duration unit '{ch}' in \"{s}\"; use h/m/s"),
            }
        }
    }

    // If we have leftover digits and no unit was ever found, try plain seconds
    if !num_buf.is_empty() {
        if found_unit {
            bail!("trailing digits without unit in \"{s}\"");
        }
        total_secs = num_buf.parse()?;
    }

    i32::try_from(total_secs).map_err(|_| anyhow::anyhow!("duration too large: {total_secs}s"))
}

/// Parse a human-friendly memory string into megabytes.
/// Supports: "16GB", "4096MB", "16gb", "4096mb"
pub fn parse_memory(s: &str) -> Result<i32> {
    let s = s.trim();
    if s.is_empty() {
        bail!("empty memory string");
    }

    let lower = s.to_ascii_lowercase();

    if let Some(gb) = lower.strip_suffix("gb") {
        let n: i64 = gb
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid memory: \"{s}\""))?;
        return i32::try_from(n * 1024).map_err(|_| anyhow::anyhow!("memory too large: \"{s}\""));
    }

    if let Some(mb) = lower.strip_suffix("mb") {
        let n: i32 = mb
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid memory: \"{s}\""))?;
        return Ok(n);
    }

    bail!("unknown memory unit in \"{s}\"; use GB or MB (e.g., \"16GB\", \"4096MB\")");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("2h").unwrap(), 7200);
    }

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("30m").unwrap(), 1800);
    }

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("3600s").unwrap(), 3600);
    }

    #[test]
    fn test_parse_duration_combined() {
        assert_eq!(parse_duration("1h30m").unwrap(), 5400);
    }

    #[test]
    fn test_parse_duration_full_combo() {
        assert_eq!(parse_duration("1h30m45s").unwrap(), 5445);
    }

    #[test]
    fn test_parse_duration_plain_number() {
        assert_eq!(parse_duration("86400").unwrap(), 86400);
    }

    #[test]
    fn test_parse_duration_empty() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn test_parse_duration_bad_unit() {
        assert!(parse_duration("5x").is_err());
    }

    #[test]
    fn test_parse_memory_gb() {
        assert_eq!(parse_memory("16GB").unwrap(), 16384);
    }

    #[test]
    fn test_parse_memory_gb_lowercase() {
        assert_eq!(parse_memory("4gb").unwrap(), 4096);
    }

    #[test]
    fn test_parse_memory_mb() {
        assert_eq!(parse_memory("4096MB").unwrap(), 4096);
    }

    #[test]
    fn test_parse_memory_mb_lowercase() {
        assert_eq!(parse_memory("512mb").unwrap(), 512);
    }

    #[test]
    fn test_parse_memory_empty() {
        assert!(parse_memory("").is_err());
    }

    #[test]
    fn test_parse_memory_no_unit() {
        assert!(parse_memory("1024").is_err());
    }
}
