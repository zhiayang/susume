// estimator.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

use itertools::Itertools;

/// An estimator that can provide an estimated rate of progress (eg. download speed),
/// and can be used to derive other metrics (eg. eta).
pub struct Estimator
{
	storage: EstimatorStorage,
}

impl Estimator
{
	const DEFAULT_WINDOW_SIZE: usize = 128;
	const DEFAULT_TIME_FACTOR: Duration = Duration::from_secs(15);

	/// Creates an estimator that does nothing. The estimated value will always be `0`.
	pub fn none() -> Self
	{
		return Self { storage: EstimatorStorage::None(Instant::now()) };
	}

	/// Creates a simple moving-window-average estimator using the default window size.
	pub fn simple() -> Self
	{
		return Self {
			storage: EstimatorStorage::Simple(SimpleEstimator::new(Instant::now(), Self::DEFAULT_WINDOW_SIZE)),
		};
	}

	/// Creates a simple moving-window-average estimator using the specified window size.
	pub fn custom_simple(window_size: usize) -> Self
	{
		return Self {
			storage: EstimatorStorage::Simple(SimpleEstimator::new(Instant::now(), window_size)),
		};
	}

	/// Creates an exponentially-weighted time-based estimator with the default time factor.
	pub fn exponential() -> Self
	{
		return Self {
			storage: EstimatorStorage::Exponential(EmaEstimator::new(Instant::now(), Self::DEFAULT_TIME_FACTOR)),
		};
	}

	/// Creates an exponentially-weighted time-based estimator with the specified time factor.
	pub fn custom_exponential(time_factor: Duration) -> Self
	{
		return Self {
			storage: EstimatorStorage::Exponential(EmaEstimator::new(Instant::now(), time_factor)),
		};
	}

	/// Creates a custom estimator using the given user-defined implementation.
	pub fn custom(estimator: Box<dyn EstimatorImpl + Send + Sync>) -> Self
	{
		return Self { storage: EstimatorStorage::Custom(estimator) };
	}

	/// Retrieves the estimate from the estimator for the given time instant.
	pub fn estimate(&self, now: Instant) -> f64
	{
		use EstimatorStorage as S;
		return match &self.storage {
			S::None(_) => 0.0,
			S::Simple(est) => est.estimate(now),
			S::Exponential(est) => est.estimate(now),
			S::Custom(est) => est.estimate(now),
		};
	}

	/// Retrieves the elapsed time of the estimator at the given time instant.
	pub fn elapsed(&self, now: Instant) -> Duration
	{
		use EstimatorStorage as S;
		return match &self.storage {
			S::None(start) => now - *start,
			S::Simple(est) => est.elapsed(now),
			S::Exponential(est) => est.elapsed(now),
			S::Custom(est) => est.elapsed(now),
		};
	}

	/// Updates the estimator at the given time instant with the given delta progress. Negative progress
	/// is not supported.
	pub fn update(&mut self, now: Instant, delta: u64) -> f64
	{
		use EstimatorStorage as S;
		return match &mut self.storage {
			S::None(_) => 0.0,
			S::Simple(est) => est.update(now, delta),
			S::Exponential(est) => est.update(now, delta),
			S::Custom(est) => est.update(now, delta),
		};
	}

	/// Resets the estimator.
	pub fn reset(&mut self, now: Instant)
	{
		use EstimatorStorage as S;
		match &mut self.storage {
			S::None(start) => *start = now,
			S::Simple(est) => est.reset(now),
			S::Exponential(est) => est.reset(now),
			S::Custom(est) => est.reset(now),
		}
	}
}

impl Default for Estimator
{
	fn default() -> Self
	{
		return Self::exponential();
	}
}

impl EstimatorImpl for Estimator
{
	fn estimate(&self, now: Instant) -> f64
	{
		return Estimator::estimate(self, now);
	}

	fn elapsed(&self, now: Instant) -> Duration
	{
		return Estimator::elapsed(self, now);
	}

	fn update(&mut self, now: Instant, delta: u64) -> f64
	{
		return Estimator::update(self, now, delta);
	}

	fn reset(&mut self, now: Instant)
	{
		Estimator::reset(self, now);
	}
}

enum EstimatorStorage
{
	None(Instant),
	Simple(SimpleEstimator),
	Exponential(EmaEstimator),
	Custom(Box<dyn EstimatorImpl + Send + Sync>),
}

/// An estimator that provides an estimated progress rate.
pub trait EstimatorImpl
{
	/// Obtains an estimate of the rate.
	fn estimate(&self, now: Instant) -> f64;

	/// Obtains the elapsed time.
	fn elapsed(&self, now: Instant) -> Duration;

	/// Updates the estimator with the given delta.
	fn update(&mut self, now: Instant, delta: u64) -> f64;

	/// Resets the estimator.
	fn reset(&mut self, now: Instant);
}

/// A doubly-smoothed exponentially-weighted time-based estimator, similar to the one used in `indicatif`.
pub struct EmaEstimator
{
	start_time: Instant,
	last_update: Instant,
	smooth_sps: f64,
	dsmooth_sps: f64,

