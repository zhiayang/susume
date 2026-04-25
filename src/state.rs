// state.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub struct State
{
	pub position: AtomicU64,
	pub ticks: AtomicU64,
	pub total: Option<u64>,
}

impl State
{
	/// Returns the completion of this bar as a number between 0.0 and 1.0, inclusive.
	#[allow(clippy::cast_precision_loss)]
	pub fn fraction(&self) -> f64
	{
		let pos = self.position.load(Ordering::Relaxed);
		return match self.total {
			Some(0) => 1.0,
			Some(len) => (pos as f64) / (len as f64),
			None => 0.0,
		};
	}
}
