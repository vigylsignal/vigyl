//! Vixie / POSIX 5-field cron parser.
//!
//! The parser is deliberately independent from any crate ecosystem so the same
//! logic can be audited by hand and mirrored in TypeScript. Only the shapes we
//! actually need are supported: `*`, integer atoms, ranges (`a-b`), lists
//! (`a,b,c`), and step (`*/k` or `a-b/k`).

use std::collections::BTreeSet;

use crate::{Result, VigylError};

const FIELD_RANGES: [(u8, u8); 5] = [
    (0, 59),  // minute
    (0, 23),  // hour
    (1, 31),  // day-of-month
    (1, 12),  // month
    (0, 7),   // day-of-week (0 and 7 both mean Sunday)
];

const DAY_ALIASES: [(&str, u8); 7] = [
    ("SUN", 0), ("MON", 1), ("TUE", 2), ("WED", 3),
    ("THU", 4), ("FRI", 5), ("SAT", 6),
];

const MONTH_ALIASES: [(&str, u8); 12] = [
    ("JAN", 1), ("FEB", 2), ("MAR", 3), ("APR", 4), ("MAY", 5), ("JUN", 6),
    ("JUL", 7), ("AUG", 8), ("SEP", 9), ("OCT", 10), ("NOV", 11), ("DEC", 12),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CronSchedule {
    pub expression: String,
    pub minutes: BTreeSet<u8>,
    pub hours: BTreeSet<u8>,
    pub days: BTreeSet<u8>,
    pub months: BTreeSet<u8>,
    pub weekdays: BTreeSet<u8>,
}

impl CronSchedule {
    pub fn parse(expr: &str) -> Result<Self> {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return Err(VigylError::InvalidCron("empty expression".into()));
        }
        if trimmed.len() > 40 {
            return Err(VigylError::InvalidCron(
                "expression exceeds 40 characters (on-chain buffer)".into(),
            ));
        }
        let parts: Vec<&str> = trimmed.split_ascii_whitespace().collect();
        if parts.len() != 5 {
            return Err(VigylError::InvalidCron(format!(
                "expected 5 fields, got {}",
                parts.len()
            )));
        }
        Ok(Self {
            expression: trimmed.to_string(),
            minutes: expand(parts[0], 0)?,
            hours: expand(parts[1], 1)?,
            days: expand(parts[2], 2)?,
            months: expand(parts[3], 3)?,
            weekdays: normalise_weekdays(expand(parts[4], 4)?),
        })
    }

    /// How many minute ticks per day fire.
    pub fn executions_per_day(&self) -> u64 {
        // upper bound: minutes * hours * (weekday coverage share). we keep it simple:
        // treat weekdays / month / day fields as active if the minute field is *.
        let per_day = self.minutes.len() as u64 * self.hours.len() as u64;
        per_day
    }

    /// Whether the schedule fires at the given `(minute, hour, day, month, weekday)`.
    pub fn matches(&self, minute: u8, hour: u8, day: u8, month: u8, weekday: u8) -> bool {
        self.minutes.contains(&minute)
            && self.hours.contains(&hour)
            && self.days.contains(&day)
            && self.months.contains(&month)
            && self.weekdays.contains(&weekday)
    }
}

fn normalise_weekdays(set: BTreeSet<u8>) -> BTreeSet<u8> {
    set.into_iter().map(|v| if v == 7 { 0 } else { v }).collect()
}

