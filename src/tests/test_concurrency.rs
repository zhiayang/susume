// test_concurrency.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

use crate::ProgressBar;
use crate::RenderTarget;

#[test]
fn test_concurrent_render_never_exceeds_visible_line_count()
{
	// hidden parent + several visible children (some nested), mirroring the
	// GLOBAL_PARENT + file/part bars layout from norikae
	let top = ProgressBar::new("top", Some(100)).with_template("{msg}").hidden();
	let bar_a = top.add_child(ProgressBar::new("a", Some(100)).with_template("{msg}"));
	let _bar_b = top.add_child(ProgressBar::new("b", Some(100)).with_template("{msg}"));
	let _bar_c = bar_a.add_child(ProgressBar::new("c", Some(100)).with_template("{msg}"));
	let _bar_d = bar_a.add_child(ProgressBar::new("d", Some(100)).with_template("{msg}"));
	let _bar_e = top.add_child(ProgressBar::new("e", Some(100)).with_template("{msg}"));

	top.core.write().target = RenderTarget::string();

	// visible bars: a, b, c, d, e (top is hidden) => 5 lines per clean frame
	let expected = top.visible_descendant_count();
	assert_eq!(expected, 5);

	let max_seen = AtomicUsize::new(0);

	thread::scope(|scope| {
		for _ in 0..8 {
			scope.spawn(|| {
				for _ in 0..20_000 {
					// mirror the ticker: render the bar to its own target
					let core = top.core.read();
					core.render(&core.target);
					max_seen.fetch_max(core.target.line_count(), Ordering::Relaxed);
				}
			});
		}
	});

	let max = max_seen.load(Ordering::Relaxed);

	assert!(
		max <= expected,
		"observed line_count() = {max}, which exceeds the {expected} visible bars: \
		 two render frames interleaved and corrupted the shared cursor/line counter"
	);
}
