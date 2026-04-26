// template.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Alignment;

use crate::fmt;
use crate::fmt::ParseOptions;
use crate::fmt::TemplateError;
use crate::fmt::WidthPrecisionSpec;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
pub enum PlaceholderKey
{
	Message,
	Prefix,
	Percent,
	Position,
	Total,
	Padding,
	Rate,
	ElapsedTime,
	RemainingTime,
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
			"len" | "length" | "total" => Self::Total,
			"pad" | "padding" => Self::Padding,
			"rate" => Self::Rate,
			"elapsed" | "elapsed_time" => Self::ElapsedTime,
			"remaining" | "remaining_time" => Self::RemainingTime,
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

pub(crate) fn parse_template<S: AsRef<str>>(template: S) -> Result<Vec<TemplatePart>, TemplateError>
{
	let parts = fmt::parse_template(
		template,
		ParseOptions {
			relative_width: true,
			defer: true,
			extra_args: true,
			flag_handler: Some(|_: char| false),
		},
	)?;

	return Ok(parts
		.into_iter()
		.map(|p| match p {
			fmt::TemplatePart::Literal(l) => TemplatePart::Literal(l),
			fmt::TemplatePart::Placeholder {
				part_idx,
				key,
				alt,
				zero,
				sign,
				fill,
				align,
				width,
				precision,
				deferred,
				extra_args,
				extra_flags: _,
			} => TemplatePart::Placeholder {
				part_idx,
				key: PlaceholderKey::from_string(key),
				alt,
				zero,
				sign,
				fill,
				align,
				width,
				precision,
				deferred,
				extra_args,
			},
		})
		.collect::<Vec<_>>());
}
