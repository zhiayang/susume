// bytes.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::thread;
use std::time::Duration;

use susume::ProgressBar;

fn main()
{
	let bar = ProgressBar::new("", Some(300_000))
		.with_template("  [{bar:30%}] {pos_bytes}/{total_bytes} {pad:10}")
		.with_tick_frequency(20.0)
		.activated();

	for _ in 0..300 {
		bar.increment(1231);
		thread::sleep(Duration::from_millis(10));
	}

	thread::sleep(Duration::from_millis(500));

	for _ in 0..300 {
		bar.decrement(1231);
		thread::sleep(Duration::from_millis(10));
	}

	thread::sleep(Duration::from_millis(500));

	bar.finish();
}
