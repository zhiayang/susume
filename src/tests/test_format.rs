// test_format.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use crate::fmt::ByteSize;

#[test]
fn test_byte_size_decimal()
{
	let dec = |x| ByteSize(x).custom().decimal().no_ibibytes();

	assert_eq!(format!("{}", dec(10)), "10.00 B");
	assert_eq!(format!("{:.0}", dec(10)), "10 B");
	assert_eq!(format!("{:.0}", dec(999)), "999 B");
	assert_eq!(format!("{:.0}", dec(1000)), "1 kB");

	assert_eq!(format!("{:.0}", dec(1_230_000)), "1 MB");
	assert_eq!(format!("{:.2}", dec(1_230_000)), "1.23 MB");
}

#[test]
fn test_byte_size_binary()
{
	let dec = |x| ByteSize(x).custom().binary();

	assert_eq!(format!("{}", dec(10)), "10.00 B");
	assert_eq!(format!("{:.0}", dec(10)), "10 B");
	assert_eq!(format!("{:.0}", dec(999)), "999 B");
	assert_eq!(format!("{:.0}", dec(1023)), "1023 B");
	assert_eq!(format!("{:.0}", dec(1024)), "1 kiB");

	assert_eq!(format!("{:.0}", dec(1_230_000)), "1 MiB");
	assert_eq!(format!("{:.2}", dec(1_230_000)), "1.17 MiB");
}

#[test]
fn test_byte_size_misc()
{
	let asdf = ByteSize(1_230_456).custom();

	assert_eq!(format!("{}", asdf.no_space()), "1.17MiB");
	assert_eq!(format!("{}", asdf.no_ibibytes()), "1.17 MB");
	assert_eq!(format!("{}", asdf.no_b_suffix()), "1.17 Mi");

	let asdf = ByteSize(3_518).custom();

	assert_eq!(format!("{}", asdf.no_space()), "3.44kiB");
	assert_eq!(format!("{}", asdf.uppercase_k()), "3.44 KiB");
}
