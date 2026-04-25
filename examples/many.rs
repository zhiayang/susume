// many.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::thread;
use std::time::Duration;

use susume::ProgressBar;

fn main()
{
	let parent = ProgressBar::new("", None).with_tick_frequency(10.0).hidden();

	let bar1 = parent.add_child(ProgressBar::new("", Some(300)));
	let bar2 = parent.add_child(ProgressBar::new("", Some(300)));
	let bar3 = parent.add_child(ProgressBar::new("", Some(300)));
	bar2.set_position(300);

	parent.activate();

	for _ in 0..3 {
		for _ in 0..300 {
			bar1.increment(1);
			bar2.decrement(1);
			bar3.increment(1);
			thread::sleep(Duration::from_millis(10));
		}

		thread::sleep(Duration::from_millis(500));

		for _ in 0..300 {
			bar1.decrement(1);
			bar2.increment(1);
			bar3.decrement(1);
			thread::sleep(Duration::from_millis(10));
		}

		thread::sleep(Duration::from_millis(500));
	}
}
