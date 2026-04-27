// duration.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::FormattingOptions;
use std::fmt::Write;
use std::sync::LazyLock;
use std::time::Duration as StdDuration;
use std::time::SystemTime;

use crate::fmt;
use crate::fmt::ParseOptions;
use crate::fmt::TemplateError;
use crate::fmt::TemplatePart;

/// A wrapper around a `SystemTime` that prints the delta in human-readable format,
/// for example `1d 3h 41m`.
///
/// Both positive (future) and negative (past) deltas are
/// allowed, with the same output for either case.
#[derive(Debug, Clone, Copy)]
pub struct RelativeTime(pub SystemTime);

/// A wrapper around a `Duration` that formats in a human-readable manner, for
/// example `4m 31s`.
#[derive(Debug, Clone, Copy)]
pub struct Duration(pub StdDuration);

/// A formatter for a [`std::time::Duration`] that allows printing the duration with
/// a set of user-provided specifiers, for example: `{years}` for years,
/// `{hours:%02}:{minutes:%02}` for hours and minutes, etc.
///
/// See [`DurationFormatter::new`] for the specific specifiers supported.
#[derive(Debug, Clone)]
pub struct DurationFormatter(Vec<TemplatePart>);


impl Display for RelativeTime
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		let now = SystemTime::now();
		if now < self.0 {
			return write!(f, "{}", Duration(self.0.duration_since(now).unwrap()));
		}

		return write!(f, "{}", Duration(now.duration_since(self.0).unwrap()));
	}
}

impl Display for Duration
{
	#[allow(unused_parens)]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		let mut secs = self.0.as_secs();
		let mut parts = vec![];

		if secs >= (24 * 60 * 60) {
			let days = secs / (24 * 60 * 60);
			secs %= (24 * 60 * 60);

			parts.push(format!("{days}d"));
		}

		if secs >= 60 * 60 {
			let hours = secs / (60 * 60);
			secs %= (60 * 60);

			if hours > 0 {
				parts.push(format!("{hours}h"));
			}
		}

		if secs >= 60 {
			let mins = secs / 60;
			secs %= 60;

			if mins > 0 {
				parts.push(format!("{mins}m"));
			}
		}

		if secs > 0 || parts.is_empty() {
			parts.push(format!("{secs}s"));
		}

		return write!(f, "{}", parts.join(" "));
	}
}

static SUPPORTED_KEYS: &[&str] = &[
	"years", "months", "weeks", "days", "hours", "hrs", "minutes", "mins", "seconds", "secs", "millis", "ms",
	"hhmmss", // special shorthand
];

static SHORTHAND_FMT_HHMMSS: LazyLock<Vec<TemplatePart>> =
	LazyLock::new(|| DurationFormatter::new("{hrs:02}:{mins:%02}:{secs:%02}").unwrap().0);

#[derive(Debug, PartialEq, Eq)]
enum Key
{
	Years,
	Months,
	Weeks,
	Days,
	Hours,
	Minutes,
	Seconds,
	Millis,
}

impl DurationFormatter
{
	/// Creates a new [`DurationFormatter`] with the given format string.
	///
	/// The format string is (in general) an "extended-format-specifier" (see [`crate::fmt`]) but in addition:
	/// - supported placeholder/argument names (the part before `:`) are fixed and limited
	/// - `%` is a supported flag that modulos the value to its natural range
	/// - `s` is a supported flag that simply outputs a literal `s` after the suffix (if any) when the value
	///   is not 1
	/// - `?` omits printing the value entirely when it is 0
	/// - the last part of the specifier can be `@...`, where `...` is any sequence of characters other than
	///   the closing `}`. This is the suffix that will be printed after the value itself.
	///
	/// The 'usual' specifiers (fill, width, precision) are supported and will be applied to the value.
	///
	/// The supported placeholder names are:
	/// - `years`            => years
	/// - `months`           => months
	/// - `weeks`            => weeks
	/// - `days`             => days
	/// - `hours`, `hrs`     => hours
	/// - `minutes`, `mins`  => minutes
	/// - `seconds`, `secs`  => seconds
	/// - `millis`, `ms`     => milliseconds
	/// - `hhmmss`           => 17:03:11 -- hours, minutes, seconds. zero-padded to two digits each.
	///
	/// # Errors
	/// Returns an error if the format string was invalid.
	#[allow(clippy::too_many_lines)]
	pub fn new<S: AsRef<str>>(template: S) -> Result<DurationFormatter, TemplateError>
	{
		let parts = fmt::parse_template(
			template,
			ParseOptions {
				relative_width: false,
				defer: false,
				style: true,
				extra_args: true,
				flag_handler: Some(|c| c == '%' || c == 's' || c == '?'),
			},
		)?;

		let it = parts.iter().find(|p| {
			if let TemplatePart::Placeholder { key, .. } = p {
				!SUPPORTED_KEYS.contains(&key.as_str())
			} else {
				false
			}
		});

		if let Some(it) = it
			&& let TemplatePart::Placeholder { key, .. } = it
		{
			return Err(TemplateError {
				char_index: 0,
				message: format!("unsupported placeholder key '{key}'"),
			});
		}

		return Ok(DurationFormatter(parts));
	}

