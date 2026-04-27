// test_duration_formatter.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::time::Duration;

use crate::fmt::DurationFormatter;

/// Wrapper struct to format a duration with a DurationFormatter via Display
struct FmtHelper
{
	df: DurationFormatter,
	duration: Duration,
}

impl std::fmt::Display for FmtHelper
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		self.df.format_into(self.duration, f)
	}
}

/// Helper: format a duration with the given template string
fn fmt(template: &str, secs: f64) -> String
{
	let df = DurationFormatter::new(template).expect("failed to parse template");
	let duration = Duration::from_secs_f64(secs);
	format!("{}", FmtHelper { df, duration })
}

/// Helper: format a Duration through the Display impl
fn display(secs: u64) -> String
{
	format!("{}", crate::fmt::Duration(Duration::from_secs(secs)))
}

#[test]
fn test_duration_display_zero()
{
	assert_eq!(display(0), "0s");
}

#[test]
fn test_duration_display_seconds_only()
{
	assert_eq!(display(45), "45s");
}

#[test]
fn test_duration_display_minutes_seconds()
{
	assert_eq!(display(125), "2m 5s");
}

#[test]
fn test_duration_display_hours_minutes_seconds()
{
	assert_eq!(display(3661), "1h 1m 1s");
}

#[test]
fn test_duration_display_days()
{
	// 1 day + 3 hours + 41 minutes + 5 seconds = 99665
	assert_eq!(display(99665), "1d 3h 41m 5s");
}

#[test]
fn test_duration_display_exact_day()
{
	assert_eq!(display(86400), "1d");
}

#[test]
fn test_duration_display_exact_hour()
{
	assert_eq!(display(3600), "1h");
}

#[test]
fn test_duration_display_exact_minute()
{
	assert_eq!(display(60), "1m");
}

#[test]
fn test_formatter_hours_minutes_seconds()
{
	// 5445 seconds. Without modulo, each key floors independently:
	// hours = floor(5445/3600) = 1
	// minutes = floor(5445/60) = 90
	// seconds = floor(5445) = 5445
	assert_eq!(fmt("{hours}h {minutes}m {seconds}s", 5445.0), "1h 90m 5445s");
}

#[test]
fn test_formatter_modulo_flag()
{
	// 1h 30m 45s = 5445 seconds
	// with % flag and floor: hours=floor(1.5125)=1, minutes=floor(90.75)=90, 90%60=30, seconds=5445%60=45
	assert_eq!(fmt("{hours:%}h {minutes:%}m {seconds:%}s", 5445.0), "1h 30m 45s");
}

#[test]
fn test_formatter_zero_padded()
{
	assert_eq!(fmt("{hours:%02}:{minutes:%02}:{seconds:%02}", 3661.0), "01:01:01");
}

#[test]
fn test_formatter_zero_duration()
{
	assert_eq!(fmt("{hours}h {minutes}m {seconds}s", 0.0), "0h 0m 0s");
}

#[test]
fn test_formatter_omit_zero_flag()
{
	// 45 seconds → hours=0, minutes=floor(45/60)=0, seconds=45
	// With ? flag, zero values are omitted, but surrounding literals are NOT —
	// the ? flag only suppresses the placeholder value, not adjacent literal text.
	// hours=0 → omitted, minutes=0 → omitted, so we get "h m 45s"
	assert_eq!(fmt("{hours:?}h {minutes:?}m {seconds}s", 45.0), "h m 45s");
}

#[test]
fn test_formatter_omit_zero_partial()
{
	// 3661s = 1h 1m 1s
	// hours=1 (not 0, printed), minutes=61 (not 0, printed), seconds=3661 (not 0, printed)
	assert_eq!(fmt("{hours:?}h {minutes:?}m {seconds}s", 3661.0), "1h 61m 3661s");
}

#[test]
fn test_formatter_omit_zero_with_modulo()
{
	// 3600s = 1h exactly
	// hours%24=1 (printed), minutes%60=0 (omitted), seconds%60=0 (omitted)
	// But surrounding literals "m " and "s" still render (? only omits the value)
	assert_eq!(fmt("{hours:%?}h {minutes:%?}m {seconds:%?}s", 3600.0), "1h m s");
}

#[test]
fn test_formatter_suffix()
{
	assert_eq!(fmt("{hours:@hrs} {minutes:@min}", 3661.0), "1hrs 61min");
}

#[test]
fn test_formatter_suffix_with_format()
{
	assert_eq!(fmt("{hours:%02@h}:{minutes:%02@m}", 3661.0), "01h:01m");
}

