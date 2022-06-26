use std::iter::*;

const MAX_I32_VALUE: f64 = 2147483647.0f64;
const PAN_LEFT: f64 = -1.0f64;
const PAN_RIGHT: f64 = 1.0f64;

trait SampleConvert {
	type Sample;

	fn from_native(&self) -> f64;
	fn to_native(sample: f64) -> Self::Sample;
}

impl SampleConvert for i32 {
	type Sample = i32;

	fn from_native(&self) -> f64 {
		(*self as f64) / MAX_I32_VALUE
	}

	fn to_native(sample: f64) -> Self::Sample {
		(sample * MAX_I32_VALUE) as Self::Sample
	}
}

/// Implements iterator for sample tuples (f64, f64) and conversion to [Mono]
pub struct Stereo<I> {
	iter_stereo: I
}

impl<I> Stereo<I> where I: Iterator<Item = (f64, f64)> {
	pub fn new(iter_stereo: I) -> Stereo<I> {
		Stereo {
			iter_stereo
		}
	}

	pub fn to_mono(self, vol: f64) -> Mono<impl Iterator<Item = f64>> {
		let factor_mono = vol;

		Mono::new(self.map(move |(left, right)| (left + right) * factor_mono))
	}

	pub fn left(self) -> Mono<impl Iterator<Item = f64>> {
		Mono::new(self.map(|(left, _)| left))
	}

	pub fn right(self) -> Mono<impl Iterator<Item = f64>> {
		Mono::new(self.map(|(_, right)| right))
	}
}

impl<I: Iterator<Item = (f64, f64)>> Iterator for Stereo<I> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter_stereo.next()
	}
}

/// Implements iterator for samples f64 and conversion to [Stereo]
pub struct Mono<I> {
	iter_mono: I
}

impl<I: Iterator<Item = f64>> Mono<I> {
	pub fn new(iter_mono: I) -> Mono<I> {
		Mono {
			iter_mono
		}
	}

	pub fn to_stereo(self, pan: f64, vol: f64) -> Stereo<impl Iterator<Item = (f64, f64)>> {
		let limited_pan = pan.max(PAN_LEFT).min(PAN_RIGHT);		
		let factor_left = (limited_pan + PAN_LEFT) * vol * -0.5;
		let factor_right = (limited_pan + PAN_RIGHT) * vol * 0.5;

		Stereo::new(self.iter_mono.map(move |m| (m * factor_left, m * factor_right)))
	}
}

impl<I: Iterator<Item = f64>> Iterator for Mono<I> {
	type Item = f64;

	fn next(&mut self) -> Option<f64> {
		self.iter_mono.next()
	}
}
