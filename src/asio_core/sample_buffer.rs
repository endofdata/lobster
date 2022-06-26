use std::iter::*;

const MAX_I32_VALUE: f64 = 2147483647.0f64;
const PAN_LEFT: f64 = -1.0f64;
const PAN_RIGHT: f64 = 1.0f64;

pub trait WritableBufferPair {
	fn write(self, samples: &mut dyn Iterator<Item = f64>);
}

pub trait NativeBufferPair : Iterator<Item = f64> {
	fn select_buffer(&mut self, is_second: bool);

	fn reset(&mut self);

	fn as_writable(&mut self) -> &mut dyn WritableBufferPair;
}


trait SampleConvert {
	type Sample;

	fn from_native(&self) -> f64;
	fn to_native(sample: f64) -> Self::Sample;
}

pub struct BufferFactory {
}

impl BufferFactory {
	pub fn create<T: SampleConvert<Sample = T> + Copy>(ptr_a: *mut (), ptr_b: *mut (), len: usize) -> Box<impl NativeBufferPair> {
		let buffer_pair = NativeBufferPairOf::<T>::new(ptr_a as *mut T, ptr_b as *mut T, len);
		let boxed = Box::new(buffer_pair);

		boxed
	}
}

pub struct NativeBufferPairOf<T> where T: SampleConvert {
	pos: usize,
	len: usize,
	is_second: bool,
	ptr_a: *mut T,
	ptr_b: *mut T
}

impl<T: SampleConvert> NativeBufferPairOf<T> {
	fn new(ptr_a: *mut T, ptr_b: *mut T, len: usize) -> NativeBufferPairOf<T> {
		NativeBufferPairOf {
			pos: 0,
			len: len,
			is_second: false,
			ptr_a: ptr_a,
			ptr_b: ptr_b
		}
	}

	fn get_current(&self) -> *mut T {
		match self.is_second {
			true => self.ptr_a,
			false => self.ptr_b
		}
	}
}

impl<T: SampleConvert<Sample=T> + Copy> WritableBufferPair for NativeBufferPairOf<T> {
	fn write(mut self, samples: &mut dyn Iterator<Item = f64>) {
		let buffer = self.get_current();
		let avail = self.len - self.pos;

		for native_sample in samples.map(|s| T::to_native(s)).take(avail) {
			unsafe {
				*buffer.offset(self.pos as isize) = native_sample;
			}
			self.pos += 1;
		}
	}
}

impl<T: SampleConvert<Sample =T> + Copy> NativeBufferPair for NativeBufferPairOf<T> {
	fn select_buffer(&mut self, is_second: bool) {
		self.is_second = is_second;
	}

	fn reset(&mut self) {
		self.pos = 0;
	}

	fn as_writable<'a>(&'a mut self) -> &'a mut dyn WritableBufferPair {
		self
	}
}

impl<T: SampleConvert<Sample = T> + Copy> Iterator for NativeBufferPairOf<T> {
	type Item = f64;

	fn next(&mut self) -> Option<Self::Item> {
		match self.pos < self.len {
			true => {
				let native_sample;
				unsafe {
					native_sample = *self.get_current().offset(self.pos as isize);
				}
				self.pos += 1;
				Some(native_sample.from_native())
			},
			false => None
		}
	}
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

fn distribute_impl(iter_mono: impl Iterator<Item = f64>) -> impl Iterator<Item = (f64,f64)> {
	iter_mono.map(|s| (s, s))
}

// fn distribute_generic<I:Iterator<Item = (f64,f64)>>(iter_mono: impl Iterator<Item = f64>) -> I {
// 	iter_mono.map(|s| (s, s))
// }

// fn distribute<I>(iter_mono: impl Iterator<Item = f64>) -> I where I: Iterator<Item = (f64,f64)> {
//  	iter_mono.map(|s| (s, s))
// }

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