	time_decay_factor: Duration,
}

impl EmaEstimator
{
	/// Creates a new exponentially-weighted time-based estimator.
	pub fn new(now: Instant, time_decay_factor: Duration) -> Self
	{
		return Self {
			start_time: now,
			last_update: now,
			smooth_sps: 0.0,
			dsmooth_sps: 0.0,
			time_decay_factor,
		};
	}

	/// Same as indicatif -- the most recent T seconds of data have 0.9 weight,
	/// the next T..2T have 0.1 weight, 2T..3T has 0.01, etc.
	fn estimator_weight(&self, age: f64) -> f64
	{
		let factor = self.time_decay_factor.as_secs_f64();
		return (0.1_f64).powf(age / factor);
	}
}

impl EstimatorImpl for EmaEstimator
{
	#[allow(clippy::cast_precision_loss)]
	fn estimate(&self, now: Instant) -> f64
	{
		let delta_t = (now - self.last_update).as_secs_f64();
		let weight = self.estimator_weight(delta_t);

		let delta_t_start = (now - self.start_time).as_secs_f64();
		let total_weight = 1.0 - self.estimator_weight(delta_t_start);


		let sps = (weight * self.smooth_sps) / total_weight;
		let dsps = (weight * self.dsmooth_sps) + ((1.0 - weight) * sps);

		return dsps / total_weight;
	}

	fn elapsed(&self, now: Instant) -> Duration
	{
		return now - self.start_time;
	}

	#[allow(clippy::cast_precision_loss)]
	fn update(&mut self, now: Instant, amount: u64) -> f64
	{
		// instants are monotonic, so this always succeeds.
		let delta_t = now - self.last_update;
		if delta_t.is_zero() {
			return self.dsmooth_sps;
		}

		let delta_t = delta_t.as_secs_f64();

		let new_rate = (amount as f64) / delta_t;

		let weight = 1.0 - self.estimator_weight(delta_t);
		self.smooth_sps = (weight * new_rate) + ((1.0 - weight) * self.smooth_sps);

		// there are some long paragraphs in indicatif's Estimator that explains something
		// about normalisation and accounting for samples from t=-inf to t=0... but i'm not
		// smart enough to understand it, so i won't explain it.
		let delta_t_start = (now - self.start_time).as_secs_f64();
		let total_weight = 1.0 - self.estimator_weight(delta_t_start);

		let norm_smooth_sps = self.smooth_sps / total_weight;

		// determine the double smoothed value (EWA smoothing of the single EWA)
		self.dsmooth_sps = (weight * norm_smooth_sps) + ((1.0 - weight) * self.dsmooth_sps);

		self.last_update = now;
		return self.dsmooth_sps;
	}

	fn reset(&mut self, now: Instant)
	{
		self.start_time = now;
		self.last_update = now;

		self.smooth_sps = 0.0;
		self.dsmooth_sps = 0.0;
	}
}


/// A simple window-based moving average
pub struct SimpleEstimator
{
	window: VecDeque<(Instant, f64)>,
	start_time: Instant,
	max_size: usize,
}

impl SimpleEstimator
{
	/// Creates a new window-based moving average estimator.
	pub fn new(now: Instant, window_size: usize) -> Self
	{
		return Self {
			window: VecDeque::new(),
			start_time: now,
			max_size: window_size,
		};
	}

	fn sum(&self) -> Option<f64>
	{
		return self.window.iter().map(|x| x.1).sum1();
	}
}

impl EstimatorImpl for SimpleEstimator
{
	#[allow(clippy::cast_precision_loss)]
	fn estimate(&self, now: Instant) -> f64
	{
		let Some(sum) = self.sum() else {
			return 0.0;
		};

		let avg = sum / (self.window.len() as f64);

		// note: unwraps are safe because we know self.sum() returned Some(_)
		let oldest = self.window.front().unwrap().0;
		let newest = self.window.back().unwrap().0;
		let total_t = (newest - oldest).as_secs_f64();

		// cut the estimate depending on the fraction of time between the
		// total window duration, and the duration since the last update.
		//
		// for example, if our window was 700ms but we haven't had an update
		// (ie. now - newest) in 300ms, then we return avg * 700/(700+300).

		let delta_t = (now - newest).as_secs_f64();
		return avg * (total_t / (total_t + delta_t));
	}

	fn elapsed(&self, now: Instant) -> Duration
	{
		return now - self.start_time;
	}

	#[allow(clippy::cast_precision_loss)]
	fn update(&mut self, now: Instant, delta: u64) -> f64
	{
		while self.window.len() >= self.max_size {
			_ = self.window.pop_front();
		}

		let delta_t = (self.window.back().map_or(self.start_time, |x| x.0) - now).as_secs_f64();
		self.window.push_back((now, (delta as f64) / delta_t));

		// unwrap is ok because we just pushed an element
		return self.sum().unwrap() / (self.window.len() as f64);
	}

	fn reset(&mut self, now: Instant)
	{
		self.window.clear();
		self.start_time = now;
	}
}
