// progress_bar.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;
use std::sync::Weak;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use parking_lot::MappedRwLockReadGuard;
use parking_lot::MappedRwLockWriteGuard;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;

use crate::Estimator;
use crate::RenderTarget;
use crate::State;
use crate::Style;
use crate::style::PlaceholderFormatter;
use crate::ticker::Ticker;

/// A progress bar. Cheap to clone if necessary.
#[derive(Clone)]
pub struct ProgressBar
{
	pub(crate) core: Arc<RwLock<ProgressBarCore>>,
}

/// An encapsulation of the renderable attributes of a progress bar.
pub struct ProgressBarAttribs
{
	pub active: bool,
	pub hidden: bool,

	pub state: State,
	pub style: Style,
	pub prefix: String,
	pub message: String,
	pub estimator: Estimator,
}

pub(crate) struct ProgressBarCore
{
	pub(crate) id: usize,
	pub(crate) target: RenderTarget,
	pub(crate) attribs: ProgressBarAttribs,
	pub(crate) linked_parent: bool,
	pub(crate) finished: bool,

	pub(crate) ticker: Option<Ticker>,
	pub(crate) parent: Option<Weak<RwLock<ProgressBarCore>>>,
	pub(crate) children: Vec<(u32, Arc<RwLock<ProgressBarCore>>)>,
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl ProgressBar
{
	/// Creates a new progress bar with a set total length and the default style that starts
	/// in a deactivated state. Call [`Self::activate`] after setting the tick interval to activate it.
	///
	/// If `total` is `None`, creates an unbounded bar that might render differently depending
	/// on the style.
	#[must_use]
	pub fn new<S: AsRef<str>>(message: S, total: Option<u64>) -> Self
	{
		return Self {
			core: Arc::new(RwLock::new(ProgressBarCore {
				id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
				attribs: ProgressBarAttribs {
					active: false,
					hidden: false,
					state: State { position: AtomicU64::new(0), ticks: AtomicU64::new(0), total },
					style: Style::default_bar(),
					prefix: String::new(),
					message: message.as_ref().to_string(),
					estimator: Estimator::default(),
				},
				linked_parent: false,
				finished: false,
				target: RenderTarget::default(),
				ticker: None,

				parent: None,
				children: vec![],
			})),
		};
	}

	/// Creates a new spinner using the default style that starts deactivated.
	///
	/// Under most circumstances, you should set the interval and then [`Self::activate`] the bar
	/// so that the spinner animation will play without manual intervention.
	///
	/// Note that spinners by default do not have an estimator.
	#[must_use]
	pub fn new_spinner<S: AsRef<str>>(message: S) -> Self
	{
		return Self {
			core: Arc::new(RwLock::new(ProgressBarCore {
				id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
				attribs: ProgressBarAttribs {
					active: false,
					hidden: false,
					state: State {
						position: AtomicU64::new(0),
						ticks: AtomicU64::new(0),
						total: None,
					},
					style: Style::default_spinner(),
					prefix: String::new(),
					message: message.as_ref().to_string(),
					estimator: Estimator::none(),
				},
				linked_parent: false,
				finished: false,
				target: RenderTarget::default(),
				ticker: None,

				parent: None,
				children: vec![],
			})),
		};
	}

	/// Creates a dummy progress bar that does nothing and will not render.
	#[must_use]
	pub fn dummy() -> Self
	{
		return Self {
			core: Arc::new(RwLock::new(ProgressBarCore {
				id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
				attribs: ProgressBarAttribs {
					active: false,
					hidden: false,
					state: State {
						position: AtomicU64::new(0),
						ticks: AtomicU64::new(0),
						total: None,
					},
					style: Style::dummy(),
					prefix: String::new(),
					message: String::new(),
					estimator: Estimator::none(),
				},
				linked_parent: false,
				finished: false,

				target: RenderTarget::none(),
				ticker: None,
				parent: None,
				children: vec![],
			})),
		}
		.hidden();
	}

	fn add_child_1(&self, indent: Option<u32>, child: ProgressBar) -> Self
	{
		{
			// disable the child's ticker
			let mut child_ticker = {
				let mut child_core = child.core.write();
				child_core.parent = Some(Arc::downgrade(&self.core));
				child_core.target = RenderTarget::none();

				child_core.ticker.take()
			};

			if let Some(ticker) = child_ticker.take() {
				ticker.stop();
			}
		}

		self.core.write().children.push((indent.unwrap_or(0), child.core.clone()));
		return child;
	}

	/// Adds a new progres bar as a child of this bar, with indentation when drawn.
	#[must_use]
	pub fn add_indented_child(&self, indent: u32, child: ProgressBar) -> Self
	{
		return self.add_child_1(Some(indent), child);
	}

	/// Adds a new progres bar as a child of this bar.
	#[must_use]
	pub fn add_child(&self, child: ProgressBar) -> Self
	{
		return self.add_child_1(None, child);
	}

	/// Returns an immutable reference to the current style.
	pub fn style(&self) -> MappedRwLockReadGuard<'_, Style>
	{
		return RwLockReadGuard::map(self.core.read(), |s| &s.attribs.style);
	}

	/// Returns a mutable reference to the current style.
	pub fn style_mut(&self) -> MappedRwLockWriteGuard<'_, Style>
	{
		return RwLockWriteGuard::map(self.core.write(), |s| &mut s.attribs.style);
	}

	/// Returns a copy of the progress bar's message.
	pub fn message(&self) -> String
	{
		return self.core.read().attribs.message.clone();
	}

	/// Sets the progress bar's message.
	pub fn set_message(&self, msg: String)
	{
		self.core.write().attribs.message = msg;
	}

	/// Returns the current tick interval, if one was set.
	pub fn tick_interval(&self) -> Option<Duration>
	{
		return self.core.read().ticker.as_ref().map(|ticker| ticker.interval());
	}

	/// Sets the tick interval of the progress bar. If it was not previously set,
	/// a new ticker is created at the given interval.
	pub fn set_tick_interval(&mut self, interval: Duration)
	{
		if self.is_child() {
			return;
		}

		let mut core = self.core.write();
		if let Some(ticker) = &core.ticker {
			ticker.set_interval(interval);
		} else {
			core.ticker = Some(Ticker::with_interval(interval));
		}
	}

	/// Returns `true` if this progress bar is a child of another.
	pub fn is_child(&self) -> bool
	{
		return self.core.read().parent.is_some();
	}

	/// Activates the progress. If the tick interval was not set, this
	/// function does nothing.
	#[must_use]
	pub fn activated(self) -> Self
	{
		self.activate();
		return self;
	}

	/// Links this progress bar to its parent, such that incrementing this bar
	/// also increments the parent bar (and same for decrementing).
	#[must_use]
	pub fn linked_to_parent(self) -> Self
	{
		self.core.write().linked_parent = true;
		return self;
	}

	/// Hides the progress bar.
	#[must_use]
	pub fn hidden(self) -> Self
	{
		self.hide(true);
		return self;
	}

	/// Returns `true` if the bar is hidden.
	pub fn is_hidden(&self) -> bool
	{
		return self.core.read().attribs.hidden;
	}

	/// Sets the tick interval, allowing the bar to automatically update and render at the given interval.
	/// If the interval `None`, disables ticking.
	///
	/// If this bar is a child of another one, ticking is disabled, and this function does nothing.
	#[must_use]
	pub fn with_tick_interval(self, interval: Duration) -> Self
	{
		self.core.write().ticker = if self.is_child() {
			None
		} else {
			Some(Ticker::with_interval(interval))
		};

		return self;
	}

	/// Sets the (optional) tick frequency in Hz. See [`Self::with_tick_interval`].
	#[must_use]
	pub fn with_tick_frequency(self, frequency: f64) -> Self
	{
		return self.with_tick_interval(Duration::from_secs_f64(1.0 / frequency));
	}

	/// Sets the style of the progress bar.
	#[must_use]
	pub fn with_style(self, style: Style) -> Self
	{
		self.core.write().attribs.style = style;
		return self;
	}

	/// Sets the template for the progress bar's style. A convenience method to operate on the style.
	#[must_use]
	pub fn with_template<S: AsRef<str>>(self, template: S) -> Self
	{
		self.core.write().attribs.style.set_template(template);
		return self;
	}

	/// Sets the spinner characters to use. A convenience method to operate on the style.
	#[must_use]
	pub fn with_spinner_chars<S: AsRef<str>>(self, chars: &[S]) -> Self
	{
		self.style_mut().set_spinner_chars(chars);
		return self;
	}

	/// Sets the progress bar characters to use. A convenience method to operate on the style.
	#[must_use]
	pub fn with_progress_bar_chars<S: AsRef<str>>(self, chars: &[S]) -> Self
	{
		self.style_mut().set_progress_bar_chars(chars);
		return self;
	}

	/// Sets the bouncer string to use. A convenience method to operate on the style.
	#[must_use]
	pub fn with_bouncer<S: AsRef<str>>(self, bouncer: S) -> Self
	{
		self.style_mut().set_bouncer(bouncer);
		return self;
	}

	/// Sets the formatter for the progress bar's style. A convenience method to operate on the style.
	#[must_use]
	pub fn with_formatter(self, formatter: Box<dyn PlaceholderFormatter>) -> Self
	{
		self.core.write().attribs.style.add_formatter(formatter);
		return self;
	}

	/// Sets the message of the progress bar. Note that this is only drawn if the
	/// style has a template including the message.
	#[must_use]
	pub fn with_message(self, message: String) -> Self
	{
		self.core.write().attribs.message = message;
		return self;
	}

	/// Sets the estimator of the progress bar.
	#[must_use]
	pub fn with_estimator(self, estimator: Estimator) -> Self
	{
		self.core.write().attribs.estimator = estimator;
		return self;
	}
}


impl ProgressBar
{
	/// Hides or unhides the progress bar.
	pub fn hide(&self, hide: bool)
	{
		self.core.write().attribs.hidden = hide;
	}

