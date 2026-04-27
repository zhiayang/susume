// lib.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

#![feature(formatting_options)]
#![allow(clippy::needless_return)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]

mod estimator;
mod format;
mod progress_bar;
mod render;
mod state;
mod style;
mod target;
mod template;
mod ticker;

#[cfg(test)]
mod tests;

pub use estimator::Estimator;
pub use estimator::EstimatorImpl;
pub use progress_bar::ProgressBar;
pub use progress_bar::ProgressBarAttribs;
pub use state::State;
pub use style::Style;
pub use target::RenderTarget;

/// # Formatting Specifiers
/// Progress bars can have templates set on them (either with [`ProgressBar::with_template`], or
/// [`Style::set_template`]) that determine how each item in the progress bar is displayed on screen.
///
/// For example, this template:
/// `"{spinner}  [{message}]  ({bar})  ({position}/{total})"`
///
/// Will result in this bar:
/// ⢿⣿  [hello, world!]  (━━━━━━━━━━━━━━━━━━&emsp;&emsp;&emsp;&emsp;&emsp;)  (100/999)
///
/// The template string is similar to what you would use in [`format!()`], except the placeholder name
/// (the part within `{}`, before the colon) is referred to as the "key".
///
/// Several built-in keys are defined for convenience:
/// - `msg`, `message`, `description` => A user-defined message. Set with [`ProgressBar::set_message()`]
/// - `prefix`                        => A user-defined prefix. Not exposed except via [`ProgressBarAttribs`]
/// - `percent`                       => The completion percentage; calculated as `pos / total * 100`
/// - `pos`, `position`               => The current "position" of the bar
/// - `len`, `length`, `total`        => The total count of the bar, if one was set
/// - `pad`, `padding`                => Used for padding, like `"{pad:10}"`
/// - `rate`                          => The estimated rate of progress, in items-per-second
/// - `elapsed`, `elapsed_time`       => The elapsed time, as a duration
/// - `remaining`, `remaining_time`   => The estimated time remaining, calculated from the (estimated) rate
/// - `bar`, `progress_bar`           => The main progress bar
/// - `spinner`                       => A spinner that indicates activity but not progress
///
///
/// # Custom Keys and Formatters
/// You can provide one (or more) custom formatters implementing [`crate::style::PlaceholderFormatter`] to the
/// progress bar via [`ProgressBar::with_formatter()`]. They are always consulted before rendering any key
/// (even builtin ones), so you can use them to override how built-in keys are rendered.
///
/// More importantly, they also let you handle custom placeholder keys however you want.
///
///
/// # Literal Keys
/// One other extension is the ability to use a single-quoted string as a placeholder key; in this case, the
/// string is printed as-is. This is a convenience feature to let you use the styling/extended specifiers,
/// without having to wrap the entire template string in `format!()` that requires double-bracing everything.
///
/// For example: `"{'foo':$blue}"` will result in the string "foo" being displayed in blue.
///
///
/// # Extended Specifiers
/// Apart from the usual format specifiers that Rust usually supports (`{foo:<10}`, for example), more exotic
/// specifiers are also supported here.
///
/// * *`$`*. This allows you to style the value that is printed using the [console](https://docs.rs/console/latest/console)
/// crate. The string following `$` is passed to [`console::Style::from_dotted_str`], allowing you to do
/// things like `{elapsed:$red.on_blue}` to print the elapsed time as red text on a blue background.
///
/// * *`@`*. This allows you to pass context-specific extra arguments to the formatter for the value.
///   Currently, this is
/// only used for duration formatting -- ie. `"elapsed"` and `"remaining"`. Those values go through a special
/// [`fmt::DurationFormatter`] that itself uses the same extended-format-string syntax. This allows you to do
/// cool (arguably also cursed) things like: `{elapsed:@{hours:02}:{minutes:%02}:{seconds:%02}}` to print
/// elapsed time in `HH:MM:SS` format. Refer to the documentation for [`fmt::DurationFormatter`] to see what
/// specific extra flags it also supports (like `%` and `?`).
///
/// Note that if you use these extended specifiers, `$` must come after *ALL* the usual specifiers (width,
/// precision, alignment, etc.). The style string will be parsed as the `$` to the closing `}` of the
/// specifier. The same applies to `@` arguments. If both are used, `$` *must* come before `@`.
pub mod fmt
{
	pub use crate::format::*;
	pub use crate::style::PlaceholderFormatter;
	pub use crate::template::PlaceholderKey;
}
