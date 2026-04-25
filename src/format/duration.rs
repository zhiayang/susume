// duration.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;
use std::time::Duration as StdDuration;
use std::time::SystemTime;

/// A wrapper around a `SystemTime` that prints the delta in human-readable format,
/// for example `1d 3h 41m`.
///
/// Both positive (future) and negative (past) deltas are
/// allowed, with the same output for either case.
#[derive(Debug, Clone, Copy)]
pub struct RelativeTime(pub SystemTime);

/// A wrapper around a `Duration` that formats in a human-readable manner, for
/// example `4m 31s`.
#[derive(Debug, Clone, Copy)]
pub struct Duration(pub StdDuration);

impl Display for RelativeTime
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		let now = SystemTime::now();
		if now < self.0 {
			return write!(f, "{}", Duration(self.0.duration_since(now).unwrap()));
		}

		return write!(f, "{}", Duration(now.duration_since(self.0).unwrap()));
	}
}

impl Display for Duration
{
	#[allow(unused_parens)]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		let mut secs = self.0.as_secs();
		let mut parts = vec![];

		if secs > (24 * 60 * 60) {
			let days = secs / (24 * 60 * 60);
			secs %= (24 * 60 * 60);

			parts.push(format!("{days}d"));
		}

		if secs > 60 * 60 {
			let hours = secs / (60 * 60);
			secs %= (60 * 60);

			if hours > 0 {
				parts.push(format!("{hours}h"));
			}
		}

		if secs > 60 {
			let mins = secs / 60;
			secs %= 60;

			if mins > 0 {
				parts.push(format!("{mins}m"));
			}
		}

		if secs > 0 {
			parts.push(format!("{secs}s"));
		}

		return write!(f, "{}", parts.join(" "));
	}
}
