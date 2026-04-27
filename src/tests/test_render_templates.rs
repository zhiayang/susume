// test_render_templates.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use crate::ProgressBar;
use crate::RenderTarget;


/// Helper: render a bar to a string target and return trimmed output
fn render_bar(bar: &ProgressBar) -> String
{
	let target = RenderTarget::string();
	bar.render(&target);
	return target.get_string().unwrap().trim().to_string();
}

#[test]
fn test_render_position_bytes()
{
	let bar = ProgressBar::new("", Some(1_000_000))
		.with_template("{pos:@bytes}")
		.with_progress_bar_chars(&["=", "-"]);

	bar.set_position(1024);
	assert_eq!(render_bar(&bar), "1.00 kiB");
}

#[test]
fn test_render_position_decimal_bytes()
{
	// @decimal uses decimal scale (1000) but ibibytes prefix is on by default
	let bar = ProgressBar::new("", Some(1_000_000))
		.with_template("{pos:@decimal}")
		.with_progress_bar_chars(&["=", "-"]);

	bar.set_position(1000);
	assert_eq!(render_bar(&bar), "1.00 kiB");
}

#[test]
fn test_render_total_bytes()
{
	let bar = ProgressBar::new("", Some(1_048_576))
		.with_template("{total:@bytes}")
		.with_progress_bar_chars(&["=", "-"]);

	assert_eq!(render_bar(&bar), "1.00 MiB");
}

#[test]
fn test_render_position_and_total_bytes()
{
	let bar = ProgressBar::new("", Some(1_048_576))
		.with_template("{pos:@bytes}/{total:@bytes}")
		.with_progress_bar_chars(&["=", "-"]);

	bar.set_position(524_288);
	assert_eq!(render_bar(&bar), "512.00 kiB/1.00 MiB");
}

#[test]
fn test_render_bytes_alt_flag()
{
	// Alt flag (#) disables ibibytes and space: "kiB" → "kB", removes space
	let bar = ProgressBar::new("", Some(1_048_576))
		.with_template("{pos:#@bytes}")
		.with_progress_bar_chars(&["=", "-"]);

	bar.set_position(1024);
	assert_eq!(render_bar(&bar), "1.00kB");
}

#[test]
fn test_render_message()
{
	let bar = ProgressBar::new("hello world", Some(100)).with_template("{msg}");
	assert_eq!(render_bar(&bar), "hello world");
}

#[test]
fn test_render_message_update()
{
	let bar = ProgressBar::new("first", Some(100)).with_template("{msg}");
	assert_eq!(render_bar(&bar), "first");

	bar.set_message("second".to_string());
	assert_eq!(render_bar(&bar), "second");
}

#[test]
fn test_render_percent()
{
	let bar = ProgressBar::new("", Some(200)).with_template("{percent:.0}%");
	bar.set_position(100);
	assert_eq!(render_bar(&bar), "50%");
}

#[test]
fn test_render_percent_full()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{percent:.0}%");
	bar.set_position(100);
	assert_eq!(render_bar(&bar), "100%");
}

#[test]
fn test_render_percent_zero()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{percent:.0}%");
	assert_eq!(render_bar(&bar), "0%");
}

#[test]
fn test_render_position_total()
{
	let bar = ProgressBar::new("", Some(500)).with_template("{pos}/{len}");
	bar.set_position(123);
	assert_eq!(render_bar(&bar), "123/500");
}

#[test]
fn test_render_position_no_total()
{
	// When total is None, {len} should produce nothing
	let bar = ProgressBar::new("", None).with_template("{pos}/{len}");
	bar.set_position(42);
	assert_eq!(render_bar(&bar), "42/");
}

#[test]
fn test_render_position_formatted()
{
	let bar = ProgressBar::new("", Some(1000)).with_template("{pos:05}");
	bar.set_position(42);
	assert_eq!(render_bar(&bar), "00042");
}

#[test]
fn test_render_key_aliases()
{
	let bar = ProgressBar::new("hello", Some(100)).with_template("{message}");
	assert_eq!(render_bar(&bar), "hello");

	let bar = ProgressBar::new("hello", Some(100)).with_template("{description}");
	assert_eq!(render_bar(&bar), "hello");

	let bar = ProgressBar::new("", Some(100)).with_template("{position}");
	bar.set_position(42);
	assert_eq!(render_bar(&bar), "42");

	let bar = ProgressBar::new("", Some(100)).with_template("{length}");
	assert_eq!(render_bar(&bar), "100");
}

