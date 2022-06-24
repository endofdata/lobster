use std::iter::*;

pub trait SampleDoubleBuffer {
	fn select_buffer(&mut self, is_second: bool);
	fn reset(&mut self);
}

pub trait SampleOutput : SampleDoubleBuffer {
	fn write(&mut self, samples: &mut dyn Iterator<Item = f64>);
}

pub trait SampleInput : SampleDoubleBuffer {
	fn read(&mut self) -> &mut dyn Iterator<Item = f64>;
}

pub struct SampleBufferFactory {
}

impl SampleBufferFactory {
	pub fn create_input_i32(ptr_a: *mut (), ptr_b: *mut (), len: usize) -> Box<dyn SampleInput> {
		Box::new(SampleBuffer::<i32>::new(ptr_a as *mut i32, ptr_b as *mut i32, len, true))
	}
	pub fn create_output_i32(ptr_a: *mut (), ptr_b: *mut (), len: usize) -> Box<dyn SampleOutput> {
		Box::new(SampleBuffer::<i32>::new(ptr_a as *mut i32, ptr_b as *mut i32, len, false))
	}
}

const MAX_I32_VALUE: f64 = 2147483647.0f64;

struct SampleBuffer<T: Copy + Clone> {
	#[allow(dead_code)]
	is_input: bool,
	raw_samples_a: *mut T,
	raw_samples_b: *mut T,
	current: *mut T,
	len: usize,
	pos: usize
}

impl<T: Copy + Clone> SampleBuffer<T> {
	pub fn new(ptr_a: *mut T, ptr_b: *mut T, len: usize, is_input: bool) -> SampleBuffer<T> {
		SampleBuffer {
			is_input: is_input,
			raw_samples_a: ptr_a,
			raw_samples_b: ptr_b,
			current: ptr_a,
			len: len,
			pos: 0
		}
	}

	fn inc_pos(&mut self) -> bool {
		self.pos += 1;
		if self.pos >= self.len {
			self.pos = 0;
			return true;
		}
		else {
			return false;
		}
	}
}

impl<T: Copy + Clone> SampleDoubleBuffer for SampleBuffer<T> {
	fn select_buffer(&mut self, is_second: bool) {
		self.current = match is_second {
			true => self.raw_samples_b,
			_ => self.raw_samples_a
		}
	}

	fn reset(&mut self) {
		self.pos = 0;
	}
}

impl Iterator for SampleBuffer<i32> {
	type Item = f64;

	fn  next(&mut self) -> Option<f64> {
		match self.inc_pos() {
			true => None,
			false => unsafe {
				let raw_sample = *self.current.offset(self.pos as isize);
				Some((raw_sample as f64) / MAX_I32_VALUE)
			}
		}
	}
}

impl SampleOutput for SampleBuffer<i32> {
	fn write(&mut self, samples: &mut dyn Iterator<Item = f64>) {
		for raw_sample in samples.map(|s| (s * MAX_I32_VALUE) as i32) {
			match self.inc_pos() {
				true => break,
				false => unsafe {
					self.current.offset(self.pos as isize).write(raw_sample)
				}
			};
		}
	}
}

impl SampleInput for SampleBuffer<i32> {
	fn read(&mut self) -> &mut dyn Iterator<Item = f64> {		
		self
	}
}

pub struct EmptyBuffer {
	len: usize,
	pos: usize,
	auto_wrap: bool
}

impl EmptyBuffer {
	pub fn new(len: usize, auto_wrap: bool) -> EmptyBuffer {
		EmptyBuffer {
			len: len,
			pos: 0,
			auto_wrap: auto_wrap
		}
	}
}

impl Iterator for EmptyBuffer {
	type Item = f64;

	fn next(&mut self) -> Option<f64> {
		if self.pos < self.len {
			self.pos += 1; 
			Some(0.0f64)
		}
		else if self.auto_wrap {
			self.pos = 1;
			Some(0.0f64)
		}
		else {
			None
		}
	}		
}

impl SampleDoubleBuffer for EmptyBuffer {

	fn select_buffer(&mut self, _is_second: bool) {
	}
	
	fn reset(&mut self) {
		self.pos = 0
	}
}


pub struct SamplePanner<'a> {
	samples_a: &'a mut dyn Iterator<Item = f64>,
	samples_b: &'a mut dyn Iterator<Item = f64>,
}

impl SamplePanner {
	pub fn new_mono(mono_samples: &mut dyn Iterator<Item = f64>) -> SamplePanner {
		let sample_array = mono_samples.collect::<Vec<f64>>();
		SamplePanner {
			samples_a: sample_array,
			samples_b: sample_array,
		}
	}

	pub fn new_stereo(samples_a: &mut dyn Iterator<Item = f64>, samples_b: &mut dyn Iterator<Item = f64>) -> SamplePanner {
		SamplePanner {
			samples_a: samples_a.collect(),
			samples_b: samples_b.collect(),
		}
	}

	pub fn mono<'a>(&self) -> &mut dyn Iterator<Item = f64> {
		&mut SampleCombiner::new(&self.samples_a.into_iter(), &self.samples_b.into_iter(), 0.0f64, 1.0f64)
	}

	pub fn left(&self) -> &mut dyn Iterator<Item = f64> {
		&mut SampleCombiner::new(&self.samples_a.into_iter(), &self.samples_b.into_iter(), -1.0f64, 1.0f64)
	}

	pub fn right(&self) -> &mut dyn Iterator<Item = f64> {
		&mut SampleCombiner::new(&self.samples_a.into_iter(), &self.samples_b.into_iter(), 1.0f64, 1.0f64)
	}
}

pub struct SampleCombiner<'a> {
	samples_a: &'a dyn Iterator<Item = f64>,
	samples_b: &'a dyn Iterator<Item = f64>,
	factor_left: f64,
	factor_right: f64
}

impl<'a> SampleCombiner<'a> {
	pub fn new(samples_a: &'a dyn Iterator<Item = f64>, samples_b: &'a dyn Iterator<Item = f64>, pan: f64, volume: f64) -> SampleCombiner<'a> {
		SampleCombiner {
			samples_a: samples_a,
			samples_b: samples_b,
			factor_left: (pan - 1.0f64) * -0.5f64 * volume,
			factor_right: (pan + 1.0f64) * 0.5f64 * volume	
		}
	}
}

impl<'a> Iterator for  SampleCombiner<'a> {
	type Item = f64;

	fn next(&mut self) -> Option<f64> {
		let left = self.samples_a.next();
		let right = self.samples_b.next();

		if left != None && right != None {
			Some(left.unwrap() * self.factor_left + right.unwrap() * self.factor_right)
		}
		else {
			None
		}
	}
}
