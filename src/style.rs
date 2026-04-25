// style.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::any::Any;
use std::fmt::FormattingOptions;
use std::fmt::Write;
use std::sync::Arc;

use console::measure_text_width;
use itertools::Itertools;

use crate::ProgressBarAttribs;
use crate::template::TemplateError;
use crate::template::TemplatePart;
use crate::template::parse_template;

#[derive(Clone)]
pub struct Style
{
	pub(crate) template: Vec<TemplatePart>,
	pub(crate) formatter: Option<Arc<dyn PlaceholderFormat>>,

	pub(crate) spinner_chars: Vec<String>,
	pub(crate) spinner_char_width: usize,

	pub(crate) progress_bar_chars: Vec<String>,

	pub(crate) bouncer_chars: String,
	pub(crate) bouncer_char_width: usize,
}

pub trait PlaceholderFormat: Send + Sync
{
	/// Formats a named placeholder key with the options given, writing the result to the output `Write`.
	///
	/// # Errors
	/// Returns an error if an error occurred while formatting or writing to the output.
	fn format(
		&self,
		attribs: &ProgressBarAttribs,
		key: &str,
		options: FormattingOptions,
		term_width: usize,
		out: &mut dyn Write,
	) -> Result<(), std::fmt::Error>;

	fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl Style
{
	/// Creates a new progress bar style from the given template string.
	///
	/// # Errors
	/// Returns an error if the template string could not be parsed.
	pub fn from_template<S: AsRef<str>>(format: S) -> Result<Self, TemplateError>
	{
		let parts = parse_template(format)?;

		let (spinner_chars, spinner_char_width) = default_spinner_chars();
		let (bouncer_chars, bouncer_char_width) = default_bouncer_chars();

		return Ok(Self {
			template: parts,
			formatter: None,
			spinner_chars,
			spinner_char_width,

			progress_bar_chars: default_progress_bar_chars(),

			bouncer_chars,
			bouncer_char_width,
		});
	}

	/// Creates a dummy style with no elements.
	pub fn dummy() -> Self
	{
		let (spinner_chars, spinner_char_width) = default_spinner_chars();
		let (bouncer_chars, bouncer_char_width) = default_bouncer_chars();

		return Self {
			template: vec![],
			formatter: None,
			spinner_chars,
			spinner_char_width,

			progress_bar_chars: default_progress_bar_chars(),

			bouncer_chars,
			bouncer_char_width,
		};
	}

	/// Returns the custom formatter, if any.
	pub fn formatter(&self) -> Option<Arc<dyn PlaceholderFormat>>
	{
		return self.formatter.clone();
	}


	/// Sets the template string for the style.
	///
	/// # Panics
	/// Panics if the template could not be parsed. If you need to handle this case (why?)
	/// prefer [`Self::from_template`] instead.
	pub fn set_template<S: AsRef<str>>(&mut self, template: S)
	{
		self.template = parse_template(template).expect("could not parse template");
	}

	/// Sets the spinner sequence for the style.
	///
	/// Each element in the array must have the same display width (not necessarily
	/// number of characters).
	///
	/// # Panics
	/// Panics if any element in the array has a width different from the rest, or
	/// the width is 0, or if the array is empty.
	pub fn set_spinner_chars<S: AsRef<str>>(&mut self, chars: &[S])
	{
		assert!(!chars.is_empty(), "Spinner chars array cannot be empty");

		self.spinner_chars = chars.iter().map(|x| x.as_ref().into()).collect();
		self.spinner_char_width = self
			.spinner_chars
			.iter()
			.map(|c| measure_text_width(c))
			.all_equal_value()
			.expect("spinner chars have differing widths");
	}

	/// Sets the progress bar characters for the style. The first string
	/// is used for filled characters; the last for empty characters.
	///
	/// Strings between the first and last are used as in-between characters, when the last
	/// bit of the progress bar is not exactly filled but not exactly empty.
	///
	/// Each element in the array must have a display width of 1 (not necessarily
	/// number of characters).
	///
	/// # Panics
	/// Panics if any element in the array has a non-1 width, or if the array has fewer
	/// than 2 elements.
	pub fn set_progress_bar_chars<S: AsRef<str>>(&mut self, chars: &[S])
	{
		assert!(
			chars.len() >= 2,
			"Progress bar chars array must have at least 2 strings"
		);

		self.progress_bar_chars = chars.iter().map(|x| x.as_ref().into()).collect();
		let w = self
			.progress_bar_chars
			.iter()
			.map(|c| measure_text_width(c))
			.all_equal_value()
			.expect("progress bar chars have differing widths");

		assert!(w == 1, "progress bar chars have non-1 width: {w}");
	}

	/// Sets the bouncer string, which is used for unbounded progress bars.x
	///
	/// If a bouncing bar is not desired, pass the empty string, which will simply leave
	/// unbounded bars blank.
	pub fn set_bouncer<S: AsRef<str>>(&mut self, bouncer: S)
	{
		self.bouncer_chars = bouncer.as_ref().to_string();
		self.bouncer_char_width = measure_text_width(&self.bouncer_chars);
	}

	/// Sets the custom formatter for this style, which will be used for formatting
	/// any user-defined placeholder keys in the template.
	pub fn set_formatter(&mut self, formatter: Box<dyn PlaceholderFormat>)
	{
		self.formatter = Some(formatter.into());
	}

	/// Returns the default style for a progress bar.
	#[must_use]
	#[expect(clippy::missing_panics_doc, reason = "infallible")]
	pub fn default_bar() -> Self
	{
		return Self::from_template("{msg} [{bar:40!}] {pos}/{len}").unwrap();
	}

	/// Returns the default style for a spinner.
	#[must_use]
	#[expect(clippy::missing_panics_doc, reason = "infallible")]
	pub fn default_spinner() -> Self
	{
		return Self::from_template(" {spinner}  {msg}").unwrap();
	}
}

fn default_spinner_chars() -> (Vec<String>, usize)
{
	// ["⡿", "⣟", "⣯", "⣷", "⣾", "⣽", "⣻", "⢿"]
	// ["⢻⣿", "⣹⣿", "⣼⣿", "⣶⣿", "⣷⣾", "⣿⣶", "⣿⣧", "⣿⣏", "⣿⡟", "⣿⠿", "⡿⢿", "⠿⣿"]
	// ["⢿⣿", "⣻⣿", "⣽⣿", "⣾⣿", "⣷⣿", "⣿⣾", "⣿⣷", "⣿⣯", "⣿⣟", "⣿⡿", "⣿⢿", "⡿⣿"]

	return (
		["⢿⣿", "⣻⣿", "⣽⣿", "⣾⣿", "⣷⣿", "⣿⣾", "⣿⣷", "⣿⣯", "⣿⣟", "⣿⡿", "⣿⢿", "⡿⣿"]
			.into_iter()
			.map(|x| x.into())
			.collect(),
		measure_text_width("⣿⣿"),
	);
}

fn default_bouncer_chars() -> (String, usize)
{
	let bouncer = "━━━━━";
	return (bouncer.to_string(), measure_text_width(bouncer));
}

fn default_progress_bar_chars() -> Vec<String>
{
	return ["━", "╸", " "].into_iter().map(|x| x.into()).collect();
}
