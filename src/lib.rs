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

pub mod fmt
{
	pub use crate::format::*;
	pub use crate::style::PlaceholderFormatter;
}