	/// Activates the progress bar, automatically updating it if a tick interval or frequency was set.
	pub fn activate(&self)
	{
		let mut core = self.core.write();
		core.attribs.active = true;

		if let Some(ticker) = &core.ticker {
			ticker.run(&self.core);
		}
	}

	/// Deactivates the progress bar.
	pub fn deactivate(&self)
	{
		let ticker = {
			let mut core = self.core.write();
			core.attribs.active = false;
			core.ticker.clone()
		};

		if let Some(ticker) = ticker {
			ticker.stop();
		}
	}

	/// Finishes the progress bar, removing it from the screen. Once the bar is finished,
	/// it should not be used again.
	///
	/// Effectively hides and deactivates the progress bar. If this was a child bar,
	/// it is removed from its parent.
	pub fn finish(self)
	{
		if self.core.read().finished {
			return;
		}

		self.hide(true);
		self.deactivate();

		self.clear();
		self.core.write().finished = true;
	}

	/// Finishes the progress bar, keeping it on the screen.
	pub fn finish_and_keep(self)
	{
		if self.core.read().finished {
			return;
		}

		self.hide(true);
		self.deactivate();
		self.core.write().finished = true;
	}

	/// Finishes the progress bar, replacing it with the given message.
	pub fn finish_and_replace<S: AsRef<str>>(self, message: S)
	{
		if self.core.read().finished {
			return;
		}

		self.hide(true);
		self.deactivate();
		self.core.write().finished = true;

		let top = self.topmost_bar();
		top.core.read().target.write_line_at(self.absolute_index(), message.as_ref());
	}

