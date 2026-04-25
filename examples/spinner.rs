// spinner.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::thread;
use std::time::Duration;

use susume::ProgressBar;

fn main()
{
	let bar = ProgressBar::new_spinner("hewwo").with_tick_frequency(10.0).activated();

	thread::sleep(Duration::from_millis(10_000));
	bar.finish();
}
