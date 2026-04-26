// render.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::FormattingOptions;
use std::fmt::Write;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use crate::ProgressBarAttribs;
use crate::State;
use crate::Style;
use crate::fmt::ByteSize;
use crate::fmt::DurationFormatter;
use crate::fmt::Scale;
use crate::progress_bar::GLOBAL_PAUSE;
use crate::progress_bar::ProgressBar;
use crate::progress_bar::ProgressBarCore;
use crate::target::RenderTarget;
use crate::template::FmtItem;
use crate::template::PlaceholderKey;

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
					FmtItem::Literal(s) => {
						let _ = buffer.write_str(s);
					}

					// if it is not time to render it yet, add it to the deferred list.
					FmtItem::Placeholder { part_idx, key, deferred, .. } if *deferred > current_pass => {
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
					FmtItem::Placeholder { deferred, .. } if *deferred < current_pass => {}

					placeholder @ FmtItem::Placeholder { part_idx, key, deferred, .. } if *deferred == current_pass => {
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

					FmtItem::Placeholder { .. } => unreachable!(),
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

#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
fn render_placeholder(
	placeholder: &FmtItem,
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

	let FmtItem::Placeholder {
		key,
		alt,
		zero,
		sign,
		fill,
		align,
		width,
		precision,
		extra_args,
		duration_formatter,
		..
	} = placeholder
	else {
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

	// see if any formatters can format this key.
	for formatter in style.formatters() {
		let maybe_result = formatter.format(
			attribs,
			key,
			extra_args.as_ref().map(|x| x.as_str()),
			options,
			avail_width,
			&mut fmt,
		);

		// if it says it can handle the formatter, we take the result.
		if let Some(result) = maybe_result {
			return result;
		}
	}

	// note: if we got here, it means we already looked through all the custom formatters
	// and none matched, so we will cry (loudly) about it.
	if let K::Custom(k) = key {
		panic!("no custom formatter for key '{k}'");
	}

	let numeric_fmt_helper = |fmt: &mut Formatter, key: &str, num: u64| -> FmtResult {
		return match extra_args.as_ref().map(|x| x.as_str()) {
			None => num.fmt(fmt),

			// we default 'byte' to binary units.
			Some("byte" | "bytes" | "bin" | "binary" | "bin_bytes" | "binary_bytes") => ByteSize(num)
				.custom()
				.with_scale(Scale::Binary)
				.with_ibibytes(!alt)
				.with_space(!alt)
				.fmt(fmt),

			Some("dec" | "decimal" | "dec_bytes" | "decimal_bytes") => ByteSize(num)
				.custom()
				.with_scale(Scale::Decimal)
				.with_ibibytes(!alt)
				.with_space(!alt)
				.fmt(fmt),

			Some(others) => {
				panic!("unsupported extra-args '{others}' for key '{key}'")
			}
		};
	};

	let duration_fmt_helper = |fmt: &mut Formatter, _key: &str, duration: Duration| -> FmtResult {
		return match extra_args.as_ref().map(|x| x.as_str()) {
			// default -- print as human-readable
			None | Some("") => crate::fmt::Duration(duration).fmt(fmt),

			// parse the time format here... using strftime (ish) format.
			Some(template) => {
				if let Some(df) = duration_formatter {
					df.format_into(duration, fmt)
				} else {
					let df = DurationFormatter::new(template);
					if let Ok(df) = df {
						df.format_into(duration, fmt)
					} else {
						Err(std::fmt::Error)
					}
				}
			}
		};
	};

	// it better be a builtin key now.
	match key {
		K::Message => attribs.message.fmt(&mut fmt)?,

		K::Padding => "".fmt(&mut fmt)?,

		K::Prefix => attribs.prefix.fmt(&mut fmt)?,

		K::Percent => (100.0 * state.fraction()).fmt(&mut fmt)?,

		K::Position => numeric_fmt_helper(&mut fmt, "position", state.position.load(Ordering::Relaxed))?,
		K::Total => {
			if let Some(total) = state.total {
				numeric_fmt_helper(&mut fmt, "total", total)?;
			}
		}

		K::Rate => {
			let rate = attribs.estimator.estimate(now);
			if extra_args.is_none() {
				rate.fmt(&mut fmt)?;
			} else {
				#[allow(clippy::cast_sign_loss)]
				#[allow(clippy::cast_possible_truncation)]
				numeric_fmt_helper(&mut fmt, "rate", attribs.estimator.estimate(now) as u64)?;
			}
		}

		K::ElapsedTime => duration_fmt_helper(&mut fmt, "elapsed_time", attribs.estimator.elapsed(now))?,

		K::RemainingTime => {
			// we need a total to estimate remaining time.
			let Some(total) = state.total else { return Ok(()) };

			// rate is in items / second
			let rate = attribs.estimator.estimate(now);
			let remaining = total.saturating_sub(state.position.load(Ordering::Relaxed));

			// protect against division by 0; if rate is 0, make it 1.
			let rate = if rate == 0.0 { 1.0 } else { rate };

			#[allow(clippy::cast_precision_loss)]
			duration_fmt_helper(
				&mut fmt,
				"remaining_time",
				Duration::from_secs_f64(remaining as f64 / rate),
			)?;
		}

		K::Bar => render_bar(state, style, avail_width, &mut fmt)?,
		K::Spinner => render_spinner(state, style, avail_width, &mut fmt)?,

		K::Custom(_) => unreachable!(),
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
