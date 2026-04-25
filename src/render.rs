// render.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::FormattingOptions;
use std::fmt::Write;
use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::ProgressBarAttribs;
use crate::State;
use crate::Style;
use crate::fmt::ByteSize;
use crate::fmt::Scale;
use crate::progress_bar::GLOBAL_PAUSE;
use crate::progress_bar::ProgressBar;
use crate::progress_bar::ProgressBarCore;
use crate::target::RenderTarget;
use crate::template::PlaceholderKey;
use crate::template::TemplatePart;

type FmtResult = std::fmt::Result;

impl ProgressBarCore
{
	pub fn render(&self, out: &RenderTarget)
	{
		if GLOBAL_PAUSE.load(Ordering::Acquire) > 0 {
			return;
		}

		self.target.reset(/* clear: */ false, /* flush: */ false);

		if !self.attribs.hidden {
			let _ = self.render_self(out);
		}

		for (indent, child) in &self.children {
			let _ = indent;

			child.read().render(out);
		}

		self.target.flush();
	}

	fn render_self(&self, out: &RenderTarget) -> FmtResult
	{
		let now = Instant::now();

		let style = &self.attribs.style;
		let state = &self.attribs.state;

		let mut current_pass = 0;

		let total_width = out.width();
		let mut avail_width = total_width;

		let mut deferred_parts = vec![];
		let mut parts = style.template.iter().collect::<Vec<&_>>();

		let mut out_buffer = String::new();

		while !parts.is_empty() {
			for part in &parts {
				let mut buffer = String::new();

				match part {
					TemplatePart::Literal(s) => {
						let _ = buffer.write_str(s);
					}

					// if it is not time to render it yet, add it to the deferred list.
					TemplatePart::Placeholder { part_idx, key, deferred, .. } if *deferred > current_pass => {
						deferred_parts.push(*part);

						// on the first defer, put the placeholder marker in the string
						// so we know where to write to later.
						if current_pass == 0 {
							// write directly to the out buffer and bypass the measurement
							let _ = out_buffer.write_fmt(format_args!("\0{key}.{part_idx}\0"));
							buffer.clear();
						}
					}

					// if the time has passed, skip it entirely.
					TemplatePart::Placeholder { deferred, .. } if *deferred > current_pass => {}

					placeholder @ TemplatePart::Placeholder { part_idx, key, deferred, .. }
						if *deferred == current_pass =>
					{
						render_placeholder(
							placeholder,
							&mut buffer,
							now,
							style,
							state,
							&self.attribs,
							avail_width,
							total_width,
						)?;

						if *deferred > 0 && *deferred == current_pass {
							out_buffer = out_buffer.replace(&format!("\0{key}.{part_idx}\0"), &buffer);

							let used_width = console::measure_text_width(&buffer);
							avail_width = avail_width.saturating_sub(used_width);

							buffer.clear();
						}
					}

					TemplatePart::Placeholder { .. } => unreachable!(),
				}

				let used_width = console::measure_text_width(&buffer);
				avail_width = avail_width.saturating_sub(used_width);
				out_buffer += &buffer;
			}

			parts = std::mem::take(&mut deferred_parts);
			current_pass += 1;
		}

		out.write_line(&out_buffer);
		return Ok(());
	}
}

impl ProgressBar
{
	pub fn render(&self, target: &RenderTarget)
	{
		self.core.read().render(target);
	}
}

