// template.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::error::Error;
use std::fmt::Alignment;
use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum WidthPrecisionSpec
{
	Absolute(u16),
	PercentAbsolute(u16),
	PercentRelative(u16),
}

impl WidthPrecisionSpec
{
	#[allow(clippy::cast_possible_truncation)]
	pub fn resolve(self, total_width: usize, avail_width: usize) -> u16
	{
		return match self {
			Self::Absolute(x) => x,
			Self::PercentAbsolute(x) => ((total_width * (x as usize)) / 100) as u16,
			Self::PercentRelative(x) => ((avail_width * (x as usize)) / 100) as u16,
		};
	}

	#[allow(clippy::cast_possible_truncation)]
	fn append_digit(&mut self, digit: char)
	{
		match self {
			Self::Absolute(x) | Self::PercentAbsolute(x) | Self::PercentRelative(x) => {
				*x = (10 * (*x)) + (digit.to_digit(10).unwrap() as u16);
			}
		}
	}

	#[allow(clippy::cast_possible_truncation)]
	fn abs_from_digit(digit: char) -> Self
	{
		return Self::Absolute(digit.to_digit(10).unwrap() as u16);
	}
}

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
pub enum PlaceholderKey
{
	Message,
	Prefix,
	Percent,
	Position,
	PositionBinaryBytes,
	PositionDecimalBytes,
	Total,
	TotalBinaryBytes,
	TotalDecimalBytes,
	Padding,
	Rate,
	RateBinaryBytes,
	RateDecimalBytes,
	Bar,
	Spinner,
	Custom(String),
}

impl PlaceholderKey
{
	pub(crate) fn from_string(key: String) -> Self
	{
		return match key.as_str() {
			"msg" | "message" | "description" => Self::Message,
			"prefix" => Self::Prefix,
			"percent" => Self::Percent,
			"pos" | "position" => Self::Position,
			"pos_bytes" | "pos_binary_bytes" => Self::PositionBinaryBytes,
			"pos_decimal_bytes" => Self::PositionDecimalBytes,
			"len" | "length" | "total" => Self::Total,
			"len_bytes" | "len_binary_bytes" | "total_bytes" | "total_binary_bytes" => Self::TotalBinaryBytes,
			"len_decimal_bytes" | "total_decimal_bytes" => Self::TotalDecimalBytes,
			"pad" | "padding" => Self::Padding,
			"rate" => Self::Rate,
			"rate_bytes" | "rate_binary_bytes" => Self::RateBinaryBytes,
			"rate_decimal_bytes" => Self::RateDecimalBytes,
			"bar" | "progress_bar" => Self::Bar,
			"spinner" => Self::Spinner,

			_ => Self::Custom(key),
		};
	}
}


#[derive(Debug, Clone)]
pub(crate) enum TemplatePart
{
	Literal(String),
	Placeholder
	{
		part_idx: usize,
		key: PlaceholderKey,
		alt: bool,
		zero: bool,
		sign: Option<char>,
		fill: Option<char>,
		align: Option<Alignment>,
		width: Option<WidthPrecisionSpec>,
		precision: Option<WidthPrecisionSpec>,
		deferred: u16,
		extra_args: Option<String>,
	},
}