	/// Clears the progress bar, but does not finish it. If not deactivated before clearing,
	/// it will be re-drawn on the next render.
	pub fn clear(&self)
	{
		let top = self.topmost_bar();
		let top = top.core.read();

		top.target.reset(/* clear: */ true, /* flush: */ false);
		top.render(&top.target);
	}

	/// Detaches this progress bar from its parent. The bar will be in a deactivated state
	/// with no `RenderTarget`.
	///
	/// # Panics
	/// Panics if this bar does not have a parent, or if its parent was already finished,
	/// or if the parent does not have this bar as a child (should be impossible).
	#[must_use]
	pub fn detach_from_parent(self) -> Self
	{
		let top = self.topmost_bar();
		let abs_idx = self.absolute_index();

		let self_id = self.core.read().id;

		let parent = self.core.write().parent.take().expect("bar does not have a parent");
		let parent = Weak::upgrade(&parent).expect("parent disappeared");

		{
			let mut parent = parent.write();
			let idx = parent
				.children
				.iter()
				.position(|(_, x)| x.read().id == self_id)
				.expect("parent did not contain this child");

			parent.children.remove(idx);
		}

		top.core.read().target.remove_line(abs_idx);
		return self;
	}


	/// Returns an immutable reference to the current state of the progress bar.
	pub fn state(&self) -> MappedRwLockReadGuard<'_, State>
	{
		return RwLockReadGuard::map(self.core.read(), |s| &s.attribs.state);
	}

