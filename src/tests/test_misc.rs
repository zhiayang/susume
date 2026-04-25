// test_misc.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use crate::ProgressBar;


#[test]
fn test_descendant_count()
{
	let top = ProgressBar::new("top", None);
	let bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_b = top.add_child(ProgressBar::new("B", None));
	let bar_x = bar_b.add_child(ProgressBar::new("X", None));
	let bar_y = bar_b.add_child(ProgressBar::new("Y", None));
	let bar_c = top.add_child(ProgressBar::new("C", None));
	let bar_p = bar_c.add_child(ProgressBar::new("P", None));
	let bar_q = bar_c.add_child(ProgressBar::new("Q", None));
	let bar_d = top.add_child(ProgressBar::new("D", None));

	assert_eq!(top.descendant_count(), 9);
	assert_eq!(bar_a.descendant_count(), 1);
	assert_eq!(bar_b.descendant_count(), 3);
	assert_eq!(bar_c.descendant_count(), 3);
	assert_eq!(bar_d.descendant_count(), 1);
	assert_eq!(bar_x.descendant_count(), 1);
	assert_eq!(bar_y.descendant_count(), 1);
	assert_eq!(bar_p.descendant_count(), 1);
	assert_eq!(bar_q.descendant_count(), 1);
}

#[test]
fn test_topmost_bar()
{
	let top = ProgressBar::new("top", None);
	let bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_b = top.add_child(ProgressBar::new("B", None));
	let bar_x = bar_b.add_child(ProgressBar::new("X", None));
	let bar_y = bar_b.add_child(ProgressBar::new("Y", None));
	let bar_c = top.add_child(ProgressBar::new("C", None));
	let bar_p = bar_c.add_child(ProgressBar::new("P", None));
	let bar_q = bar_c.add_child(ProgressBar::new("Q", None));
	let bar_d = top.add_child(ProgressBar::new("D", None));

	let top_id = top.core.read().id;
	assert_eq!(top.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_a.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_b.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_c.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_d.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_x.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_y.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_p.topmost_bar().core.read().id, top_id);
	assert_eq!(bar_q.topmost_bar().core.read().id, top_id);
}


#[test]
fn test_parent_index()
{
	let top = ProgressBar::new("top", None);
	let bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_b = top.add_child(ProgressBar::new("B", None));
	let bar_x = bar_b.add_child(ProgressBar::new("X", None));
	let bar_y = bar_b.add_child(ProgressBar::new("Y", None));
	let bar_c = top.add_child(ProgressBar::new("C", None));
	let bar_p = bar_c.add_child(ProgressBar::new("P", None));
	let bar_q = bar_c.add_child(ProgressBar::new("Q", None));
	let bar_d = top.add_child(ProgressBar::new("D", None));

	assert_eq!(top.parent_index(), 0);
	assert_eq!(bar_a.parent_index(), 1);
	assert_eq!(bar_b.parent_index(), 2);
	assert_eq!(bar_c.parent_index(), 3);
	assert_eq!(bar_d.parent_index(), 4);
	assert_eq!(bar_x.parent_index(), 1);
	assert_eq!(bar_y.parent_index(), 2);
	assert_eq!(bar_p.parent_index(), 1);
	assert_eq!(bar_q.parent_index(), 2);
}

#[test]
fn test_absolute_index()
{
	let top = ProgressBar::new("top", None);
	let bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_b = top.add_child(ProgressBar::new("B", None));
	let bar_x = bar_b.add_child(ProgressBar::new("X", None));
	let bar_y = bar_b.add_child(ProgressBar::new("Y", None));
	let bar_c = top.add_child(ProgressBar::new("C", None));
	let bar_p = bar_c.add_child(ProgressBar::new("P", None));
	let bar_q = bar_c.add_child(ProgressBar::new("Q", None));
	let bar_d = top.add_child(ProgressBar::new("D", None));

	assert_eq!(top.absolute_index(), 0);
	assert_eq!(bar_a.absolute_index(), 1);
	assert_eq!(bar_b.absolute_index(), 2);
	assert_eq!(bar_c.absolute_index(), 5);
	assert_eq!(bar_d.absolute_index(), 8);
	assert_eq!(bar_x.absolute_index(), 3);
	assert_eq!(bar_y.absolute_index(), 4);
	assert_eq!(bar_p.absolute_index(), 6);
	assert_eq!(bar_q.absolute_index(), 7);
}