#[test]
fn test_render_elapsed_default_format()
{
	// The elapsed time will be very small since we render immediately
	let bar = ProgressBar::new("", Some(100)).with_template("{elapsed}");
	let output = render_bar(&bar);
	// Should produce a valid duration string (e.g. "0s")
	assert!(output.contains('s'), "elapsed should contain 's', got: {output}");
}

#[test]
fn test_render_elapsed_custom_format()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{elapsed:@{hours:%02}:{minutes:%02}:{seconds:%02}}");
	let output = render_bar(&bar);
	// Should produce "00:00:00" (or close to it, depending on timing)
	assert_eq!(output.len(), 8, "expected HH:MM:SS format, got: {output}");
	assert_eq!(&output[2..3], ":");
	assert_eq!(&output[5..6], ":");
}

#[test]
fn test_render_remaining_no_total()
{
	// Without a total, remaining time should produce nothing
	let bar = ProgressBar::new("", None).with_template("[{remaining}]");
	assert_eq!(render_bar(&bar), "[]");
}

#[test]
fn test_render_remaining_with_total()
{
	// With a total and a default estimator, should produce some duration string
	let bar = ProgressBar::new("", Some(100)).with_template("{remaining}");
	let output = render_bar(&bar);
	// The estimator returns 0 at startup → rate fallback to 0.01 → remaining = 100/0.01 = 10000s
	assert!(
		output.contains('s') || output.contains('m') || output.contains('h') || output.contains('d'),
		"remaining should contain a time unit, got: {output}"
	);
}

#[test]
fn test_render_rate()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{rate:.2}");
	let output = render_bar(&bar);
	// At startup, rate should be 0
	assert!(output.contains("0"), "rate should be 0 at startup, got: {output}");
}

#[test]
fn test_render_rate_bytes()
{
	let bar = ProgressBar::new("", Some(1_000_000)).with_template("{rate:@bytes}");
	let output = render_bar(&bar);
	// At startup, rate is 0 → ByteSize(0) → "0.00 B" or similar
	assert!(output.contains("B"), "rate @bytes should contain 'B', got: {output}");
}

#[test]
fn test_render_padding()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{msg}{pad:.<20}{pos}");
	bar.set_position(5);
	// Message is empty, padding fills 20 chars with '.', then position
	let output = render_bar(&bar);
	assert!(
		output.contains("...................."),
		"padding should contain dots, got: {output}"
	);
	assert!(output.ends_with("5"));
}

#[test]
fn test_increment_decrement()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{pos}/{len}");

	bar.increment(10);
	assert_eq!(render_bar(&bar), "10/100");

	bar.increment(5);
	assert_eq!(render_bar(&bar), "15/100");

	bar.decrement(3);
	assert_eq!(render_bar(&bar), "12/100");
}

#[test]
fn test_set_position()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{pos}");

	bar.set_position(42);
	assert_eq!(render_bar(&bar), "42");

	bar.set_position(99);
	assert_eq!(render_bar(&bar), "99");
}

#[test]
fn test_decrement_below_zero_saturates()
{
	let bar = ProgressBar::new("", Some(100)).with_template("{pos}");

	bar.set_position(5);
	bar.decrement(10); // should saturate at 0
	assert_eq!(render_bar(&bar), "0");
}

#[test]
fn test_multiple_renders_to_string_target()
{
	let target = RenderTarget::string();
	let bar = ProgressBar::new("", Some(100)).with_template("{pos}");

	bar.set_position(10);
	bar.render(&target);

	bar.set_position(20);
	bar.render(&target);

	// String target doesn't have cursor management — it just accumulates
	// (reset clears the string, actually)
	let output = target.get_string().unwrap();
	// After reset + re-render, should show only the latest
	assert!(output.contains("20"));
}

#[test]
fn test_deferred_bar_fills_remaining()
{
	// The bar should expand to fill remaining width after other parts are rendered
	let bar = ProgressBar::new("x", Some(100))
		.with_template("{msg} [{bar:50!}]")
		.with_progress_bar_chars(&["=", "-"]);

	bar.set_position(50);
	let output = render_bar(&bar);
	// Should contain 'x' then the bar
	assert!(output.starts_with("x ["));
	assert!(output.ends_with(']'));
	// Bar should be exactly 50 chars of = and -
	let bar_content: String = output.chars().skip(3).take_while(|c| *c != ']').collect();
	assert_eq!(bar_content.len(), 50);
}