#[allow(clippy::too_many_arguments)]
fn render_placeholder(
	placeholder: &TemplatePart,
	buffer: &mut String,
	now: Instant,
	style: &Style,
	state: &State,
	attribs: &ProgressBarAttribs,
	avail_width: usize,
	total_width: usize,
) -> FmtResult
{
	use PlaceholderKey as K;

	let TemplatePart::Placeholder { key, alt, zero, sign, fill, align, width, precision, .. } = placeholder else {
		unreachable!();
	};

	let mut options = FormattingOptions::new();

	options
		.align(*align)
		.alternate(*alt)
		.sign_aware_zero_pad(*zero)
		.width(width.map(|x| x.resolve(total_width, avail_width)))
		.precision(precision.map(|x| x.resolve(total_width, avail_width)))
		.sign(match sign {
			None => None,
			Some('+') => Some(std::fmt::Sign::Plus),
			Some('-') => Some(std::fmt::Sign::Minus),
			Some(_) => unreachable!(),
		});

	if let Some(fill) = *fill {
		options.fill(fill);
	}

	let mut fmt = options.create_formatter(buffer);

	match key {
		K::Message => attribs.message.fmt(&mut fmt)?,

		K::Padding => "".fmt(&mut fmt)?,

		K::Prefix => attribs.prefix.fmt(&mut fmt)?,

		K::Percent => (100.0 * state.fraction()).fmt(&mut fmt)?,

		K::Position => state.position.load(Ordering::Relaxed).fmt(&mut fmt)?,

		K::Rate => attribs.estimator.estimate(now).fmt(&mut fmt)?,

		K::Total => {
			if let Some(length) = state.total {
				length.fmt(&mut fmt)?;
			}
		}

		K::PositionBinaryBytes | K::PositionDecimalBytes => {
			let bin = *key == K::PositionBinaryBytes;

			let (ib, spc) = if *alt { (false, false) } else { (bin, true) };
			ByteSize(state.position.load(Ordering::Relaxed))
				.custom()
				.with_scale(if bin { Scale::Binary } else { Scale::Decimal })
				.with_ibibytes(ib)
				.with_space(spc)
				.fmt(&mut fmt)?;
		}

		K::TotalBinaryBytes | K::TotalDecimalBytes => {
			if let Some(total) = state.total {
				let bin = *key == K::TotalBinaryBytes;

				let (ib, spc) = if *alt { (false, false) } else { (bin, true) };
				ByteSize(total)
					.custom()
					.with_scale(if bin { Scale::Binary } else { Scale::Decimal })
					.with_ibibytes(ib)
					.with_space(spc)
					.fmt(&mut fmt)?;
			}
		}

		K::RateBinaryBytes | K::RateDecimalBytes => {
			let bin = *key == K::RateBinaryBytes;

			let (ib, spc) = if *alt { (false, false) } else { (bin, true) };

			#[allow(clippy::cast_sign_loss)]
			#[allow(clippy::cast_possible_truncation)]
			ByteSize(attribs.estimator.estimate(now) as u64)
				.custom()
				.with_scale(if bin { Scale::Binary } else { Scale::Decimal })
				.with_ibibytes(ib)
				.with_space(spc)
				.fmt(&mut fmt)?;
		}

		K::Bar => render_bar(state, style, avail_width, &mut fmt)?,
		K::Spinner => render_spinner(state, style, avail_width, &mut fmt)?,

		K::Custom(key) => {
			let Some(custom) = &style.formatter else {
				panic!("no custom formatter for key '{key}'");
			};

			custom.format(attribs, key, options, avail_width, &mut fmt)?;
		}
	}

	return Ok(());
}




const EPSILON: f64 = 1e-7;
const EMPTY: &str = "";

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
fn render_bar(state: &State, style: &Style, avail_width: usize, fmt: &mut Formatter<'_>) -> FmtResult
{
	let bar_width = fmt.width().unwrap_or(avail_width).min(avail_width);
	if bar_width == 0 {
		return Ok(());
	}

	if state.total.is_some() {
		let frac = state.fraction().clamp(0.0, 1.0);
		let num_chars = frac * (bar_width as f64);

		let whole_chars = (num_chars + EPSILON).floor() as usize;
		let partial_frac = (num_chars - whole_chars as f64).clamp(0.0, 1.0);

		// write the whole portion
		for _ in 0..whole_chars {
			fmt.write_str(&style.progress_bar_chars[0])?;
		}

		if whole_chars < bar_width {
			let partial_chars = &style.progress_bar_chars[1..];
			if partial_chars.len() == 1 {
				fmt.write_str(&partial_chars[0])?;
			} else {
				let n = ((1.0 - partial_frac) * partial_chars.len() as f64 - 1.0).round();

				let selected = &partial_chars[(n as usize).min(partial_chars.len() - 1)];
				fmt.write_str(selected)?;
			}

			// write the empty bits.
			let empty_chars = bar_width - whole_chars - 1;
			for _ in 0..empty_chars {
				fmt.write_str(style.progress_bar_chars.last().unwrap())?;
			}
		}
	} else {
		// if the bar width cannot fit our bouncer, just fill it with spaces and call it a day.
		if bar_width < style.bouncer_char_width {
			fmt.write_fmt(format_args!("{EMPTY:bar_width$}"))?;
			return Ok(());
		}

		// the number of positions we have to render the bar, taking into account its own width, and in the reverse
		// direction as well.
		let bar_space = bar_width - style.bouncer_char_width;
		let positions = 2 * bar_space;
		let bouncer = &style.bouncer_chars;

		let pos = (state.ticks.load(Ordering::Relaxed) as usize) % positions;
		let pos = if pos > bar_space {
			bar_space - (pos - bar_space)
		} else {
			pos
		};

		let post_spaces = bar_width.saturating_sub(pos).saturating_sub(style.bouncer_char_width);

		// print spaces before the bar, then the bar, then spaces after.
		fmt.write_fmt(format_args!("{EMPTY:pos$}{bouncer}{EMPTY:post_spaces$}"))?;
	}

	return Ok(());
}

#[allow(clippy::cast_possible_truncation)]
fn render_spinner(state: &State, style: &Style, avail_width: usize, fmt: &mut Formatter<'_>) -> FmtResult
{
	let _ = avail_width;

	let num_chars = style.spinner_chars.len();
	let spinner = &style.spinner_chars[state.ticks.load(Ordering::Relaxed) as usize % num_chars];

	spinner.fmt(fmt)?;
	return Ok(());
}
