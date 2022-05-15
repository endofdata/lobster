use std::iter::*;

pub trait SampleOutput {
	type Sample;
	fn write(&mut self, samples: &mut dyn Iterator<Item = f64>, auto_wrap: bool);
}

pub trait SampleInput {
	type Sample;
	fn read(&mut self, auto_wrap: bool) -> &mut dyn Iterator<Item = f64>;
}

pub struct SampleBufferFactory {
}

impl<'a> SampleBufferFactory {
	pub fn create_input_i32(ptr: *mut (), len: usize) -> Box<dyn SampleInput<Sample = i32>> {
		Box::new(SampleBuffer::<i32>::new(ptr as *mut i32, len, true))
	}
	pub fn create_output_i32(ptr: *mut (), len: usize) -> Box<dyn SampleOutput<Sample = i32>> {
		Box::new(SampleBuffer::<i32>::new(ptr as *mut i32, len, false))
	}
}

const MAX_I32_VALUE: f64 = 2147483647.0f64;

struct SampleBuffer<T: Copy + Clone> {
	is_input: bool,
	raw_samples: *mut T,
	len: usize,
	pos: usize
}

impl<T: Copy + Clone> SampleBuffer<T> {
	pub fn new(ptr: *mut T, len: usize, is_input: bool) -> SampleBuffer<T> {
		SampleBuffer {
			is_input: is_input,
			raw_samples: ptr,
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

impl Iterator for SampleBuffer<i32> {
	type Item = f64;

	fn  next(&mut self) -> Option<f64> {
		unsafe {
			let raw_sample = *self.raw_samples.offset(self.pos as isize);
			self.inc_pos();
			Some((raw_sample as f64) / MAX_I32_VALUE)
		}
	}
}

impl SampleOutput for SampleBuffer<i32> {
	type Sample = i32;

	fn write(&mut self, samples: &mut dyn Iterator<Item = f64>, auto_wrap: bool) {

		for raw_sample in samples.map(|s| (s * MAX_I32_VALUE) as i32) {
			unsafe {
				self.raw_samples.offset(self.pos as isize).write(raw_sample);
			}
			if self.inc_pos() == true && auto_wrap == false {
				break;				
			}
		}
	}
}

impl SampleInput for SampleBuffer<i32> {
	type Sample = i32;

	fn read(&mut self, auto_wrap: bool) -> &mut dyn Iterator<Item = f64> {		
		self
	}
}

