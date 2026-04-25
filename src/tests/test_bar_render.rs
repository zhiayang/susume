// test_bar_render.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use crate::ProgressBar;
use crate::RenderTarget;

#[test]
fn test_render_simple()
{
	fn bar(pos: u64) -> String
	{
		let mut target = RenderTarget::string();
		let bar = ProgressBar::new("", Some(100))
			.with_template("[{bar:50}] {pos}/{len}")
			.with_progress_bar_chars(&["=", "-"]);

		bar.set_position(pos);
		bar.render(&mut target);

		return target.get_string().unwrap().trim().to_string();
	}

	for fill in 0..=100 {
		let expected = format!(
			"[{l}{r}] {fill}/100",
			l = "=".repeat(fill / 2),
			r = "-".repeat(50 - (fill / 2))
		);

		assert_eq!(bar(fill as u64), expected);
	}
}

#[test]
fn test_render_frac()
{
	fn bar(pos: u64) -> String
	{
		let mut target = RenderTarget::string();
		let bar = ProgressBar::new("", Some(20))
			.with_template("[{bar:2}] {pos:02}/{len}")
			.with_progress_bar_chars(&["=", "9", "8", "7", "6", "5", "4", "3", "2", "1", "-"]);

		bar.set_position(pos);
		bar.render(&mut target);

		return target.get_string().unwrap().trim().to_string();
	}

	assert_eq!(bar(00), "[--] 00/20");
	assert_eq!(bar(01), "[1-] 01/20");
	assert_eq!(bar(02), "[2-] 02/20");
	assert_eq!(bar(03), "[3-] 03/20");
	assert_eq!(bar(04), "[4-] 04/20");
	assert_eq!(bar(05), "[5-] 05/20");
	assert_eq!(bar(06), "[6-] 06/20");
	assert_eq!(bar(07), "[7-] 07/20");
	assert_eq!(bar(08), "[8-] 08/20");
	assert_eq!(bar(09), "[9-] 09/20");
	assert_eq!(bar(10), "[=-] 10/20");
	assert_eq!(bar(11), "[=1] 11/20");
	assert_eq!(bar(12), "[=2] 12/20");
	assert_eq!(bar(13), "[=3] 13/20");
	assert_eq!(bar(14), "[=4] 14/20");
	assert_eq!(bar(15), "[=5] 15/20");
	assert_eq!(bar(16), "[=6] 16/20");
	assert_eq!(bar(17), "[=7] 17/20");
	assert_eq!(bar(18), "[=8] 18/20");
	assert_eq!(bar(19), "[=9] 19/20");
	assert_eq!(bar(20), "[==] 20/20");
}
