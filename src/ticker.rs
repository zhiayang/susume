// ticker.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use parking_lot::Condvar;
use parking_lot::Mutex;
use parking_lot::RwLock;

use crate::progress_bar::ProgressBarCore;

#[derive(Clone)]
pub(crate) struct Ticker
{
	interval: Arc<RwLock<Duration>>,

	join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
	stop_flag: Arc<(Mutex<bool>, Condvar)>,
}

impl Ticker
{
	pub(crate) fn run(&self, bar: &Arc<RwLock<ProgressBarCore>>)
	{
		let interval = self.interval.clone();
		let stop_flag = self.stop_flag.clone();

		let bar = Arc::downgrade(bar);

		*self.join_handle.lock() = Some(std::thread::spawn(move || {
			loop {
				if let Some(bar) = bar.upgrade() {
					let bar = bar.read();
					bar.tick();
					bar.render(&bar.target);
				} else {
					// the bar got yeeted, bail.
					break;
				}

				let timeout = *interval.read();

				// wait for the interval to pass.
				let (mutex, cv) = &*stop_flag;

				let mut stop_lock = mutex.lock();
				let result = cv.wait_for(&mut stop_lock, timeout);

				// if we timed out, then nobody woke us up by stopping, so we should keep going.
				if result.timed_out() && !*stop_lock {
					continue;
				}

				// stop is true, bail.
				if *stop_lock {
					break;
				}

				// parking_lot::Condvar is not supposed to have spurious wakeups,
				// so if we were woken from a not-timeout then stop was set.
				//
				// under normal circumstances we should not reach here.
				debug_assert!(false);
			}
		}));
	}

	pub(crate) fn stop(&self)
	{
		if let Some(handle) = self.join_handle.lock().take() {
			*self.stop_flag.0.lock() = true;
			self.stop_flag.1.notify_all();

			_ = handle.join();

			// reset the stop flag.
			*self.stop_flag.0.lock() = false;
		}
	}

	pub(crate) fn interval(&self) -> Duration
	{
		return *self.interval.read();
	}

	pub(crate) fn set_interval(&self, interval: Duration)
	{
		*self.interval.write() = interval;
	}

	pub(crate) fn with_interval(interval: Duration) -> Self
	{
		return Self {
			interval: Arc::new(RwLock::new(interval)),
			join_handle: Arc::new(Mutex::new(None)),
			stop_flag: Arc::new((Mutex::new(false), Condvar::new())),
		};
	}
}
