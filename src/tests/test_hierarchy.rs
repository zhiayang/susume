// test_hierarchy.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use crate::ProgressBar;
use crate::RenderTarget;

#[test]
fn test_visible_descendant_count_all_visible()
{
	let top = ProgressBar::new("top", None);
	let bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_b = top.add_child(ProgressBar::new("B", None));
	let _bar_x = bar_b.add_child(ProgressBar::new("X", None));
	let _bar_y = bar_b.add_child(ProgressBar::new("Y", None));
	let bar_c = top.add_child(ProgressBar::new("C", None));
	let _bar_d = top.add_child(ProgressBar::new("D", None));

	assert_eq!(top.visible_descendant_count(), 7);
	assert_eq!(bar_a.visible_descendant_count(), 1);
	assert_eq!(bar_b.visible_descendant_count(), 3);
	assert_eq!(bar_c.visible_descendant_count(), 1);
}

#[test]
fn test_visible_descendant_count_hidden_leaf()
{
	let top = ProgressBar::new("top", None);
	let _bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_b = top.add_child(ProgressBar::new("B", None).hidden());

	// B is hidden: visible count for top should exclude B
	assert_eq!(bar_b.visible_descendant_count(), 0);
	assert_eq!(top.visible_descendant_count(), 2); // top + A
}

#[test]
fn test_visible_descendant_count_hidden_parent_visible_children()
{
	// Hidden parent with visible children -- children are still rendered
	let top = ProgressBar::new("top", None).hidden();
	let _bar_a = top.add_child(ProgressBar::new("A", None));
	let _bar_b = top.add_child(ProgressBar::new("B", None));
	let _bar_c = top.add_child(ProgressBar::new("C", None));

	// top is hidden (0), but children are visible (3 total)
	assert_eq!(top.visible_descendant_count(), 3);
}

#[test]
fn test_visible_descendant_count_hidden_subtree()
{
	let top = ProgressBar::new("top", None);
	let bar_b = top.add_child(ProgressBar::new("B", None).hidden());
	let _bar_x = bar_b.add_child(ProgressBar::new("X", None));
	let _bar_y = bar_b.add_child(ProgressBar::new("Y", None));

	// B is hidden, but X and Y are visible children of B
	assert_eq!(bar_b.visible_descendant_count(), 2); // X + Y (B itself hidden)
	assert_eq!(top.visible_descendant_count(), 3); // top + X + Y
}

#[test]
fn test_visible_descendant_count_all_hidden()
{
	let top = ProgressBar::new("top", None).hidden();
	let _bar_a = top.add_child(ProgressBar::new("A", None).hidden());
	let _bar_b = top.add_child(ProgressBar::new("B", None).hidden());

	assert_eq!(top.visible_descendant_count(), 0);
}

#[test]
fn test_visible_descendant_count_deep_nesting()
{
	let top = ProgressBar::new("top", None);
	let a = top.add_child(ProgressBar::new("A", None));
	let b = a.add_child(ProgressBar::new("B", None));
	let _c = b.add_child(ProgressBar::new("C", None));

	assert_eq!(top.visible_descendant_count(), 4); // top, A, B, C
	assert_eq!(a.visible_descendant_count(), 3);
	assert_eq!(b.visible_descendant_count(), 2);
}

#[test]
fn test_absolute_index_hidden_parent()
{
	// Hidden parent doesn't occupy a line
	let top = ProgressBar::new("top", None).hidden();
	let bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_b = top.add_child(ProgressBar::new("B", None));
	let bar_c = top.add_child(ProgressBar::new("C", None));

	// top is hidden: occupies 0 lines
	// Rendered:
	//   A  → line 0
	//   B  → line 1
	//   C  → line 2
	assert_eq!(top.absolute_index(), 0);
	assert_eq!(bar_a.absolute_index(), 0);
	assert_eq!(bar_b.absolute_index(), 1);
	assert_eq!(bar_c.absolute_index(), 2);
}

