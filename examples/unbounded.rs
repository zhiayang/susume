// unbounded.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::thread;
use std::time::Duration;

use susume::ProgressBar;

fn main()
{
	let bar = ProgressBar::new("", None).with_tick_interval(Duration::from_millis(150)).activated();

	for _ in 0..5000 {
		bar.increment(1);
		thread::sleep(Duration::from_millis(10));
	}
}