	/// Returns the current position of the progress bar.
	pub fn position(&self) -> u64
	{
		return self.core.read().attribs.state.position.load(Ordering::Acquire);
	}

	/// Sets the current position of the progress bar.
	pub fn set_position(&self, pos: u64)
	{
		let now = Instant::now();

		let attribs = &mut self.core.write().attribs;
		attribs.state.position.store(pos, Ordering::Release);
		attribs.estimator.reset(now);
	}

	/// Resets the current position to 0.
	pub fn reset(&self)
	{
		let now = Instant::now();

		let attribs = &mut self.core.write().attribs;
		attribs.state.position.store(0, Ordering::Release);
		attribs.estimator.reset(now);
	}

	/// Increments the current position of the progress bar by the delta.
	pub fn increment(&self, delta: u64)
	{
		{
			let now = Instant::now();
			let core = &mut self.core.write();

			let attribs = &mut core.attribs;
			attribs.state.position.fetch_add(delta, Ordering::AcqRel);
			attribs.estimator.update(now, delta);
		}

		let core = &self.core.read();
		if core.linked_parent
			&& let Some(parent) = &core.parent
			&& let Some(parent) = Weak::upgrade(parent)
		{
			Self::from_core(parent).increment(delta);
		}
	}

	/// Decrements the current position of the progress bar by the delta.
	///
	/// # Panics
	/// Shouldn't panic.
	pub fn decrement(&self, delta: u64)
	{
		let now = Instant::now();

		let attribs = &mut self.core.write().attribs;
		attribs
			.state
			.position
			.fetch_update(Ordering::AcqRel, Ordering::Relaxed, |current| Some(current.saturating_sub(delta)))
			.unwrap();
		attribs.estimator.reset(now);
	}

	/// Returns the current total (length) of the progress bar.
	pub fn total(&self) -> Option<u64>
	{
		return self.core.read().attribs.state.total;
	}

	/// Sets the total (length) of the progress bar.
	pub fn set_total(&self, len: u64)
	{
		return self.core.write().attribs.state.total = Some(len);
	}

	/// Unsets the total (length) of the progress bar, making the bar unbounded.
	pub fn unset_total(&self)
	{
		return self.core.write().attribs.state.total = None;
	}

	/// An alias for [`Self::unset_total`], making the bar unbounded.
	pub fn make_unbounded(&self)
	{
		self.unset_total();
	}

	/// Ticks the progress bar, advancing any animation state without changing the actual progress.
	pub fn tick(&self)
	{
		self.core.read().tick();
	}

	/// Gets the parent of this progress bar, if one exists.
	pub fn parent(&self) -> Option<Self>
	{
		let Some(parent) = &self.core.read().parent else {
			return None;
		};

		let parent = Weak::upgrade(parent)?;

		return Some(Self::from_core(parent));
	}

	/// Gets the top-most bar in a progress-bar hierarchy, or returns `self` if it was
	/// already the topmost bar.
	#[must_use]
	pub fn topmost_bar(&self) -> Self
	{
		// call recursively. we don't expect too many
		// levels of bar nesting, so this should be safe.
		return self.parent().map(|p| p.topmost_bar()).unwrap_or(self.clone());
	}