	/// A helper function that formats parts.
	#[allow(clippy::too_many_lines)]
	fn format_parts_into<Writer: Write>(
		parts: &[TemplatePart],
		duration: StdDuration,
		outer_fmt: &mut Writer,
	) -> std::fmt::Result
	{
		for part in parts {
			use TemplatePart::*;

			if let Literal(lit) = part {
				outer_fmt.write_str(lit)?;
				continue;
			}

			let Placeholder {
				part_idx: _,
				key,
				alt,
				zero,
				sign,
				fill,
				align,
				width,
				precision,
				deferred: _,
				extra_args,
				extra_flags,
				ansi_style,
			} = part
			else {
				unreachable!()
			};

			let mut options = FormattingOptions::new();
			options
				.align(*align)
				.alternate(*alt)
				.sign_aware_zero_pad(*zero)
				.width(width.map(|x| x.resolve(0, 0)))
				.precision(precision.map(|x| x.resolve(0, 0)))
				.sign(match sign {
					None => None,
					Some('+') => Some(std::fmt::Sign::Plus),
					Some('-') => Some(std::fmt::Sign::Minus),
					Some(_) => unreachable!(),
				});

			if let Some(fill) = fill {
				options.fill(*fill);
			}

			let mut output = options.create_formatter(outer_fmt);
			let value_is_one = if key == "hhmmss" {
				let mut s = String::new();
				Self::format_parts_into(&SHORTHAND_FMT_HHMMSS, duration, &mut s)?;
				if let Some(style) = ansi_style {
					style.apply_to(s).fmt(&mut output)?;
				} else {
					s.fmt(&mut output)?;
				}

				false
			} else {
				let key = match key.as_str() {
					"years" => Key::Years,
					"months" => Key::Months,
					"weeks" => Key::Weeks,
					"days" => Key::Days,
					"hours" | "hrs" => Key::Hours,
					"minutes" | "mins" => Key::Minutes,
					"seconds" | "secs" => Key::Seconds,
					"millis" | "ms" => Key::Millis,
					_ => unreachable!(),
				};

				let secs = duration.as_secs_f64();
				let value = match key {
					Key::Years => secs / 86400.0 / 365.0,
					Key::Months => secs / 86400.0 / 30.0,
					Key::Weeks => secs / 86400.0 / 7.0,
					Key::Days => secs / 86400.0,
					Key::Hours => secs / 3600.0,
					Key::Minutes => secs / 60.0,
					Key::Seconds => secs,
					Key::Millis => secs * 1000.0,
				};

				#[allow(clippy::cast_sign_loss)]
				#[allow(clippy::cast_precision_loss)]
				#[allow(clippy::cast_possible_truncation)]
				let value = value.floor() as u64;

				// modulo it to its natural range if requested
				let value = if extra_flags.contains(&'%') {
					match key {
						Key::Years => value,
						Key::Months => value % 12,
						Key::Weeks => value % 52,
						Key::Days => value % 30,
						Key::Hours => value % 24,
						Key::Minutes | Key::Seconds => value % 60,
						Key::Millis => value % 1000,
					}
				} else {
					value
				};

				// if we asked to omit and the value is 0, then omit.
				if extra_flags.contains(&'?') && value == 0 {
					continue;
				}

				// format the actual value now
				if let Some(style) = ansi_style {
					style.apply_to(value).fmt(&mut output)?;
				} else {
					value.fmt(&mut output)?;
				}

				value == 1
			};

			if let Some(suffix) = extra_args {
				outer_fmt.write_str(suffix)?;
			}

			// add the 's' after the suffix, if any.
			if extra_flags.contains(&'s') && !value_is_one {
				outer_fmt.write_str("s")?;
			}
		}

		return Ok(());
	}


	/// Prints the duration using this configured [`DurationFormatter`] into the given formatter.
	///
	/// # Errors
	/// Returns an error if formatting failed.
	#[allow(clippy::too_many_lines)]
	pub fn format_into(&self, duration: StdDuration, outer_fmt: &mut Formatter) -> std::fmt::Result
	{
		return Self::format_parts_into(&self.0, duration, outer_fmt);
	}
}