#[test]
fn test_formatter_pluralize_flag()
{
	// New behaviour: `s` flag appends 's' AFTER the suffix (if any) when value != 1.
	// The value is always printed.
	// {hours:s} → value=1, no 's' appended → "1"
	assert_eq!(fmt("{hours:s}", 3600.0), "1");
	// {hours:s} → value=2, 's' appended → "2s"
	assert_eq!(fmt("{hours:s}", 7200.0), "2s");
	// {hours:s} → value=0, 's' appended → "0s"
	assert_eq!(fmt("{hours:s}", 0.0), "0s");

	// Combined with suffix: {hours:s@hr} → prints value, then suffix "hr", then 's' if != 1
	assert_eq!(fmt("{hours:s@ hour}", 3600.0), "1 hour"); // value=1 → no 's'
	assert_eq!(fmt("{hours:s@ hour}", 7200.0), "2 hours"); // value=2 → 's' after suffix
	assert_eq!(fmt("{hours:s@ hour}", 0.0), "0 hours"); // value=0 → 's' after suffix
}

#[test]
fn test_formatter_literal_passthrough()
{
	assert_eq!(fmt("elapsed: {seconds}s", 42.0), "elapsed: 42s");
}

#[test]
fn test_formatter_millis()
{
	// 1.5 seconds = 1500ms
	assert_eq!(fmt("{millis}ms", 1.5), "1500ms");
}

#[test]
fn test_formatter_millis_alias()
{
	assert_eq!(fmt("{ms}ms", 1.5), "1500ms");
}

#[test]
fn test_formatter_days_weeks_months_years()
{
	let one_year = 365.0 * 86400.0;
	assert_eq!(fmt("{years}y", one_year), "1y");

	let one_month = 30.0 * 86400.0;
	assert_eq!(fmt("{months}mo", one_month), "1mo");

	let one_week = 7.0 * 86400.0;
	assert_eq!(fmt("{weeks}w", one_week), "1w");

	let one_day = 86400.0;
	assert_eq!(fmt("{days}d", one_day), "1d");
}

#[test]
fn test_formatter_key_aliases()
{
	let secs = 5445.0;
	// hrs == hours, mins == minutes, secs == seconds, ms == millis
	assert_eq!(fmt("{hrs}", secs), fmt("{hours}", secs));
	assert_eq!(fmt("{mins}", secs), fmt("{minutes}", secs));
	assert_eq!(fmt("{secs}", secs), fmt("{seconds}", secs));
	assert_eq!(fmt("{ms}", secs), fmt("{millis}", secs));
}

#[test]
fn test_formatter_invalid_key()
{
	let result = DurationFormatter::new("{invalid_key}");
	assert!(result.is_err());
}

#[test]
fn test_formatter_empty_template()
{
	// Empty template should work and produce empty output
	let df = DurationFormatter::new("").expect("empty should be valid");
	let duration = Duration::from_secs(100);
	let output = format!("{}", FmtHelper { df, duration });
	assert_eq!(output, "");
}

#[test]
fn test_formatter_literal_only()
{
	assert_eq!(fmt("hello world", 100.0), "hello world");
}

#[test]
fn test_formatter_unclosed_brace()
{
	let result = DurationFormatter::new("{hours");
	assert!(result.is_err());
}

#[test]
fn test_formatter_nested_braces_in_suffix()
{
	// The parser supports nested braces in @... args
	// This exercises the nested_brace_count tracking
	assert_eq!(fmt("{hours:@{h}}", 3600.0), "1{h}");
}

// --- hhmmss shorthand tests ---

#[test]
fn test_shorthand_hhmmss_basic()
{
	// 1h 30m 45s = 5445s → hrs=1, mins=90%60=30, secs=5445%60=45
	assert_eq!(fmt("{hhmmss}", 5445.0), "01:30:45");
}

#[test]
fn test_shorthand_hhmmss_zero()
{
	assert_eq!(fmt("{hhmmss}", 0.0), "00:00:00");
}

#[test]
fn test_shorthand_hhmmss_large()
{
	// 100h 5m 3s = 360303s
	// Hours shows total (not modulo'd), mins and secs are modulo'd
	assert_eq!(fmt("{hhmmss}", 360303.0), "100:05:03");
}

#[test]
fn test_shorthand_hhmmss_exact_hour()
{
	assert_eq!(fmt("{hhmmss}", 3600.0), "01:00:00");
}

#[test]
fn test_shorthand_hhmmss_seconds_only()
{
	assert_eq!(fmt("{hhmmss}", 45.0), "00:00:45");
}

#[test]
fn test_shorthand_hhmmss_with_surrounding_text()
{
	// 3661s = 1h 1m 1s
	assert_eq!(fmt("elapsed: {hhmmss}", 3661.0), "elapsed: 01:01:01");
}

#[test]
fn test_shorthand_hhmmss_with_width()
{
	// Shorthand writes to a String first, then formats through the outer formatter,
	// so width/alignment works.
	// 61s = 0h 1m 1s → "00:01:01" (8 chars), right-aligned in width 12
	assert_eq!(fmt("{hhmmss:>12}", 61.0), "    00:01:01");
}

#[test]
fn test_shorthand_hhmmss_with_suffix()
{
	assert_eq!(fmt("{hhmmss:@ elapsed}", 3661.0), "01:01:01 elapsed");
}