#[test]
fn test_absolute_index_hidden_sibling()
{
	let top = ProgressBar::new("top", None);
	let bar_a = top.add_child(ProgressBar::new("A", None).hidden());
	let bar_b = top.add_child(ProgressBar::new("B", None));
	let bar_c = top.add_child(ProgressBar::new("C", None));

	// Rendered:
	//   top → line 0
	//   (A hidden, no line)
	//   B   → line 1
	//   C   → line 2
	assert_eq!(top.absolute_index(), 0);
	assert_eq!(bar_a.absolute_index(), 1); // A is still "at" index 1 even though hidden
	assert_eq!(bar_b.absolute_index(), 1); // B takes A's slot since A is invisible
	assert_eq!(bar_c.absolute_index(), 2);
}

#[test]
fn test_absolute_index_hidden_sibling_with_children()
{
	let top = ProgressBar::new("top", None);
	let bar_a = top.add_child(ProgressBar::new("A", None));
	let bar_group = top.add_child(ProgressBar::new("group", None).hidden());
	let _bar_x = bar_group.add_child(ProgressBar::new("X", None));
	let _bar_y = bar_group.add_child(ProgressBar::new("Y", None));
	let bar_b = top.add_child(ProgressBar::new("B", None));

	// Rendered:
	//   top   → line 0
	//   A     → line 1
	//   (group hidden, no line)
	//   X     → line 2
	//   Y     → line 3
	//   B     → line 4
	assert_eq!(top.absolute_index(), 0);
	assert_eq!(bar_a.absolute_index(), 1);
	assert_eq!(bar_b.absolute_index(), 4);
}

#[test]
fn test_absolute_index_all_visible_unchanged()
{
	// Verify the original test case still works (regression test)
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
	assert_eq!(bar_x.absolute_index(), 3);
	assert_eq!(bar_y.absolute_index(), 4);
	assert_eq!(bar_c.absolute_index(), 5);
	assert_eq!(bar_p.absolute_index(), 6);
	assert_eq!(bar_q.absolute_index(), 7);
	assert_eq!(bar_d.absolute_index(), 8);
}

#[test]
fn test_render_hidden_parent_visible_children()
{
	let target = RenderTarget::string();
	let top = ProgressBar::new("top", Some(100)).with_template("{msg}").hidden();
	let _bar_a = top.add_child(ProgressBar::new("child_a", Some(100)).with_template("{msg}"));
	let _bar_b = top.add_child(ProgressBar::new("child_b", Some(100)).with_template("{msg}"));

	top.render(&target);
	let output = target.get_string().unwrap();

	// Hidden parent shouldn't appear; children should
	assert!(!output.contains("top"));
	assert!(output.contains("child_a"));
	assert!(output.contains("child_b"));
}

#[test]
fn test_render_hidden_child()
{
	let target = RenderTarget::string();
	let top = ProgressBar::new("top", Some(100)).with_template("{msg}");
	let _bar_a = top.add_child(ProgressBar::new("visible", Some(100)).with_template("{msg}"));
	let _bar_b = top.add_child(ProgressBar::new("hidden_child", Some(100)).with_template("{msg}").hidden());

	top.render(&target);
	let output = target.get_string().unwrap();

	assert!(output.contains("top"));
	assert!(output.contains("visible"));
	assert!(!output.contains("hidden_child"));
}

#[test]
fn test_render_line_count_accounts_for_hidden()
{
	let target = RenderTarget::string();
	let top = ProgressBar::new("top", Some(100)).with_template("{msg}").hidden();
	let _bar_a = top.add_child(ProgressBar::new("A", Some(100)).with_template("{msg}"));
	let _bar_b = top.add_child(ProgressBar::new("B", Some(100)).with_template("{msg}"));
	let _bar_c = top.add_child(ProgressBar::new("C", Some(100)).with_template("{msg}"));

	top.render(&target);

	// Only 3 children rendered (hidden parent produces no line)
	assert_eq!(target.line_count(), 3);
}

#[test]
fn test_render_line_count_all_visible()
{
	let target = RenderTarget::string();
	let top = ProgressBar::new("top", Some(100)).with_template("{msg}");
	let _bar_a = top.add_child(ProgressBar::new("A", Some(100)).with_template("{msg}"));
	let _bar_b = top.add_child(ProgressBar::new("B", Some(100)).with_template("{msg}"));

	top.render(&target);

	// Parent + 2 children = 3 lines
	assert_eq!(target.line_count(), 3);
}