fn expand(field: &str, index: usize) -> Result<BTreeSet<u8>> {
    let (min, max) = FIELD_RANGES[index];
    let mut out = BTreeSet::new();
    for part in field.split(',') {
        let (range_part, step_part) = match part.split_once('/') {
            Some((r, s)) => (r, Some(s)),
            None => (part, None),
        };
        let step: u8 = step_part.map(parse_number).transpose()?.unwrap_or(1);
        if step == 0 {
            return Err(VigylError::InvalidCron(format!(
                "step must be > 0 at field {}",
                index + 1
            )));
        }
        let (start, end) = if range_part == "*" || range_part.is_empty() {
            (min, max)
        } else if let Some((a, b)) = range_part.split_once('-') {
            (parse_atom(a, index)?, parse_atom(b, index)?)
        } else {
            let n = parse_atom(range_part, index)?;
            (n, n)
        };
        if start > end || start < min || end > max {
            return Err(VigylError::InvalidCron(format!(
                "value out of range at field {}",
                index + 1
            )));
        }
        let mut v = start;
        while v <= end {
            out.insert(v);
            let Some(next) = v.checked_add(step) else { break };
            v = next;
        }
    }
    if out.is_empty() {
        return Err(VigylError::InvalidCron(format!(
            "empty set at field {}",
            index + 1
        )));
    }
    Ok(out)
}

fn parse_number(s: &str) -> Result<u8> {
    s.trim()
        .parse::<u8>()
        .map_err(|_| VigylError::InvalidCron(format!("invalid number \"{s}\"")))
}

fn parse_atom(s: &str, index: usize) -> Result<u8> {
    let up = s.trim().to_ascii_uppercase();
    if index == 3 {
        if let Some((_, v)) = MONTH_ALIASES.iter().find(|(k, _)| *k == up.as_str()) {
            return Ok(*v);
        }
    }
    if index == 4 {
        if let Some((_, v)) = DAY_ALIASES.iter().find(|(k, _)| *k == up.as_str()) {
            return Ok(*v);
        }
    }
    parse_number(&up)
}

/// Encode a cron trigger into the on-chain 128-byte buffer.
///
/// Layout (see docs/anchor-spec.md §2.1):
/// * bytes 0..40   -- ASCII expression (padded with 0)
/// * bytes 40..48  -- i64 little-endian timezone offset seconds
/// * bytes 48..128 -- reserved
pub fn encode_cron(expression: &str, tz_offset_seconds: i64) -> Result<[u8; 128]> {
    CronSchedule::parse(expression)?; // validates length + syntax
    let mut buf = [0u8; 128];
    let bytes = expression.trim().as_bytes();
    if bytes.len() > 40 {
        return Err(VigylError::InvalidCron("expression > 40 bytes".into()));
    }
    buf[..bytes.len()].copy_from_slice(bytes);
    buf[40..48].copy_from_slice(&tz_offset_seconds.to_le_bytes());
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hourly_expands() {
        let s = CronSchedule::parse("0 * * * *").unwrap();
        assert_eq!(s.minutes.len(), 1);
        assert_eq!(s.hours.len(), 24);
        assert!(s.matches(0, 5, 1, 1, 3));
        assert!(!s.matches(1, 5, 1, 1, 3));
    }

    #[test]
    fn every_five_minutes() {
        let s = CronSchedule::parse("*/5 * * * *").unwrap();
        assert_eq!(s.minutes.len(), 12);
        assert!(s.matches(0, 0, 1, 1, 0));
        assert!(s.matches(55, 0, 1, 1, 0));
        assert!(!s.matches(3, 0, 1, 1, 0));
    }

    #[test]
    fn weekday_alias_normalised() {
        let s = CronSchedule::parse("0 9 * * MON-FRI").unwrap();
        assert!(s.matches(0, 9, 1, 1, 1));
        assert!(s.matches(0, 9, 1, 1, 5));
        assert!(!s.matches(0, 9, 1, 1, 6));
    }

    #[test]
    fn seven_normalises_to_sunday() {
        let s = CronSchedule::parse("0 * * * 7").unwrap();
        assert!(s.matches(0, 12, 1, 1, 0));
    }

    #[test]
    fn oversized_expression_rejected() {
        let long = "0 0 1 1 * # some extremely long extra content over budget";
        assert!(CronSchedule::parse(long).is_err());
    }

    #[test]
    fn encoded_length_is_128() {
        let buf = encode_cron("0 * * * *", 0).unwrap();
        assert_eq!(buf.len(), 128);
        assert_eq!(&buf[..9], b"0 * * * *");
    }
}
