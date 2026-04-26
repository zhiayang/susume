// parser.rs
// Copyright (c) 2026, yuki
// SPDX-License-Identifier: MPL-2.0

use std::error::Error;
use std::fmt::Alignment;
use std::fmt::Display;

use console::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WidthPrecisionSpec
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

#[derive(Debug, Clone)]
pub(crate) enum TemplatePart
{
	Literal(String),
	Placeholder
	{
		part_idx: usize,
		key: String,
		alt: bool,
		zero: bool,
		sign: Option<char>,
		fill: Option<char>,
		align: Option<Alignment>,
		width: Option<WidthPrecisionSpec>,
		precision: Option<WidthPrecisionSpec>,
		deferred: u16,
		extra_args: Option<String>,
		extra_flags: Vec<char>,
		ansi_style: Option<Style>,
	},
}

pub(crate) struct ParseOptions<F: FnMut(char) -> bool>
{
	pub(crate) relative_width: bool,
	pub(crate) extra_args: bool,
	pub(crate) defer: bool,
	pub(crate) style: bool,

	pub(crate) flag_handler: Option<F>,
}

impl<F: FnMut(char) -> bool> Default for ParseOptions<F>
{
	fn default() -> Self
	{
		return ParseOptions {
			relative_width: false,
			extra_args: false,
			defer: false,
			style: false,
			flag_handler: None,
		};
	}
}

// shut the fuck up
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_template<S: AsRef<str>, FH: FnMut(char) -> bool>(
	template: S,
	mut options: ParseOptions<FH>,
) -> Result<Vec<TemplatePart>, TemplateError>
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
		PlaceholderAnsiStyle,
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
	let mut extra_flags = vec![];
	let mut ansi_style: Option<String> = None;

	let mut nested_brace_count = 0;

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
				if !buffer.is_empty() {
					parts.push(TemplatePart::Literal(take(&mut buffer)));
				}

				(PlaceholderName, Some(*ch))
			}

			// handle braces in the extra_args (@) part by tracking the count. only close if we matched.
			(PlaceholderExtraArgs, '{', _) => {
				extra_args.push('{');
				nested_brace_count += 1;
				(PlaceholderExtraArgs, None)
			}

			(PlaceholderExtraArgs, '}', _) if nested_brace_count > 0 => {
				extra_args.push('}');
				nested_brace_count -= 1;
				(PlaceholderExtraArgs, None)
			}

			// if we are parsing the placeholder name or its params (incl precision) and we see the '}',
			// we are finished.
			(
				PlaceholderName | PlaceholderParams | PlaceholderWidth | PlaceholderPrecision | PlaceholderAnsiStyle
				| PlaceholderExtraArgs,
				'}',
				_,
			) => {
				let name = take(&mut buffer);
				let align = alignment;

				parts.push(TemplatePart::Placeholder {
					part_idx: parts.len(),
					key: name,
					alt,
					zero,
					sign,
					fill,
					align,
					deferred,
					width,
					precision,
					extra_args: if extra_args.is_empty() { None } else { Some(extra_args) },
					extra_flags,
					ansi_style: ansi_style.map(|x| Style::from_dotted_str(x.as_ref()).for_stderr()),
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
				extra_flags = vec![];
				ansi_style = None;

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

			(s @ (PlaceholderParams | PlaceholderWidth | PlaceholderPrecision), '!', _) if options.defer => {
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

			(PlaceholderWidth, '%', _) if options.relative_width && width.is_some() => {
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

			(PlaceholderPrecision, '%', _) if options.relative_width && width.is_some() => {
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

			(PlaceholderParams | PlaceholderPrecision | PlaceholderWidth, '$', _) if options.style => {
				ansi_style = Some(String::new());
				(PlaceholderAnsiStyle, None)
			}

			// if we see a '@', we should go into the extra_args state. the dotted-string
			// format that `console::Style` accepts shouldn't contain a '@'
			(PlaceholderParams | PlaceholderPrecision | PlaceholderWidth | PlaceholderAnsiStyle, '@', _)
				if options.extra_args =>
			{
				(PlaceholderExtraArgs, None)
			}

			(PlaceholderExtraArgs, ch, _) => {
				extra_args.push(*ch);
				(PlaceholderExtraArgs, None)
			}

			(PlaceholderAnsiStyle, ch, _) => {
				ansi_style.as_mut().map(|x| x.push(*ch));
				(PlaceholderAnsiStyle, None)
			}

			(s @ (PlaceholderPrecision | PlaceholderParams | PlaceholderWidth), ch, _) => {
				if let Some(ref mut flag_handler) = options.flag_handler
					&& flag_handler(*ch)
				{
					extra_flags.push(*ch);
					(s, None)
				} else {
					return Err(TemplateError { char_index, message: format!("invalid char '{ch}'") });
				}
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

	// if we are out of chars and we are not in the literal state, it's an error.
	if state != Literal {
		return Err(TemplateError {
			char_index: chars.len(),
			message: format!("unterminated format specifier: left in state {state:?}"),
		});
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
