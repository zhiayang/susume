// test_parser.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Alignment;

use crate::fmt::ParseOptions;
use crate::fmt::TemplatePart;
use crate::fmt::WidthPrecisionSpec;
use crate::fmt::parse_template;

#[test]
fn test_parse_literal_only()
{
	let parts = parse_template("hello world", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	assert_eq!(parts.len(), 1);
	assert!(matches!(&parts[0], TemplatePart::Literal(s) if s == "hello world"));
}

#[test]
fn test_parse_empty()
{
	let parts = parse_template("", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	assert_eq!(parts.len(), 0);
}

#[test]
fn test_parse_simple_placeholder()
{
	let parts = parse_template("{foo}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	assert_eq!(parts.len(), 1);
	match &parts[0] {
		TemplatePart::Placeholder { key, .. } => assert_eq!(key, "foo"),
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_placeholder_with_literal_around()
{
	let parts = parse_template("before {key} after", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	assert_eq!(parts.len(), 3);
	assert!(matches!(&parts[0], TemplatePart::Literal(s) if s == "before "));
	assert!(matches!(&parts[1], TemplatePart::Placeholder { key, .. } if key == "key"));
	assert!(matches!(&parts[2], TemplatePart::Literal(s) if s == " after"));
}

#[test]
fn test_parse_multiple_placeholders()
{
	let parts = parse_template("{a}/{b}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	assert_eq!(parts.len(), 3);
	assert!(matches!(&parts[0], TemplatePart::Placeholder { key, .. } if key == "a"));
	assert!(matches!(&parts[1], TemplatePart::Literal(s) if s == "/"));
	assert!(matches!(&parts[2], TemplatePart::Placeholder { key, .. } if key == "b"));
}

#[test]
fn test_parse_escaped_open_brace()
{
	let parts = parse_template("{{hello}}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	assert_eq!(parts.len(), 1);
	assert!(matches!(&parts[0], TemplatePart::Literal(s) if s == "{hello}"));
}

#[test]
fn test_parse_escaped_braces_mixed()
{
	let parts = parse_template("{{}} and {key}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	assert_eq!(parts.len(), 2);
	assert!(matches!(&parts[0], TemplatePart::Literal(s) if s == "{} and "));
	assert!(matches!(&parts[1], TemplatePart::Placeholder { key, .. } if key == "key"));
}

#[test]
fn test_parse_width()
{
	let parts = parse_template("{foo:10}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { width, .. } => {
			assert_eq!(*width, Some(WidthPrecisionSpec::Absolute(10)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_precision()
{
	let parts = parse_template("{foo:.5}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { precision, .. } => {
			assert_eq!(*precision, Some(WidthPrecisionSpec::Absolute(5)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_width_and_precision()
{
	let parts = parse_template("{foo:10.5}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { width, precision, .. } => {
			assert_eq!(*width, Some(WidthPrecisionSpec::Absolute(10)));
			assert_eq!(*precision, Some(WidthPrecisionSpec::Absolute(5)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_alignment()
{
	let parts = parse_template("{foo:<10}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { align, width, .. } => {
			assert_eq!(*align, Some(Alignment::Left));
			assert_eq!(*width, Some(WidthPrecisionSpec::Absolute(10)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_center_alignment()
{
	let parts = parse_template("{foo:^10}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { align, .. } => {
			assert_eq!(*align, Some(Alignment::Center));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_fill_and_alignment()
{
	let parts = parse_template("{foo:-<10}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { fill, align, width, .. } => {
			assert_eq!(*fill, Some('-'));
			assert_eq!(*align, Some(Alignment::Left));
			assert_eq!(*width, Some(WidthPrecisionSpec::Absolute(10)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_sign_plus()
{
	let parts = parse_template("{foo:+}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { sign, .. } => {
			assert_eq!(*sign, Some('+'));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_zero_pad()
{
	let parts = parse_template("{foo:05}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { zero, width, .. } => {
			assert_eq!(*zero, true);
			assert_eq!(*width, Some(WidthPrecisionSpec::Absolute(5)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_alt_flag()
{
	let parts = parse_template("{foo:#}", ParseOptions::<fn(char) -> bool>::default()).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { alt, .. } => {
			assert_eq!(*alt, true);
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_percent_absolute_width()
{
	let opts = ParseOptions {
		relative_width: true,
		..ParseOptions::<fn(char) -> bool>::default()
	};
	let parts = parse_template("{foo:30%}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { width, .. } => {
			assert_eq!(*width, Some(WidthPrecisionSpec::PercentAbsolute(30)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_percent_relative_width()
{
	let opts = ParseOptions {
		relative_width: true,
		..ParseOptions::<fn(char) -> bool>::default()
	};
	let parts = parse_template("{foo:100%%}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { width, .. } => {
			assert_eq!(*width, Some(WidthPrecisionSpec::PercentRelative(100)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_percent_not_allowed_without_option()
{
	// Without relative_width, % should be an error
	let result = parse_template("{foo:30%}", ParseOptions::<fn(char) -> bool>::default());
	assert!(result.is_err());
}

#[test]
fn test_parse_deferred()
{
	let opts = ParseOptions { defer: true, ..ParseOptions::<fn(char) -> bool>::default() };
	let parts = parse_template("{foo:10!}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { deferred, width, .. } => {
			assert_eq!(*deferred, 1);
			assert_eq!(*width, Some(WidthPrecisionSpec::Absolute(10)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_double_deferred()
{
	let opts = ParseOptions { defer: true, ..ParseOptions::<fn(char) -> bool>::default() };
	let parts = parse_template("{foo:!!}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { deferred, .. } => {
			assert_eq!(*deferred, 2);
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_defer_not_allowed_without_option()
{
	let result = parse_template("{foo:10!}", ParseOptions::<fn(char) -> bool>::default());
	assert!(result.is_err());
}

#[test]
fn test_parse_extra_args()
{
	let opts = ParseOptions {
		extra_args: true,
		..ParseOptions::<fn(char) -> bool>::default()
	};
	let parts = parse_template("{foo:@bytes}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { extra_args, .. } => {
			assert_eq!(extra_args.as_deref(), Some("bytes"));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_extra_args_with_nested_braces()
{
	let opts = ParseOptions {
		extra_args: true,
		..ParseOptions::<fn(char) -> bool>::default()
	};
	let parts = parse_template("{foo:@{bar:02}}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { extra_args, .. } => {
			assert_eq!(extra_args.as_deref(), Some("{bar:02}"));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_extra_args_not_allowed_without_option()
{
	let result = parse_template("{foo:@bytes}", ParseOptions::<fn(char) -> bool>::default());
	assert!(result.is_err());
}

#[test]
fn test_parse_extra_flags()
{
	let opts = ParseOptions {
		extra_args: false,
		relative_width: false,
		defer: false,
		style: false,
		flag_handler: Some(|c: char| c == '%' || c == '?' || c == 's'),
	};
	let parts = parse_template("{foo:%?02}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { extra_flags, zero, width, .. } => {
			assert!(extra_flags.contains(&'%'));
			assert!(extra_flags.contains(&'?'));
			assert_eq!(*zero, true);
			assert_eq!(*width, Some(WidthPrecisionSpec::Absolute(2)));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_unknown_flag_rejected()
{
	let result = parse_template("{foo:X}", ParseOptions::<fn(char) -> bool>::default());
	assert!(result.is_err());
}

#[test]
fn test_parse_empty_placeholder()
{
	let result = parse_template("{}", ParseOptions::<fn(char) -> bool>::default());
	assert!(result.is_err());
}

#[test]
fn test_parse_empty_placeholder_name_with_colon()
{
	let result = parse_template("{:10}", ParseOptions::<fn(char) -> bool>::default());
	assert!(result.is_err());
}

#[test]
fn test_parse_lone_close_brace()
{
	let result = parse_template("hello } world", ParseOptions::<fn(char) -> bool>::default());
	assert!(result.is_err());
}

#[test]
fn test_resolve_absolute()
{
	assert_eq!(WidthPrecisionSpec::Absolute(42).resolve(200, 100), 42);
}

#[test]
fn test_resolve_percent_absolute()
{
	// 30% of total_width=200 = 60
	assert_eq!(WidthPrecisionSpec::PercentAbsolute(30).resolve(200, 100), 60);
}

#[test]
fn test_resolve_percent_relative()
{
	// 50% of avail_width=100 = 50
	assert_eq!(WidthPrecisionSpec::PercentRelative(50).resolve(200, 100), 50);
}

#[test]
fn test_parse_full_spec()
{
	let opts = ParseOptions {
		relative_width: true,
		defer: true,
		style: false,
		extra_args: true,
		flag_handler: Some(|_: char| false),
	};
	let parts = parse_template("{bar:40%!@bytes}", opts).unwrap();
	match &parts[0] {
		TemplatePart::Placeholder { key, width, deferred, extra_args, .. } => {
			assert_eq!(key, "bar");
			assert_eq!(*width, Some(WidthPrecisionSpec::PercentAbsolute(40)));
			assert_eq!(*deferred, 1);
			assert_eq!(extra_args.as_deref(), Some("bytes"));
		}
		_ => panic!("expected placeholder"),
	}
}

#[test]
fn test_parse_part_idx_increments()
{
	let opts = ParseOptions::<fn(char) -> bool>::default();
	let parts = parse_template("before {a} middle {b} after", opts).unwrap();
	// parts: Literal("before "), Placeholder(a, idx=1), Literal(" middle "), Placeholder(b, idx=3), Literal("
	// after") part_idx is parts.len() at time of push, so first placeholder gets idx=1 (after the literal),
	// second gets idx=3
	match &parts[1] {
		TemplatePart::Placeholder { part_idx, key, .. } => {
			assert_eq!(key, "a");
			assert_eq!(*part_idx, 1);
		}
		_ => panic!("expected placeholder"),
	}
	match &parts[3] {
		TemplatePart::Placeholder { part_idx, key, .. } => {
			assert_eq!(key, "b");
			assert_eq!(*part_idx, 3);
		}
		_ => panic!("expected placeholder"),
	}
}