// shut the fuck up
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_template<S: AsRef<str>>(template: S) -> Result<Vec<TemplatePart>, TemplateError>
{
	#[derive(Debug, PartialEq, Eq)]
	enum ParserState
	{
		Literal,
		PlaceholderName,
		OneOpenBrace,
		OneCloseBrace,

		PlaceholderParams,
		PlaceholderWidth,
		PlaceholderPrecision,
		PlaceholderFillAlign,
		PlaceholderExtraArgs,
	}

	use std::mem::take;

	use ParserState::*;
	use WidthPrecisionSpec as WPS;

	let mut state = Literal;
	let mut parts = vec![];
	let mut buffer = String::new();

	let mut alt = false;
	let mut zero = false;
	let mut fill = None;
	let mut sign = None;
	let mut width = None;
	let mut deferred = 0;
	let mut precision = None;
	let mut alignment = None;
	let mut extra_args = String::new();

	let chars: Vec<_> = template.as_ref().chars().collect();

	for (char_index, input) in chars.iter().enumerate() {
		let next = if char_index + 1 < chars.len() {
			Some(chars[char_index + 1])
		} else {
			None
		};

		if state == PlaceholderFillAlign && !matches!((input, next), (_, Some('<' | '^' | '>'))) {
			state = PlaceholderParams;
		}

		let (next_state, maybe_ch) = match (state, input, next) {
			(Literal, '{', _) => (OneOpenBrace, None),
			(Literal, '}', _) => (OneCloseBrace, None),

			(Literal, ch, _) => (Literal, Some(*ch)),

			(OneOpenBrace, '{', _) => (Literal, Some('{')),
			(OneOpenBrace, ':', _) => {
				return Err(TemplateError {
					char_index,
					message: "invalid empty placeholder name".to_string(),
				});
			}
			(OneOpenBrace, '}', _) => {
				return Err(TemplateError { char_index, message: "invalid empty placeholder".to_string() });
			}

			// buffer the 'ch' as the start of the key, and start parsing the placeholder name.
			(OneOpenBrace, ch, _) => {
				// flush the current buffer
				parts.push(TemplatePart::Literal(take(&mut buffer)));

				(PlaceholderName, Some(*ch))
			}

			// if we are parsing the placeholder name or its params (incl precision) and we see the '}',
			// we are finished.
			(
				PlaceholderName | PlaceholderParams | PlaceholderWidth | PlaceholderPrecision | PlaceholderExtraArgs,
				'}',
				_,
			) => {
				let name = take(&mut buffer);
				let align = alignment;

				parts.push(TemplatePart::Placeholder {
					part_idx: parts.len(),
					key: PlaceholderKey::from_string(name),
					alt,
					zero,
					sign,
					fill,
					align,
					deferred,
					width,
					precision,
					extra_args: if extra_args.is_empty() { None } else { Some(extra_args) },
				});

				// reset all of them
				alt = false;
				zero = false;
				fill = None;
				sign = None;
				width = None;
				deferred = 0;
				precision = None;
				alignment = None;
				extra_args = String::new();

				// go back to parsing literal strings
				(Literal, None)
			}

			(PlaceholderName, ':', _) => (PlaceholderFillAlign, None),
			(PlaceholderName, ch, _) => (PlaceholderName, Some(*ch)),

			(PlaceholderFillAlign, ch, Some('<' | '^' | '>')) => {
				fill = Some(*ch);
				(PlaceholderParams, None)
			}

			(PlaceholderParams, '<', _) => {
				alignment = Some(Alignment::Left);
				(PlaceholderParams, None)
			}

			(PlaceholderParams, '^', _) => {
				alignment = Some(Alignment::Center);
				(PlaceholderParams, None)
			}

			(PlaceholderParams, '>', _) => {
				alignment = Some(Alignment::Right);
				(PlaceholderParams, None)
			}

			(s @ (PlaceholderParams | PlaceholderWidth | PlaceholderPrecision), '!', _) => {
				deferred += 1;
				(s, None)
			}

			(PlaceholderParams, '+', _) => {
				sign = Some('+');
				(PlaceholderParams, None)
			}

			(PlaceholderParams, '-', _) => {
				sign = Some('-');
				(PlaceholderParams, None)
			}

			(PlaceholderParams, '#', _) => {
				alt = true;
				(PlaceholderParams, None)
			}

			(PlaceholderParams, '0', _) => {
				zero = true;
				(PlaceholderParams, None)
			}

			(PlaceholderWidth, '%', _) if width.is_some() => {
				width = match width {
					Some(WPS::Absolute(x)) => Some(WPS::PercentAbsolute(x)),
					Some(WPS::PercentAbsolute(x)) => Some(WPS::PercentRelative(x)),
					Some(WPS::PercentRelative(_)) => {
						return Err(TemplateError {
							char_index,
							message: "at most two '%'s are allowed".to_string(),
						});
					}
					None => unreachable!(),
				};

				(PlaceholderWidth, None)
			}

			(PlaceholderPrecision, '%', _) if width.is_some() => {
				precision = match precision {
					Some(WPS::Absolute(x)) => Some(WPS::PercentAbsolute(x)),
					Some(WPS::PercentAbsolute(x)) => Some(WPS::PercentRelative(x)),
					Some(WPS::PercentRelative(_)) => {
						return Err(TemplateError {
							char_index,
							message: "at most two '%'s are allowed".to_string(),
						});
					}
					None => unreachable!(),
				};

				(PlaceholderPrecision, None)
			}

			(PlaceholderParams | PlaceholderWidth, '.', _) => (PlaceholderPrecision, None),

			#[allow(clippy::cast_possible_truncation)]
			(PlaceholderParams | PlaceholderWidth, ch @ '1'..='9', _) | (PlaceholderWidth, ch @ '0'..='9', _) => {
				match width.as_mut() {
					Some(width) => width.append_digit(*ch),
					None => width = Some(WPS::abs_from_digit(*ch)),
				}
				(PlaceholderWidth, None)
			}

			#[allow(clippy::cast_possible_truncation)]
			(PlaceholderPrecision, ch @ '0'..='9', _) => {
				match precision.as_mut() {
					Some(prec) => prec.append_digit(*ch),
					None => precision = Some(WPS::abs_from_digit(*ch)),
				}
				(PlaceholderPrecision, None)
			}

			(PlaceholderParams | PlaceholderPrecision | PlaceholderWidth, '@', _) => (PlaceholderExtraArgs, None),

			(PlaceholderExtraArgs, ch, _) => {
				extra_args.push(*ch);
				(PlaceholderExtraArgs, None)
			}

			(PlaceholderPrecision | PlaceholderParams, ch, _) => {
				return Err(TemplateError { char_index, message: format!("invalid char '{ch}'") });
			}

			(OneCloseBrace, '}', _) => (Literal, Some('}')),
			(OneCloseBrace, ch, _) => {
				return Err(TemplateError {
					char_index,
					message: format!("unexpected character '{ch}' after '}}'"),
				});
			}

			(state, ch, _) => {
				return Err(TemplateError {
					char_index,
					message: format!("unexpected character '{ch}' in state {state:?}"),
				});
			}
		};

		state = next_state;
		if let Some(ch) = maybe_ch {
			buffer.push(ch);
		}
	}

	if !buffer.is_empty() {
		parts.push(TemplatePart::Literal(buffer));
	}

	return Ok(parts);
}

/// An error that occurred while parsing the template string for a progres bar style.
#[derive(Debug)]
pub struct TemplateError
{
	/// The character index in the template string where the error occurred.
	pub char_index: usize,

	/// An error message describing the error.
	pub message: String,
}

impl Display for TemplateError
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		return writeln!(f, "template error at position {}: {}", self.char_index, self.message);
	}
}

impl Error for TemplateError {}
