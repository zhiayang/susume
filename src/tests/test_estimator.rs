// test_estimator.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::time::Duration;
use std::time::Instant;

use crate::Estimator;

#[test]
fn test_none_estimate_is_zero()
{
	let est = Estimator::none();
	let now = Instant::now();
	assert_eq!(est.estimate(now), 0.0);
}

#[test]
fn test_none_update_returns_zero()
{
	let mut est = Estimator::none();
	let now = Instant::now();
	assert_eq!(est.update(now, 100), 0.0);
}

#[test]
fn test_none_elapsed()
{
	let before = Instant::now();
	let est = Estimator::none();
	std::thread::sleep(Duration::from_millis(10));
	let after = Instant::now();
	let elapsed = est.elapsed(after);
	assert!(elapsed >= Duration::from_millis(10));
	assert!(elapsed <= after - before + Duration::from_millis(5));
}

#[test]
fn test_ema_estimate_at_startup_no_nan()
{
	// At startup, total_weight ~= 0 which used to cause NaN
	let est = Estimator::exponential();
	let now = Instant::now();
	let result = est.estimate(now);
	assert!(result.is_finite(), "estimate at startup should be finite, got {result}");
	assert_eq!(result, 0.0);
}

#[test]
fn test_ema_estimate_immediately_after_creation()
{
	// Estimate at the exact creation instant
	let est = Estimator::exponential();
	let now = Instant::now();
	let result = est.estimate(now);
	assert!(result.is_finite());
}

#[test]
fn test_ema_update_zero_delta_time()
{
	// Updating with zero time elapsed (now == last_update) should not panic
	let mut est = Estimator::exponential();
	let now = Instant::now();
	let result = est.update(now, 10);
	assert!(result.is_finite(), "update with zero delta_t should be finite, got {result}");
}

#[test]
fn test_ema_update_and_estimate()
{
	let mut est = Estimator::custom_exponential(Duration::from_secs(1));
	let start = Instant::now();

	// Simulate updates over time
	let t1 = start + Duration::from_millis(100);
	let t2 = start + Duration::from_millis(200);
	let t3 = start + Duration::from_millis(300);

	est.update(t1, 10);
	est.update(t2, 10);
	est.update(t3, 10);

	let rate = est.estimate(t3);
	assert!(rate.is_finite());
	assert!(rate > 0.0, "rate after updates should be positive, got {rate}");
}

#[test]
fn test_ema_reset()
{
	let mut est = Estimator::exponential();
	let start = Instant::now();
	let t1 = start + Duration::from_millis(100);
	est.update(t1, 100);

	let t2 = start + Duration::from_millis(200);
	est.reset(t2);

	// After reset, estimate should be 0 (no data)
	let result = est.estimate(t2);
	assert!(result.is_finite());
	assert_eq!(result, 0.0);
}

#[test]
fn test_simple_estimate_empty_window()
{
	let est = Estimator::simple();
	let now = Instant::now();
	assert_eq!(est.estimate(now), 0.0);
}

#[test]
fn test_simple_update_zero_delta_time()
{
	// First update at now == start_time → delta_t = 0 → should not panic or produce Inf
	let mut est = Estimator::simple();
	let now = Instant::now();
	let result = est.update(now, 10);
	assert!(result.is_finite(), "simple update with zero delta_t should be finite, got {result}");
}

#[test]
fn test_simple_estimate_single_sample_at_same_time()
{
	// After one update where now == newest, delta_t = 0 → should not produce NaN
	let mut est = Estimator::simple();
	let now = Instant::now();
	let t1 = now + Duration::from_millis(100);
	est.update(t1, 10);

	// Estimate at the same instant as the update
	let result = est.estimate(t1);
	assert!(result.is_finite(), "estimate at same time as update should be finite, got {result}");
}

#[test]
fn test_simple_update_and_estimate()
{
	let mut est = Estimator::custom_simple(10);
	let start = Instant::now();

	let t1 = start + Duration::from_millis(100);
	let t2 = start + Duration::from_millis(200);
	let t3 = start + Duration::from_millis(300);

	est.update(t1, 10);
	est.update(t2, 10);
	est.update(t3, 10);

	let t4 = start + Duration::from_millis(350);
	let rate = est.estimate(t4);
	assert!(rate.is_finite());
	assert!(rate > 0.0, "rate after updates should be positive, got {rate}");
}

#[test]
fn test_simple_window_overflow()
{
	// Window size = 3, push 5 updates → oldest 2 should be evicted
	let mut est = Estimator::custom_simple(3);
	let start = Instant::now();

	for i in 1..=5 {
		let t = start + Duration::from_millis(i * 100);
		est.update(t, 10);
	}

	let t = start + Duration::from_millis(600);
	let rate = est.estimate(t);
	assert!(rate.is_finite());
	assert!(rate > 0.0);
}

#[test]
fn test_simple_reset()
{
	let mut est = Estimator::simple();
	let start = Instant::now();
	let t1 = start + Duration::from_millis(100);
	est.update(t1, 100);

	let t2 = start + Duration::from_millis(200);
	est.reset(t2);

	assert_eq!(est.estimate(t2), 0.0);
}

#[test]
fn test_simple_elapsed()
{
	let est = Estimator::simple();
	std::thread::sleep(Duration::from_millis(10));
	let later = Instant::now();
	let elapsed = est.elapsed(later);
	assert!(elapsed >= Duration::from_millis(10));
}

#[test]
fn test_default_estimator_is_finite_at_startup()
{
	let est = Estimator::default();
	let now = Instant::now();
	let result = est.estimate(now);
	assert!(result.is_finite());
	assert_eq!(result, 0.0);
}
