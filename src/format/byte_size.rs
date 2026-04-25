// byte_size.rs
// Copyright (c) 2025, yuki
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Alignment;
use std::fmt::Display;

/// A wrapper to format the given size (number) as a human-readable byte-count,
/// for example `34.1 kiB`.
#[derive(Debug, Clone, Copy)]
pub struct ByteSize(pub u64);

impl ByteSize
{
	/// Returns a customisable instance of `ByteSize`.
	pub const fn custom(self) -> ByteSizeCustom
	{
		return ByteSizeCustom {
			inner: self,
			scale: Scale::Binary,
			space: true,
			ibibytes: true,
			b_suffix: true,
			uppercase_k: false,
		};
	}
}

/// The scaling factor / style to use for printing bytes.
#[derive(Debug, Clone, Copy)]
pub enum Scale
{
	/// Binary scale, aka binary units, that scale by 1024s. Also known as kibibytes (kiB),
	/// mebibytes (MiB), etc.
	Binary,

	/// Decimal scale, aka SI units, that scale by 1000s. Sometimes known simply as kilobytes
	/// (kB), megabytes (MB), etc.
	Decimal,
}

/// A customisable variant of `ByteSize`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub struct ByteSizeCustom
{
	inner: ByteSize,
	scale: Scale,
	space: bool,
	ibibytes: bool,
	b_suffix: bool,
	uppercase_k: bool,
}

impl ByteSizeCustom
{
	/// Omits the space between the number and the unit. For example: `41.3kiB`
	#[must_use]
	pub const fn no_space(self) -> Self
	{
		return Self { space: false, ..self };
	}

	/// Prints a space between the number and the unit. For example: `41.3 kiB`
	#[must_use]
	pub const fn space(self) -> Self
	{
		return Self { space: true, ..self };
	}

	/// Omits the final `B` suffix. For example: `41.3 M`
	#[must_use]
	pub const fn no_b_suffix(self) -> Self
	{
		return Self { b_suffix: false, ..self };
	}

	/// Prints the final `B` suffix. For example: `41.3 MB`
	#[must_use]
	pub const fn b_suffix(self) -> Self
	{
		return Self { b_suffix: true, ..self };
	}

	/// Prints the 'k' in KB as uppercase.
	#[must_use]
	pub const fn uppercase_k(self) -> Self
	{
		return Self { uppercase_k: true, ..self };
	}

	/// Prints the 'k' in kB as lowercase.
	#[must_use]
	pub const fn lowercase_k(self) -> Self
	{
		return Self { uppercase_k: false, ..self };
	}

	/// Use SI units (scale by 1000s). Note that this does not change whether the 'i' is printed.
	#[must_use]
	pub const fn decimal(self) -> Self
	{
		return Self { scale: Scale::Decimal, ..self };
	}

	/// Use binary units (scale by 1024s). Note that this does not change whether the 'i' is printed.
	#[must_use]
	pub const fn binary(self) -> Self
	{
		return Self { scale: Scale::Binary, ..self };
	}

	/// Prints the 'i' regardless of whether SI or binary units are used.
	/// For example: `41.3 MiB`.
	#[must_use]
	pub const fn ibibytes(self) -> Self
	{
		return Self { ibibytes: true, ..self };
	}

	/// Omits the 'i' regardless of whether SI or binary units are used.
	/// For example: `41.3 MB`.
	#[must_use]
	pub const fn no_ibibytes(self) -> Self
	{
		return Self { ibibytes: false, ..self };
	}

	/// Prints a space between the number and the unit. For example:
	/// `41.3 MiB` vs `41.3MiB`.
	#[must_use]
	pub const fn with_space(self, space: bool) -> Self
	{
		return Self { space, ..self };
	}

	/// Sets the scale to use (binary or SI).
	#[must_use]
	pub const fn with_scale(self, scale: Scale) -> Self
	{
		return Self { scale, ..self };
	}

	/// Prints the 'i' between the prefix and the unit. For example:
	/// `41.3 MiB` vs `41.3MB`.
	#[must_use]
	pub const fn with_ibibytes(self, yes: bool) -> Self
	{
		return Self { ibibytes: yes, ..self };
	}

	/// Prints the 'B' unit. For example: `41.3MB` vs `41.3M`.
	#[must_use]
	pub const fn with_b_suffix(self, yes: bool) -> Self
	{
		return Self { b_suffix: yes, ..self };
	}

	/// Prints 'k' in kB as uppercase or lowercase. For example: `41.3kiB` vs `41.3KiB`.
	#[must_use]
	pub const fn with_uppercase_k(self, yes: bool) -> Self
	{
		return Self { uppercase_k: yes, ..self };
	}
}

const PREFIXES: [&str; 6] = ["", "k", "M", "G", "T", "P"];

impl Display for ByteSizeCustom
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		#[allow(clippy::cast_precision_loss)]
		let mut num = self.inner.0 as f64;
		let scale = match self.scale {
			Scale::Decimal => 1000.0,
			Scale::Binary => 1024.0,
		};

		let mut prefix = 0;
		while num >= scale {
			prefix += 1;
			num /= scale;
		}

		let prefix = PREFIXES[prefix];
		let prefix = if self.uppercase_k && prefix == "k" { "K" } else { prefix };

		let num_part = format_args!("{num:.*}", f.precision().unwrap_or(2));

		let space = if self.space { " " } else { "" };
		let b_suffix = if self.b_suffix { "B" } else { "" };
		let ibi = if self.ibibytes && !prefix.is_empty() { "i" } else { "" };

		let s = format!("{num_part}{space}{prefix}{ibi}{b_suffix}");
		let w = f.width();

		return match f.align() {
			None => write!(f, "{s}"),
			Some(Alignment::Left) => match w {
				Some(w) => write!(f, "{s:<w$}"),
				None => write!(f, "{s:<}"),
			},
			Some(Alignment::Center) => match w {
				Some(w) => write!(f, "{s:^w$}"),
				None => write!(f, "{s:^}"),
			},
			Some(Alignment::Right) => match w {
				Some(w) => write!(f, "{s:>w$}"),
				None => write!(f, "{s:>}"),
			},
		};
	}
}

impl Display for ByteSize
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		return self.custom().fmt(f);
	}
}