	/// Calculates the index of this progress bar relative to its parent, if it
	/// is a child bar. Returns 0 if the bar is not a child, 1 if it is the first child, etc.
	///
	/// # Panics
	/// Panics if this bar had a parent, but was not present in the parent's children list.
	pub fn parent_index(&self) -> usize
	{
		let self_id = self.core.read().id;
		let Some(parent) = self.parent() else {
			return 0;
		};

		for (idx, (_, child)) in parent.core.read().children.iter().enumerate() {
			if child.read().id == self_id {
				return 1 + idx;
			}
		}

		unreachable!("child bar {self_id} was not found in parent");
	}

	/// Calculates the absolute index of this progress bar relative to the topmost
	/// ancestor bar. If this bar is already the topmost, returns 0.
	///
	/// For example:
	///
	/// ```text
	/// <topmost>       // #0
	///   <bar A>       // #1
	///   <bar B>       // #2
	///      <bar X>    // #3
	///      <bar Y>    // #4
	///   <bar C>       // #5
	///   <bar D>       // #6
	/// ```
	pub fn absolute_index(&self) -> usize
	{
		let Some(parent) = self.parent() else {
			return 0;
		};

		let parent_visible = usize::from(!parent.is_hidden());
		let mut idx = parent_visible + parent.absolute_index();

		for (_, sibling) in parent.core.read().children.iter().take(self.parent_index() - 1) {
			idx += Self::from_core(sibling.clone()).visible_descendant_count();
		}

		return idx;
	}

	/// Calculates the total number of progress bars, including itself, in this bar and
	/// all its children recursively. If this bar has no children, returns 1.
	pub fn descendant_count(&self) -> usize
	{
		return 1 + self
			.core
			.read()
			.children
			.iter()
			.map(|c| Self::from_core(c.1.clone()).descendant_count())
			.sum::<usize>();
	}

	/// Calculates the total number of *visible* progress bars, including itself,
	/// in this bar and all its children recursively. Similar to [`descendant_count`],
	/// but accounts for hidden bars.
	pub fn visible_descendant_count(&self) -> usize
	{
		let self_count = usize::from(!self.is_hidden());

		return self
			.core
			.read()
			.children
			.iter()
			.map(|c| Self::from_core(c.1.clone()).visible_descendant_count())
			.sum::<usize>()
			+ self_count;
	}

	pub(crate) fn from_core(core: Arc<RwLock<ProgressBarCore>>) -> Self
	{
		return Self { core };
	}
}

pub(crate) static GLOBAL_PAUSE: AtomicUsize = AtomicUsize::new(0);

impl ProgressBar
{
	/// Pauses all rendering of progress bars. Returns a guard that will unpause
	/// the progress bars when dropped.
	pub fn pause_all() -> PauseGuard
	{
		Self::pause_all_raw();
		return PauseGuard();
	}

	/// Pauses all rendering of progress bars.
	pub fn pause_all_raw()
	{
		if GLOBAL_PAUSE.fetch_add(1, Ordering::AcqRel) == 0 {
			// if we were the first to pause, then clear the stderr and stdout render targets.
			RenderTarget::stderr().reset(/* clear: */ true, /* flush: */ true);
			RenderTarget::stdout().reset(/* clear: */ true, /* flush: */ true);
		}
	}

	/// Unpauses all rendering of progress bars. Should be paired with a prior call to
	/// `pause_all_raw`.
	pub fn unpause_all_raw()
	{
		GLOBAL_PAUSE.fetch_sub(1, Ordering::AcqRel);
	}
}

pub struct PauseGuard();

impl Drop for PauseGuard
{
	fn drop(&mut self)
	{
		ProgressBar::unpause_all_raw();
	}
}

impl ProgressBarCore
{
	pub(crate) fn tick(&self)
	{
		self.attribs.state.ticks.fetch_add(1, Ordering::AcqRel);
	}
}

impl Eq for ProgressBarCore {}

impl PartialEq for ProgressBarCore
{
	fn eq(&self, other: &Self) -> bool
	{
		return self.id == other.id;
	}
}
