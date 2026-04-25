// target.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use std::sync::LazyLock;

use parking_lot::Mutex;

enum Target
{
	None,
	Term(console::Term),
	String(Mutex<String>),
}

/// A rendering target, typically either a terminal (eg. stderr or stdout), a string,
/// (mainly for testing/debugging), or nothing (a void).
pub struct RenderTarget
{
	core: Arc<RenderTargetCore>,
}

struct RenderTargetCore
{
	target: Target,
	lines: Mutex<usize>,
}

impl Default for RenderTarget
{
	fn default() -> Self
	{
		// check if stderr is interactive; if not, don't render anything at all.
		if !console::Term::stderr().is_term() {
			return Self::none();
		}

		return Self::stderr();
	}
}

impl RenderTarget
{
	/// Returns the current width of the render target. For interactive terminals, this is queried
	/// through the window size if possible. For other targets, it is fixed at 120.
	pub fn width(&self) -> usize
	{
		return match &self.core.target {
			Target::None | Target::String(_) => 120,
			Target::Term(term) => term.size().1 as usize,
		};
	}

	/// Flushes the target if it is a terminal.
	pub fn flush(&self)
	{
		if let Target::Term(term) = &self.core.target {
			_ = term.flush();
		}
	}

	/// Writes a line to the render target.
	pub fn write_line(&self, line: &str)
	{
		let line_width = console::measure_text_width(line);
		let extra_padding = self.width().saturating_sub(line_width);

		// lock while writing the line.
		let lines = &mut *self.core.lines.lock();

		match &self.core.target {
			Target::None => {}
			Target::String(s) => _ = write!(&mut s.lock(), "{line}{empty:<extra_padding$}", empty = ""),
			Target::Term(term) => {
				_ = term.write_str(&format!("{line}{empty:<extra_padding$}", empty = ""));
				_ = term.flush();
			}
		}

		*lines += 1;
	}

	/// Resets the render target, getting it ready for a new render operation.
	pub fn reset(&self, clear: bool, flush: bool)
	{
		let lines = &mut *self.core.lines.lock();
		match &self.core.target {
			Target::None => {}
			Target::String(s) => s.lock().clear(),
			Target::Term(term) => {
				if clear {
					_ = term.clear_line();
				} else {
					_ = term.write_str("\r");
				}

				for _ in 1..*lines {
					_ = term.move_cursor_up(1);
					if clear {
						_ = term.clear_line();
					} else {
						_ = term.write_str("\r");
					}
				}

				if flush {
					_ = term.flush();
				}
			}
		}

		*lines = 0;
	}

	/// Erases the line at the given index, where 0 is the first (top) line written. Does
	/// nothing if the number of lines drawn was less than `line_idx`.
	///
	/// Does nothing if the render target is not a terminal.
	pub(crate) fn erase_line(&self, line_idx: usize)
	{
		let Target::Term(term) = &self.core.target else {
			return;
		};

		// lock while erasing
		let lines = &*self.core.lines.lock();

		if line_idx < *lines {
			// by default the cursor is left on the last line, so start from the back.
			let up = (*lines - line_idx) - 1;

			_ = term.move_cursor_up(up);
			_ = term.clear_line();
			_ = term.move_cursor_down(up);
			_ = term.flush();
		}
	}

	/// Erases the line at the given index, where 0 is the first (top) line written. Does
	/// nothing if the number of lines drawn was less than `line_idx`.
	///
	/// Does nothing if the render target is not a terminal.
	pub(crate) fn write_line_at(&self, line_idx: usize, line: &str)
	{
		let Target::Term(term) = &self.core.target else {
			return;
		};

		// lock while erasing
		let lines = &*self.core.lines.lock();

		if line_idx < *lines {
			// by default the cursor is left on the last line, so start from the back.
			let up = (*lines - line_idx) - 1;

			_ = term.move_cursor_up(up);
			_ = term.clear_line();
			_ = term.write_str(line);
			_ = term.move_cursor_down(up);
			_ = term.flush();
		}
	}
}

impl RenderTarget
{
	pub fn none() -> Self
	{
		return Self {
			core: Arc::new(RenderTargetCore { target: Target::None, lines: Mutex::new(0) }),
		};
	}

	pub fn stdout() -> Self
	{
		static GLOBAL_STDOUT: LazyLock<Arc<RenderTargetCore>> = LazyLock::new(|| {
			Arc::new(RenderTargetCore {
				target: Target::Term(console::Term::buffered_stdout()),
				lines: Mutex::new(0),
			})
		});

		return Self { core: GLOBAL_STDOUT.clone() };
	}

	pub fn stderr() -> Self
	{
		static GLOBAL_STDERR: LazyLock<Arc<RenderTargetCore>> = LazyLock::new(|| {
			Arc::new(RenderTargetCore {
				target: Target::Term(console::Term::buffered_stderr()),
				lines: Mutex::new(0),
			})
		});

		return Self { core: GLOBAL_STDERR.clone() };
	}

	pub fn string() -> Self
	{
		return Self {
			core: Arc::new(RenderTargetCore {
				target: Target::String(Mutex::new(String::new())),
				lines: Mutex::new(0),
			}),
		};
	}

	pub fn get_string(&self) -> Option<String>
	{
		return match &self.core.target {
			Target::None | Target::Term(_) => None,
			Target::String(s) => Some(s.lock().clone()),
		};
	}
}
